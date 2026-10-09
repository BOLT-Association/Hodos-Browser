// AuthBOLT presentation requests from a page (BoltRequest.h, header-only).
//
// A page asks Hodos to present an AuthBOLT identity with POST /bolt/request. The shell checks the
// request before it reaches the prompt: the app key is a compressed public key, the 66 bytes of
// auth data carry the purpose's tag and that same app key, and nothing else gets through to the
// overlay's URL. NC: make ValidateBoltRequest return true unconditionally — every refusal below
// goes red.
#include <gtest/gtest.h>

#include <string>

#include "core/BoltRequest.h"

namespace {

const std::string kApp = "02" + std::string(64, 'a');
const std::string kHash(64, 'b');

std::string Data(const std::string& tag, const std::string& app = kApp) { return tag + app + kHash; }

bool Valid(const std::string& app, const std::string& data, const std::string& purpose) {
    std::string why;
    return hodos::ValidateBoltRequest(app, data, purpose, why);
}

}  // namespace

TEST(BoltRequest, AWellFormedRequestPasses) {
    EXPECT_TRUE(Valid(kApp, Data("01"), "register"));
    EXPECT_TRUE(Valid(kApp, Data("02"), "signin"));
    EXPECT_TRUE(Valid(kApp, Data("03"), "refresh"));
    EXPECT_TRUE(Valid(kApp, Data("04"), "write"));
    EXPECT_TRUE(Valid("03" + std::string(64, '9'), Data("02", "03" + std::string(64, '9')), "signin"));
}

TEST(BoltRequest, TheAppKeyMustBeACompressedPublicKey) {
    EXPECT_FALSE(Valid("04" + std::string(64, 'a'), Data("02", "04" + std::string(64, 'a')), "signin"));
    EXPECT_FALSE(Valid("02" + std::string(62, 'a'), Data("02"), "signin"));
    EXPECT_FALSE(Valid("02" + std::string(64, 'A'), Data("02", "02" + std::string(64, 'A')), "signin")) << "lower-case hex only";
    EXPECT_FALSE(Valid("", Data("02"), "signin"));
}

TEST(BoltRequest, TheDataMustNameThisAppAndThisPurpose) {
    EXPECT_FALSE(Valid(kApp, Data("02", "02" + std::string(64, 'c')), "signin")) << "another app";
    EXPECT_FALSE(Valid(kApp, Data("01"), "signin")) << "a registration tag for a sign-in";
    EXPECT_FALSE(Valid(kApp, Data("04"), "signin")) << "a write's tag for a sign-in";
    EXPECT_FALSE(Valid(kApp, Data("03"), "write")) << "a keep-alive's tag for a write";
    EXPECT_FALSE(Valid(kApp, Data("05"), "signin")) << "an unknown tag";
    EXPECT_FALSE(Valid(kApp, Data("02").substr(2), "signin")) << "65 bytes";
    EXPECT_FALSE(Valid(kApp, Data("02") + "00", "signin")) << "67 bytes";
    EXPECT_FALSE(Valid(kApp, Data("02").replace(100, 1, "&"), "signin")) << "not hex: nothing else reaches the overlay URL";
}

// A write (one change a person makes in an app, its hash in the auth data) is signed only silently,
// under the keep-signed-in grant: it never opens the prompt, so a page cannot dress one up as a
// sign-in. NC: make ValidateBoltSilence return true unconditionally.
TEST(BoltRequest, AWriteIsOnlyEverSilent) {
    std::string why;
    EXPECT_TRUE(hodos::ValidateBoltSilence("write", true, why));
    EXPECT_FALSE(hodos::ValidateBoltSilence("write", false, why));
    EXPECT_NE(why.find("silent"), std::string::npos) << why;
    EXPECT_TRUE(hodos::ValidateBoltSilence("refresh", true, why));
    EXPECT_TRUE(hodos::ValidateBoltSilence("signin", false, why));
    EXPECT_TRUE(hodos::ValidateBoltSilence("register", false, why));
}

TEST(BoltRequest, OnlyTheFourPurposes) {
    EXPECT_FALSE(Valid(kApp, Data("02"), "pay"));
    EXPECT_FALSE(Valid(kApp, Data("02"), ""));
    EXPECT_FALSE(Valid(kApp, Data("02"), "signin&x=1"));
}

TEST(BoltRequest, TheRefusalSaysWhy) {
    std::string why;
    EXPECT_FALSE(hodos::ValidateBoltRequest(kApp, Data("02", "02" + std::string(64, 'c')), "signin", why));
    EXPECT_NE(why.find("app"), std::string::npos) << why;
}

// ---- POST /bolt/sign: holder-key signatures (ChainBrowsers docs/authbolt-registration.md, "After
// registration"). The wallet builds the digest itself; the shell checks only the shape of what the
// page sends, so nothing malformed reaches the prompt or the silent signer. NC: make
// ValidateBoltSign return true unconditionally — every refusal below goes red.

namespace {

bool Sign(const std::string& kind, const std::string& payload, bool silent = true, const std::string& app = kApp) {
    std::string why;
    return hodos::ValidateBoltSign(kind, app, payload, silent, why);
}

const std::string kWrite = R"({"v":1,"kind":"message.post","target":"POST /api/channels/1/messages","body":{"text":"hi"},"at":1,"seq":1,"sid":"s"})";

}  // namespace

TEST(BoltSign, EachKindTakesItsOwnPayload) {
    EXPECT_TRUE(Sign("signin", Data("02")));
    EXPECT_TRUE(Sign("refresh", Data("03")));
    EXPECT_TRUE(Sign("write", kWrite));
    EXPECT_TRUE(Sign("write", kWrite, false)) << "a prompted-tier write goes to the prompt";
    EXPECT_TRUE(Sign("rotate", ""));
    EXPECT_TRUE(Sign("confirm", ""));
    EXPECT_TRUE(Sign("recover", Data("02"), false));
    EXPECT_FALSE(Sign("pay", "")) << "an unknown kind";
    EXPECT_FALSE(Sign("Signin", Data("02"))) << "kind words are exact";
    EXPECT_FALSE(Sign("signin", "")) << "a sign-in signs the challenge";
    EXPECT_FALSE(Sign("signin", Data("02").replace(10, 1, "Z"))) << "the challenge is lower-case hex";
    EXPECT_FALSE(Sign("signin", std::string(266, 'a'))) << "a challenge is at most 132 bytes";
    EXPECT_FALSE(Sign("rotate", "x")) << "rotate and confirm carry nothing";
    EXPECT_FALSE(Sign("confirm", Data("02")));
    EXPECT_FALSE(Sign("signin", Data("02"), true, "04" + std::string(64, 'a'))) << "the app key";
}

TEST(BoltSign, AWriteIsAJsonObjectOfAtMost64KiB) {
    EXPECT_FALSE(Sign("write", "")) << "empty";
    EXPECT_FALSE(Sign("write", "[1,2]")) << "not an object";
    EXPECT_FALSE(Sign("write", "{\"v\":1")) << "not JSON";
    EXPECT_FALSE(Sign("write", R"({"v":1})")) << "names no kind";
    std::string big = R"({"kind":"message.post","body":")" + std::string(64 * 1024, 'x') + "\"}";
    EXPECT_FALSE(Sign("write", big)) << "over 64 KiB";
}

TEST(BoltSign, ARecoveryIsNeverSilent) {
    EXPECT_FALSE(Sign("recover", Data("02"), true));
}

// Only an https main frame may ask (audit H3): `cefMessage` reaches every frame, so a subframe or a
// plain-http page could otherwise send wallet_call itself. NC: make BoltFrameAllowed return true.
TEST(BoltSign, OnlyAnHttpsMainFrameMayAsk) {
    std::string why;
    EXPECT_TRUE(hodos::BoltFrameAllowed(true, "https://app.lab:8443/peerloop/", why));
    EXPECT_FALSE(hodos::BoltFrameAllowed(false, "https://app.lab:8443/peerloop/", why)) << "a subframe";
    EXPECT_FALSE(hodos::BoltFrameAllowed(true, "http://app.lab:8443/", why)) << "plain http";
    EXPECT_FALSE(hodos::BoltFrameAllowed(true, "data:text/html,https://x", why));
    EXPECT_FALSE(hodos::BoltFrameAllowed(true, "", why));
}

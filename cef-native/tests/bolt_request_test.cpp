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

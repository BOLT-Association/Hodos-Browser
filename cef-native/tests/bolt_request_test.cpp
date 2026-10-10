// AuthBOLT requests from a page (BoltRequest.h, header-only). ChainBrowsers
// docs/authbolt-onchain-holder-keys.md: an identity's token moves on chain, paid for by the app.
//
// A page asks Hodos to register an AuthBOLT identity with POST /bolt/request. The shell checks the
// request before it reaches the prompt: the app key is a compressed public key, the 70 bytes of auth
// data carry the register tag, that same app key and the holder count, and nothing else gets through
// to the overlay's URL. NC: make ValidateBoltRequest return true unconditionally — every refusal below
// goes red.
#include <gtest/gtest.h>

#include <string>

#include "core/BoltRequest.h"

namespace {

const std::string kApp = "02" + std::string(64, 'a');
const std::string kHash(64, 'b');

// Counted auth data (register 01, rotate 05, reissue 06): tag, app key, challenge hash, count.
std::string Counted(const std::string& tag, const std::string& count = "00000001", const std::string& app = kApp) {
    return tag + app + kHash + count;
}
// Uncounted (signin 02, refresh 03): tag, app key, challenge hash.
std::string Data(const std::string& tag, const std::string& app = kApp) { return tag + app + kHash; }

bool Valid(const std::string& app, const std::string& data, const std::string& purpose) {
    std::string why;
    return hodos::ValidateBoltRequest(app, data, purpose, why);
}

}  // namespace

TEST(BoltRequest, AWellFormedRegistrationPasses) {
    EXPECT_TRUE(Valid(kApp, Counted("01"), "register"));
    EXPECT_TRUE(Valid(kApp, Counted("01", "0000002a"), "register"));
    EXPECT_TRUE(Valid("03" + std::string(64, '9'), Counted("01", "00000001", "03" + std::string(64, '9')), "register"));
}

TEST(BoltRequest, TheAppKeyMustBeACompressedPublicKey) {
    EXPECT_FALSE(Valid("04" + std::string(64, 'a'), Counted("01", "00000001", "04" + std::string(64, 'a')), "register"));
    EXPECT_FALSE(Valid("02" + std::string(62, 'a'), Counted("01"), "register"));
    EXPECT_FALSE(Valid("02" + std::string(64, 'A'), Counted("01", "00000001", "02" + std::string(64, 'A')), "register")) << "lower-case hex only";
    EXPECT_FALSE(Valid("", Counted("01"), "register"));
}

TEST(BoltRequest, TheDataMustNameThisAppTheRegisterTagAndACount) {
    EXPECT_FALSE(Valid(kApp, Counted("01", "00000001", "02" + std::string(64, 'c')), "register")) << "another app";
    EXPECT_FALSE(Valid(kApp, Counted("05"), "register")) << "a rotation's tag";
    EXPECT_FALSE(Valid(kApp, Counted("01", "00000000"), "register")) << "count 0: holders count from 1";
    EXPECT_FALSE(Valid(kApp, Data("01"), "register")) << "66 bytes: no count";
    EXPECT_FALSE(Valid(kApp, Counted("01") + "00", "register")) << "71 bytes";
    EXPECT_FALSE(Valid(kApp, Counted("01").replace(100, 1, "&"), "register")) << "not hex: nothing else reaches the overlay URL";
}

// Only a registration is a presentation now: sign-in and keep-alives are holder signatures
// (POST /bolt/sign). NC: accept "signin" in ValidateBoltRequest.
TEST(BoltRequest, OnlyARegistration) {
    EXPECT_FALSE(Valid(kApp, Data("02"), "signin"));
    EXPECT_FALSE(Valid(kApp, Data("03"), "refresh"));
    EXPECT_FALSE(Valid(kApp, Counted("01"), "pay"));
    EXPECT_FALSE(Valid(kApp, Counted("01"), ""));
    EXPECT_FALSE(Valid(kApp, Counted("01"), "register&x=1"));
}

// A registration moves the token and asks the person which identity: never silent. NC: make
// ValidateBoltSilence return true unconditionally.
TEST(BoltRequest, ARegistrationIsNeverSilent) {
    std::string why;
    EXPECT_TRUE(hodos::ValidateBoltSilence("register", false, why));
    EXPECT_FALSE(hodos::ValidateBoltSilence("register", true, why));
    EXPECT_NE(why.find("silent"), std::string::npos) << why;
}

TEST(BoltRequest, TheRefusalSaysWhy) {
    std::string why;
    EXPECT_FALSE(hodos::ValidateBoltRequest(kApp, Counted("01", "00000001", "02" + std::string(64, 'c')), "register", why));
    EXPECT_NE(why.find("app"), std::string::npos) << why;
}

// ---- POST /bolt/sign: holder-key signatures, and the token's moves the app asks for. The wallet
// builds the digest or the transactions itself; the shell checks only the shape of what the page
// sends, so nothing malformed reaches the prompt or the silent signer. NC: make ValidateBoltSign
// return true unconditionally — every refusal below goes red.

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
    EXPECT_TRUE(Sign("rotate", Counted("05", "00000002"))) << "a rotation the app asked for, silently";
    EXPECT_TRUE(Sign("rotate", Counted("05", "00000002"), false));
    EXPECT_TRUE(Sign("reissue", Counted("06", "00000003"), false));
    EXPECT_FALSE(Sign("pay", "")) << "an unknown kind";
    EXPECT_FALSE(Sign("confirm", "")) << "the off-chain kinds are gone";
    EXPECT_FALSE(Sign("recover", Data("02"), false));
    EXPECT_FALSE(Sign("Signin", Data("02"))) << "kind words are exact";
    EXPECT_FALSE(Sign("signin", "")) << "a sign-in signs the challenge";
    EXPECT_FALSE(Sign("signin", Data("02").replace(10, 1, "Z"))) << "the challenge is lower-case hex";
    EXPECT_FALSE(Sign("signin", std::string(266, 'a'))) << "a challenge is at most 132 bytes";
    EXPECT_FALSE(Sign("signin", Data("02"), true, "04" + std::string(64, 'a'))) << "the app key";
}

TEST(BoltSign, AMoveIsCountedDataForThisApp) {
    EXPECT_FALSE(Sign("rotate", "")) << "a rotation carries the app's challenge";
    EXPECT_FALSE(Sign("rotate", Counted("06", "00000002"))) << "a reissue's tag";
    EXPECT_FALSE(Sign("rotate", Counted("05", "00000000"))) << "count 0";
    EXPECT_FALSE(Sign("rotate", Data("05"))) << "no count";
    EXPECT_FALSE(Sign("rotate", Counted("05", "00000002", "02" + std::string(64, 'c')))) << "another app";
    EXPECT_FALSE(Sign("reissue", Counted("05", "00000002"), false)) << "a rotation's tag";
}

// A resume hands the app a rotation it never received: the app's rotation data for that count, checked
// like a rotation's (the wallet signs and pays nothing for it).
TEST(BoltSign, AResumeNamesTheAppsRotationForACount) {
    EXPECT_TRUE(Sign("resume", Counted("05", "00000002")));
    EXPECT_TRUE(Sign("resume", Counted("05", "00000002"), false));
    EXPECT_FALSE(Sign("resume", Counted("06", "00000002"))) << "a reissue's tag";
    EXPECT_FALSE(Sign("resume", Counted("05", "00000000"))) << "count 0";
    EXPECT_FALSE(Sign("resume", Counted("05", "00000002", "02" + std::string(64, 'c')))) << "another app";
    EXPECT_FALSE(Sign("resume", "")) << "no data";
}

TEST(BoltSign, AWriteIsAJsonObjectOfAtMost64KiB) {
    EXPECT_FALSE(Sign("write", "")) << "empty";
    EXPECT_FALSE(Sign("write", "[1,2]")) << "not an object";
    EXPECT_FALSE(Sign("write", "{\"v\":1")) << "not JSON";
    EXPECT_FALSE(Sign("write", R"({"v":1})")) << "names no kind";
    std::string big = R"({"kind":"message.post","body":")" + std::string(64 * 1024, 'x') + "\"}";
    EXPECT_FALSE(Sign("write", big)) << "over 64 KiB";
}

TEST(BoltSign, AReissueIsNeverSilent) {
    EXPECT_FALSE(Sign("reissue", Counted("06", "00000003"), true));
}

// The app may name the holder it expects (a refused sign-in names it): a count and the holder's hash,
// together, for a signature only. They reach the wallet's script, so the shell checks them first.
// NC: make ValidateBoltSignHolder return true unconditionally.
TEST(BoltSign, TheExpectedHolderIsACountAndAHash) {
    const auto ok = [](const std::string& kind, const nlohmann::json& body) {
        std::string why;
        return hodos::ValidateBoltSignHolder(kind, body, why);
    };
    const std::string h(40, 'a');
    EXPECT_TRUE(ok("signin", nlohmann::json::object())) << "none named";
    EXPECT_TRUE(ok("signin", {{"count", 2}, {"holder", h}}));
    EXPECT_TRUE(ok("refresh", {{"count", 1}, {"holder", h}}));
    EXPECT_TRUE(ok("write", {{"count", 4294967295LL}, {"holder", h}}));
    EXPECT_FALSE(ok("rotate", {{"count", 2}, {"holder", h}})) << "a move names its count in its data";
    EXPECT_FALSE(ok("signin", {{"count", 2}})) << "a count without the hash";
    EXPECT_FALSE(ok("signin", {{"holder", h}})) << "a hash without the count";
    EXPECT_FALSE(ok("signin", {{"count", 0}, {"holder", h}}));
    EXPECT_FALSE(ok("signin", {{"count", 4294967296LL}, {"holder", h}}));
    EXPECT_FALSE(ok("signin", {{"count", "2"}, {"holder", h}})) << "a count as text";
    EXPECT_FALSE(ok("signin", {{"count", 2}, {"holder", std::string(40, 'A')}})) << "upper case";
    EXPECT_FALSE(ok("signin", {{"count", 2}, {"holder", h + "aa"}})) << "21 bytes";
    EXPECT_FALSE(ok("signin", {{"count", 2}, {"holder", "a'" + std::string(38, 'a')}}));
}

// The name an app accepted for an identity (POST /bolt/label, after a successful sign-in): the wallet
// keeps it to show instead of a key. It reaches the wallet's script, so the shell checks it first.
// NC: make ValidateBoltLabel return true unconditionally.
TEST(BoltLabel, ANameIsShortTextForAnIdentityOfThisApp) {
    const auto ok = [](const std::string& app, const std::string& identity, const std::string& name) {
        std::string why;
        return hodos::ValidateBoltLabel(app, identity, name, why);
    };
    const std::string id = "03" + std::string(64, 'c');
    EXPECT_TRUE(ok(kApp, id, "fred"));
    EXPECT_TRUE(ok(kApp, id, "admin-123.b_c"));
    EXPECT_TRUE(ok(kApp, id, "Zo\xc3\xab"));  // UTF-8 text ("Zo" + e with diaeresis)
    EXPECT_TRUE(ok(kApp, id, std::string(64, 'x')));
    EXPECT_FALSE(ok(kApp, id, ""));
    EXPECT_FALSE(ok(kApp, id, std::string(65, 'x'))) << "over 64 characters";
    EXPECT_FALSE(ok(kApp, id, "a\nb")) << "a control character";
    EXPECT_FALSE(ok(kApp, id, std::string("a\0b", 3)));
    EXPECT_FALSE(ok(kApp, id, "\xff\xfe")) << "not UTF-8";
    EXPECT_FALSE(ok("04" + std::string(64, 'a'), id, "fred")) << "not an app key";
    EXPECT_FALSE(ok(kApp, "zz" + std::string(64, 'c'), "fred")) << "not an identity key";
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

// ---- The app pays: while a request is pending, the wallet (in Hodos's overlay) asks the page for a
// coin and the page answers. Both cross into another process's JavaScript, so the shell checks their
// shape first. NC: make ValidateBoltFund / ValidateBoltFundResult return true unconditionally.

TEST(BoltFund, TheIdIsShortAndPlain) {
    EXPECT_TRUE(hodos::IsBoltFundId("fundlxk3z0"));
    EXPECT_TRUE(hodos::IsBoltFundId(""));  // a request without funding
    EXPECT_FALSE(hodos::IsBoltFundId("a'b"));
    EXPECT_FALSE(hodos::IsBoltFundId(std::string(65, 'a')));
    EXPECT_FALSE(hodos::IsBoltFundId("a b"));
}

TEST(BoltFund, ACoinRequestIsAStepAnAmountATransactionAndTheNextInput) {
    std::string why;
    EXPECT_TRUE(hodos::ValidateBoltFund("commit", 321, "0200000001", 1, why)) << why;
    EXPECT_TRUE(hodos::ValidateBoltFund("settle", 5, "02", 2, why)) << why;
    EXPECT_FALSE(hodos::ValidateBoltFund("melt", 321, "02", 1, why)) << "only commit and settle";
    EXPECT_FALSE(hodos::ValidateBoltFund("commit", 0, "02", 1, why)) << "no coin of 0";
    EXPECT_FALSE(hodos::ValidateBoltFund("commit", 1000001, "02", 1, why)) << "over the shell's cap";
    EXPECT_FALSE(hodos::ValidateBoltFund("commit", 5, "", 1, why)) << "no transaction";
    EXPECT_FALSE(hodos::ValidateBoltFund("commit", 5, "0G", 1, why)) << "not hex";
    EXPECT_FALSE(hodos::ValidateBoltFund("commit", 5, "02", -1, why));
    EXPECT_FALSE(hodos::ValidateBoltFund("commit", 5, "02", 17, why));
    EXPECT_FALSE(hodos::ValidateBoltFund("commit", 5, std::string(400002, 'a'), 1, why)) << "too big";
}

TEST(BoltFund, ThePagesAnswerIsACoinOrAnError) {
    std::string why;
    EXPECT_TRUE(hodos::ValidateBoltFundResult(nlohmann::json::parse(R"({"key":"bolt-7","fundId":"f1","coin":{"tx":"0101","vout":0,"unlockingScript":"47aa"}})"), why)) << why;
    EXPECT_TRUE(hodos::ValidateBoltFundResult(nlohmann::json::parse(R"({"key":"bolt-7","fundId":"f1","error":"the app has no funds"})"), why)) << why;
    EXPECT_FALSE(hodos::ValidateBoltFundResult(nlohmann::json::parse(R"({"key":"bolt-7","fundId":"f1"})"), why)) << "neither";
    EXPECT_FALSE(hodos::ValidateBoltFundResult(nlohmann::json::parse(R"({"key":"x","fundId":"f1","error":"e"})"), why)) << "not a bolt key";
    EXPECT_FALSE(hodos::ValidateBoltFundResult(nlohmann::json::parse(R"({"key":"bolt-7","fundId":"f1","coin":{"tx":"zz","vout":0,"unlockingScript":"47"}})"), why)) << "not hex";
    EXPECT_FALSE(hodos::ValidateBoltFundResult(nlohmann::json::parse(R"({"key":"bolt-7","fundId":"f1","coin":{"tx":"01","vout":-1,"unlockingScript":"47"}})"), why)) << "a vout below 0";
    EXPECT_FALSE(hodos::ValidateBoltFundResult(nlohmann::json::parse(R"({"key":"bolt-7","fundId":"f1","error":")" + std::string(2000, 'e') + R"("})"), why)) << "an error over 1000 characters";
}

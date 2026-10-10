// AuthBOLT requests from a page (POST /bolt/request and /bolt/sign over the wallet_call bridge).
//
// An identity's token moves on chain, paid for by the app (ChainBrowsers
// docs/authbolt-onchain-holder-keys.md). A page never moves it itself: it asks, Hodos shows its own
// prompt (the "bolt_request" / "bolt_sign" types of BRC100AuthOverlayRoot) or answers silently under
// the person's keep-signed-in grant, and the page receives only the result.
//
//   POST /bolt/request  register: the issuer key moves the token from its mint to holder 1.
//                       70 bytes of auth data: [tag 01][app key 33][challenge hash 32][count 4].
//   POST /bolt/sign     signin / refresh: a holder signature over the app's challenge (hex);
//                       write: one change, a JSON object naming its kind; rotate (tag 05) and
//                       reissue (tag 06, never silent): the token's next move, counted data as above.
//
// While a request is pending the wallet (in the overlay) may ask the page for coins: the overlay sends
// bolt_fund, the shell runs the page's __boltFund, and the page answers with POST /bolt/fund-result.
// The shell checks the shape of everything before it reaches the prompt's URL or another process's
// JavaScript: only lower-case hex, the known words and plain ids pass. Header-only and pure, so it is
// unit-tested without CEF (tests/bolt_request_test.cpp).
#pragma once

#include <string>

#include <nlohmann/json.hpp>

namespace hodos {

inline bool IsLowerHex(const std::string& s) {
    if (s.empty() || s.size() % 2 != 0) return false;
    for (char c : s) {
        if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'))) return false;
    }
    return true;
}

inline bool IsAppKey(const std::string& k) {
    return k.size() == 66 && IsLowerHex(k) && (k.compare(0, 2, "02") == 0 || k.compare(0, 2, "03") == 0);
}

// Counted auth data for `tag` (01 register, 05 rotate, 06 reissue) naming `appPubKey`: 70 bytes, the
// holder count at least 1. False with the reason in `why`.
inline bool ValidateCountedData(const std::string& tag, const std::string& appPubKey, const std::string& data,
                                std::string& why) {
    if (data.size() != 140 || !IsLowerHex(data)) {
        why = "the auth data must be exactly 70 bytes (lower-case hex)";
        return false;
    }
    if (data.compare(0, 2, tag) != 0) {
        why = "the auth data is for another purpose";
        return false;
    }
    if (data.compare(2, 66, appPubKey) != 0) {
        why = "the auth data names another app than the one asking";
        return false;
    }
    if (data.compare(132, 8, "00000000") == 0) {
        why = "the holder count starts at 1";
        return false;
    }
    return true;
}

// True when a registration request may go to the prompt; otherwise false with the reason in `why`.
inline bool ValidateBoltRequest(const std::string& appPubKey, const std::string& data,
                                const std::string& purpose, std::string& why) {
    if (purpose != "register") {
        why = "a presentation is only ever a registration (sign-in and keep-alives are /bolt/sign)";
        return false;
    }
    if (!IsAppKey(appPubKey)) {
        why = "the app key must be a 33-byte compressed public key (lower-case hex)";
        return false;
    }
    return ValidateCountedData("01", appPubKey, data, why);
}

// A registration moves the token and asks the person which identity: never silent. False with the
// reason in `why`.
inline bool ValidateBoltSilence(const std::string& purpose, bool silent, std::string& why) {
    if (purpose == "register" && silent) {
        why = "a registration is never silent: the person chooses the identity";
        return false;
    }
    return true;
}

constexpr std::size_t kBoltSignMaxPayload = 64 * 1024;

// True when a /bolt/sign request may go on; otherwise false with the reason in `why`.
inline bool ValidateBoltSign(const std::string& kind, const std::string& appPubKey,
                             const std::string& payload, bool silent, std::string& why) {
    if (!IsAppKey(appPubKey)) {
        why = "the app key must be a 33-byte compressed public key (lower-case hex)";
        return false;
    }
    if (kind == "signin" || kind == "refresh") {
        if (payload.size() > 264 || !IsLowerHex(payload)) {
            why = "a " + kind + " signs the app's challenge: lower-case hex, at most 132 bytes";
            return false;
        }
        return true;
    }
    if (kind == "rotate" || kind == "reissue" || kind == "resume") {
        if (kind == "reissue" && silent) {
            why = "a reissue is never silent";
            return false;
        }
        // A resume hands the app a rotation it never received: the app's rotation data for that count.
        return ValidateCountedData(kind == "reissue" ? "06" : "05", appPubKey, payload, why);
    }
    if (kind == "write") {
        if (payload.empty() || payload.size() > kBoltSignMaxPayload) {
            why = "a write is at most 64 KiB";
            return false;
        }
        const nlohmann::json w = nlohmann::json::parse(payload, nullptr, false);
        if (w.is_discarded() || !w.is_object() || !w.contains("kind") || !w["kind"].is_string()) {
            why = "a write is the change as a JSON object naming its kind";
            return false;
        }
        return true;
    }
    why = "kind must be signin, refresh, write, rotate, reissue or resume";
    return false;
}

// The holder the app expects, named with a signature (a refused sign-in names it): `count` (1 to
// 2^32-1) and `holder` (its hash, 20 bytes lower-case hex), both or neither, for signin, refresh and
// write only (a move names its count in its auth data). The wallet derives that holder's key and signs
// with it only if it hashes to `holder`. False with the reason in `why`.
inline bool ValidateBoltSignHolder(const std::string& kind, const nlohmann::json& body, std::string& why) {
    const bool hasCount = body.contains("count"), hasHolder = body.contains("holder");
    if (!hasCount && !hasHolder) return true;
    if (kind != "signin" && kind != "refresh" && kind != "write") {
        why = "only a signature names the holder the app expects";
        return false;
    }
    if (!hasCount || !hasHolder) {
        why = "the expected holder is a count and its hash, together";
        return false;
    }
    const nlohmann::json& c = body["count"];
    if (!c.is_number_integer() || c.get<long long>() < 1 || c.get<long long>() > 4294967295LL) {
        why = "the holder count is a whole number from 1";
        return false;
    }
    const nlohmann::json& h = body["holder"];
    if (!h.is_string() || h.get<std::string>().size() != 40 || !IsLowerHex(h.get<std::string>())) {
        why = "the holder is its hash: 20 bytes of lower-case hex";
        return false;
    }
    return true;
}

// The name an app accepted for one of its identities (POST /bolt/label): the app's key, the identity
// (its issuer key) and 1 to 64 characters of UTF-8 text with no control characters. False with the
// reason in `why`.
inline bool ValidateBoltLabel(const std::string& appPubKey, const std::string& identity, const std::string& name,
                              std::string& why) {
    if (!IsAppKey(appPubKey)) {
        why = "the app key must be a 33-byte compressed public key (lower-case hex)";
        return false;
    }
    if (!IsAppKey(identity)) {
        why = "the identity is its issuer key: 33 bytes, compressed, lower-case hex";
        return false;
    }
    std::size_t chars = 0;
    for (std::size_t i = 0; i < name.size(); ++chars) {
        const unsigned char c = static_cast<unsigned char>(name[i]);
        const std::size_t len = c < 0x80 ? 1 : (c >> 5) == 0x6 ? 2 : (c >> 4) == 0xe ? 3 : (c >> 3) == 0x1e ? 4 : 0;
        if (len == 0 || i + len > name.size()) {
            why = "a name is UTF-8 text";
            return false;
        }
        for (std::size_t k = 1; k < len; ++k) {
            if ((static_cast<unsigned char>(name[i + k]) >> 6) != 0x2) {
                why = "a name is UTF-8 text";
                return false;
            }
        }
        if (len == 1 && (c < 0x20 || c == 0x7f)) {
            why = "a name has no control characters";
            return false;
        }
        i += len;
    }
    if (chars < 1 || chars > 64) {
        why = "a name is 1 to 64 characters";
        return false;
    }
    return true;
}

// Only an https main frame may ask for an identity or a signature (audit H3): `cefMessage`, the raw
// IPC under the wallet bridge, reaches every frame, so a subframe or a plain-http page could
// otherwise send the request itself. `frameUrl` is the asking frame's own URL.
inline bool BoltFrameAllowed(bool isMainFrame, const std::string& frameUrl, std::string& why) {
    if (!isMainFrame) {
        why = "only the page itself may ask, not a frame inside it";
        return false;
    }
    if (frameUrl.compare(0, 8, "https://") != 0) {
        why = "only an https page may ask";
        return false;
    }
    return true;
}

// ---- The app pays: the page's funding id, the wallet's coin request, the page's answer ----

// A page's funding id (it names the page's fund callback): letters, digits, '-' and '_', at most 64;
// empty when the request carries none.
inline bool IsBoltFundId(const std::string& id) {
    if (id.size() > 64) return false;
    for (char c : id) {
        if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '-' || c == '_')) return false;
    }
    return true;
}

constexpr long long kBoltFundMaxAmount = 1000000;  // the shell's own cap; the app has a lower one
constexpr std::size_t kBoltFundMaxTx = 400000;      // hex characters of the wallet's draft

// The wallet's coin request (from the overlay, to the page): a step of the move, the amount, the
// draft (raw hex) and the coin's input index. False with the reason in `why`.
inline bool ValidateBoltFund(const std::string& step, long long amount, const std::string& tx, long long index,
                             std::string& why) {
    if (step != "commit" && step != "settle") {
        why = "a coin is for a commit or a settle";
        return false;
    }
    if (amount < 1 || amount > kBoltFundMaxAmount) {
        why = "the amount is outside the shell's limit";
        return false;
    }
    if (tx.empty() || tx.size() > kBoltFundMaxTx || !IsLowerHex(tx)) {
        why = "the transaction must be raw lower-case hex";
        return false;
    }
    if (index < 0 || index > 16) {
        why = "the coin's input index is out of range";
        return false;
    }
    return true;
}

// The page's answer (POST /bolt/fund-result): {key, fundId, coin: {tx, vout, unlockingScript}} or
// {key, fundId, error}. False with the reason in `why`.
inline bool ValidateBoltFundResult(const nlohmann::json& r, std::string& why) {
    if (!r.is_object() || !r.contains("key") || !r["key"].is_string() || r["key"].get<std::string>().rfind("bolt-", 0) != 0 ||
        r["key"].get<std::string>().size() > 32) {
        why = "the answer names no pending request";
        return false;
    }
    if (!r.contains("fundId") || !r["fundId"].is_string() || r["fundId"].get<std::string>().empty() ||
        !IsBoltFundId(r["fundId"].get<std::string>())) {
        why = "the answer carries no funding id";
        return false;
    }
    if (r.contains("error")) {
        if (!r["error"].is_string() || r["error"].get<std::string>().size() > 1000) {
            why = "the error must be text of at most 1000 characters";
            return false;
        }
        return true;
    }
    if (!r.contains("coin") || !r["coin"].is_object()) {
        why = "the answer carries neither a coin nor an error";
        return false;
    }
    const nlohmann::json& c = r["coin"];
    if (!c.contains("tx") || !c["tx"].is_string() || !IsLowerHex(c["tx"].get<std::string>()) ||
        c["tx"].get<std::string>().size() > 4 * 1024 * 1024) {
        why = "the coin's transaction must be lower-case hex";
        return false;
    }
    if (!c.contains("vout") || !c["vout"].is_number_integer() || c["vout"].get<long long>() < 0 || c["vout"].get<long long>() > 100000) {
        why = "the coin's output index is out of range";
        return false;
    }
    if (!c.contains("unlockingScript") || !c["unlockingScript"].is_string() || !IsLowerHex(c["unlockingScript"].get<std::string>()) ||
        c["unlockingScript"].get<std::string>().size() > 2000) {
        why = "the coin's unlocking script must be lower-case hex";
        return false;
    }
    return true;
}

}  // namespace hodos

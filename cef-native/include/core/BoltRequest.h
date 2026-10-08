// AuthBOLT presentation requests from a page (POST /bolt/request over the wallet_call bridge).
//
// A page never presents an AuthBOLT identity itself: it asks, Hodos shows its own prompt (the
// "bolt_request" type of BRC100AuthOverlayRoot), and the person chooses or creates the identity.
// The page receives only the presentation. ChainBrowsers docs/authbolt-registration.md.
//
// The request carries the app's public key, 66 bytes of auth data ([purpose tag 1][app key 33]
// [challenge hash 32]) and the purpose. The shell checks all three before anything else happens:
// the overlay is opened with them in its URL, so only lower-case hex and the four purpose words
// may pass (nothing a page could use to add a query parameter or reach script). A write (tag 04,
// the hash of one change a person makes in the app) is signed silently or not at all. Header-only and
// pure, so it is unit-tested without CEF (tests/bolt_request_test.cpp).
#pragma once

#include <string>

namespace hodos {

inline bool IsLowerHex(const std::string& s) {
    if (s.empty() || s.size() % 2 != 0) return false;
    for (char c : s) {
        if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'))) return false;
    }
    return true;
}

// The auth data's tag for a purpose; empty for anything else.
inline std::string BoltPurposeTag(const std::string& purpose) {
    if (purpose == "register") return "01";
    if (purpose == "signin") return "02";
    if (purpose == "refresh") return "03";
    if (purpose == "write") return "04";
    return "";
}

// True when the request may go to the prompt; otherwise false with the reason in `why`.
inline bool ValidateBoltRequest(const std::string& appPubKey, const std::string& data,
                                const std::string& purpose, std::string& why) {
    const std::string tag = BoltPurposeTag(purpose);
    if (tag.empty()) {
        why = "purpose must be register, signin, refresh or write";
        return false;
    }
    if (appPubKey.size() != 66 || !IsLowerHex(appPubKey) ||
        (appPubKey.compare(0, 2, "02") != 0 && appPubKey.compare(0, 2, "03") != 0)) {
        why = "the app key must be a 33-byte compressed public key (lower-case hex)";
        return false;
    }
    if (data.size() != 132 || !IsLowerHex(data)) {
        why = "the auth data must be exactly 66 bytes (lower-case hex)";
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
    return true;
}

// A write (tag 04: one change a person makes in an app) is signed only silently, under the
// person's keep-signed-in grant, never through the prompt. False with the reason in `why`.
inline bool ValidateBoltSilence(const std::string& purpose, bool silent, std::string& why) {
    if (purpose == "write" && !silent) {
        why = "a write is only ever signed silently, under the keep-signed-in grant";
        return false;
    }
    return true;
}

}  // namespace hodos

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

#include <nlohmann/json.hpp>

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

// ---- POST /bolt/sign: holder-key signatures (ChainBrowsers docs/authbolt-registration.md, "After
// registration"). An app that registered an identity knows its holder key, and the page then asks
// the wallet to sign with it: `signin` / `refresh` (the app's challenge, lower-case hex), `write`
// (the change as a JSON object naming its kind, at most 64 KiB), `rotate` / `confirm` (nothing: the
// wallet moves the app's signing key itself) and `recover` (the challenge; issuer key; never
// silent). The wallet builds the digest itself and decides what it signs silently; the shell only
// checks the shape, so nothing malformed reaches the prompt or the silent signer.

constexpr std::size_t kBoltSignMaxPayload = 64 * 1024;

inline bool IsAppKey(const std::string& k) {
    return k.size() == 66 && IsLowerHex(k) && (k.compare(0, 2, "02") == 0 || k.compare(0, 2, "03") == 0);
}

// True when the request may go on; otherwise false with the reason in `why`.
inline bool ValidateBoltSign(const std::string& kind, const std::string& appPubKey,
                             const std::string& payload, bool silent, std::string& why) {
    if (!IsAppKey(appPubKey)) {
        why = "the app key must be a 33-byte compressed public key (lower-case hex)";
        return false;
    }
    if (kind == "signin" || kind == "refresh" || kind == "recover") {
        if (payload.size() > 264 || !IsLowerHex(payload)) {
            why = "a " + kind + " signs the app's challenge: lower-case hex, at most 132 bytes";
            return false;
        }
        if (kind == "recover" && silent) {
            why = "a recovery is never silent";
            return false;
        }
        return true;
    }
    if (kind == "rotate" || kind == "confirm") {
        if (!payload.empty()) {
            why = "a " + kind + " carries nothing: the wallet chooses the key";
            return false;
        }
        return true;
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
    why = "kind must be signin, refresh, write, rotate, confirm or recover";
    return false;
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

}  // namespace hodos

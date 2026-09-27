# 🔍 The find bar does not follow a tab switch — the new tab shows no results until you retype

**Found:** 2026-09-27, 👤 owner: *"we do the Ctrl+F on one tab, and then we switch tabs… I think we just have to type in it."*
**Status:** 📌 PROPOSED (G2) · **Track:** B5-T6 Browser shell · **Filed by:** Claude, at the owner's request

> ⚠️ **Method note.** **Code reading** (`frontend/src/components/FindBar.tsx`) + Chromium source read 2026-09-27
> (`chrome/browser/ui/find_bar/find_bar_controller.cc`, via the GitHub mirror). Not run.
> Related but different: `TICKET_window_scoped_work_uses_process_globals.md` row 1 (Ctrl+F in window B opens in window A).

---

## What happens

Our find bar is **one bar per window header**, not per tab (`MainBrowserView.tsx` holds `findBarVisible`;
`FindBar.tsx` holds `query`). It sends `find_text` **only when the input changes** (`handleInputChange`). Switch tabs
and the bar stays open with the old text, but **no search runs on the new tab** — no highlights, no count — until the
user types. The previous tab's highlights are also left behind.

## How Chromium does it (read from source, 2026-09-27)

- **Find state is per tab** (`FindTabHelper` per tab). On a tab switch (`FindBarController::ChangeWebContents`) the bar
  **hides** unless the new tab has its own active find, and **returning to a tab restores its search**.
- Opening find in a tab that has none **prepopulates the last search from any tab** (`MaybeSetPrepopulateText`:
  *"Usually, this will be the last search in this tab, but if no search has been issued in this tab we use the last
  search string (from any tab)"*), shown selected, with matches highlighted but no jump (`StartFinding(… false /*
  find_match */)`).
⇒ In Chrome the user **does not retype**. ⚠️ Brave inherits Chromium's find bar; Firefox not checked.

## Proposed fix

Adopt Chromium's model: per-tab find state (query + open/closed) held against the tab id; on tab switch hide or restore
accordingly; Ctrl+F on a tab without a search prepopulates the last query, selected, and highlights without jumping.
Clear a tab's highlights when its find closes.

## Test and negative control (sketch)

| | |
|---|---|
| **GREEN** | Find "wallet" on tab A → switch to B → Ctrl+F → the field shows "wallet", selected, with B's match count; back to A → A's search is restored |
| **RED** | Current build: on B the count stays empty until typing |
| **SUBJECT** | The find result count reported for **the active tab's browser**, not the previous one — assert the tab id in the result |

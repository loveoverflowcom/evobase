# Accessibility and state acceptance

Design targets only. Arithmetic/static inspection results are in qa; keyboard/browser/native behavior must be tested during implementation.

## Global state vocabulary

| State | Plain user copy | Behavior / invariant |
|---|---|---|
| Loading | Đang tải… | skeleton only for known layout; status announced, no fake numbers |
| Empty | Chưa có đơn hàng | one authorized create action, no marketing dashboard |
| Denied | Bạn không có quyền xem mục này | no hidden title/count/details; navigate to allowed view |
| Invalid | Kiểm tra trường được đánh dấu | field-associated explanation; input retained; first error focused |
| Saving | Đang lưu… | prevent duplicate intent; keep cancellation semantics clear |
| Saved | Đã lưu | only after receipt; local saved separately named |
| Local pending | Đã giữ nháp trên thiết bị | never called server confirmed/sent |
| Conflict | Đơn đã được chỉnh ở nơi khác | compare/review; no silent overwrite or auto approval |
| Session expired | Đăng nhập lại để tiếp tục | hide sensitive panes, preserve unsent draft, recheck current permissions |
| Offline | Ngoại tuyến · chưa đồng bộ | supported profile only; no server authority from a file |
| OutcomeUnknown | Chưa rõ kết quả gửi | reconcile; no false success or blind send-again |
| Unsupported | Chưa được hỗ trợ | disabled control with reason/support gate; never a fake enabled provider |
| Simulation | Mô phỏng · không gửi thật | zero provider/network effects; distinguish fixtures from real tests |

## Minimum targets

- 48×48 CSS px/dp touch target; meaningful icon labels, no hover-only actions. Visual chip can be shorter only when its containing trigger still has the full target.
- 3px keyboard focus ring; focused-cell state different from selected-row state. Never remove browser outline without a replacement.
- Normal text contrast >=4.5:1, large text >=3:1, meaningful non-text controls >=3:1 against adjacent surface where needed. Audit paired semantic role values, not isolated colors.
- Tabular numerals, currency units, wrap long labels without ellipsis hiding decisive information. IDs only in Advanced or diagnostics.
- Use words/icon shape for status, not only hue. Denied/Unknown/Offline each have distinct copy.
- Responsive support: 390×844 compact viewport, 1440×1000 desktop reference, 200% zoom, text scale 1.3 and 2.0, VI/EN long labels. Full-page PNG height is not a fixed device viewport spec.
- Logical reading order: header/nav/main/inspector; h1 once per surface; explicit field labels and support/error association. Dialog title/description, focus trap, Escape and focus restoration.
- Live regions only for useful status/results, not every timer/keystroke. No repeated screen-reader announcements on each render.
- Reduced-motion, high-contrast/forced-colors, no animation-dependent meaning. Respect user setting rather than assuming accessibility prefs from OS theme.
- Mobile forms respect keyboard insets and safe area; primary action cannot cover focused field or unsent text.

## Interrupted/repeated flow tests required

1. Create draft → edit → Back/Close/reopen; preserve unsent values or ask before discard.
2. Reference picker → search → no result → cancel → reopen; no hidden record leakage, no selection drift.
3. IME input → sort/page/blur/Escape; composition neither double-commits nor targets a new record.
4. Import ambiguous Ref → resolve one → invalid second → cancel; source mapping retained, no partial invisible write.
5. Policy preview actor switch → deny → advanced details; no stale previous actor result shown as current.
6. Preview → simulated connector step → stop/restart; zero real external delivery and fixture label always visible.
7. Publish → missing gate → cancel; active release unchanged. Upgrade → backfill failure; old release still active.
8. Runtime form offline → Save Draft → reconnect → revoked grant; revalidation before any server write.
9. Approval click twice → same immutable intent; one original receipt. Newer revision after review → conflict and renewed review.
10. Function worker timeout/lost ACK → Unknown → reconcile; do not auto-label Sent or duplicate external delivery.
11. Native Android Back/iOS edge swipe and browser Back/Forward; origin and local draft retained; closing modal does not create command.
12. Theme/locale/text-scale switch on same fixture; statuses/focus/action meaning remains consistent.

## Evidence labels

- `static-svg-raster`: original layered SVG exported to PNG by Inkscape; visual anatomy/contrast/layout review only.
- `fixture-prototype`: HTML/CSS/MJS semantic reference. Node syntax/static checks do not establish browser behavior.
- `browser-interaction`: future evidence captured from mounted Leptos consumer with exact commit/build/scenario.
- `native-cmp`: future semantics/native interaction/screenshot evidence from each target. Mobile Web screenshot is not native CMP proof.
- `provider-simulated` vs `provider-real`: explicit, separate evidence. No real provider/authentication was used in this design task.

Current browser/local-host routes were previously blocked by the environment; no attempt to bypass that restriction was made and no browser/native pass is claimed.

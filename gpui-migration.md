# GPUI migration plan

Temporary working doc for the `fogde/gpui` worktree. It replaces the Tauri + React desktop app with a native Rust app on Zed's GPUI through Longbridge's GPUI Kit. `main` keeps shipping the React app while this worktree builds the replacement; delete this doc when the worktree merges. Researched 2026-10-26; versions and issue numbers are from that day.

## Ground rules

- Every crate under `crates/` stays. The host still runs in-process and speaks JSON-RPC over WebSocket, so local and server machines keep one code path.
- No GPUIX, no webview, and no GPL code: Zed's `terminal`, `terminal_view`, `ui` and `auto_update` crates and Paneflow are off limits. Reading Zed's GPL code for technique is fine; copying it isn't.
- Phase 0 deletes the React app, the Tauri shell and the Node toolchain. The GPUI app replaces them in place at `apps/desktop`, as package and binary `fogde`.
- Port from `main`, which keeps the old app: `git show main:apps/desktop/src/<path>` for any source or test.
- Every phase ends with `just check` green. Nothing is committed until Pär asks.
- Tests follow AGENTS.md: port the existing `*.test.ts` suites for risky logic to Rust, and test terminal parsing with real captured Herdr frames.
- Never start the app or run visual checks; Pär runs each gate.
- Keep the updater identity for existing installs: product name `Fogde`, identifier `dev.fogde.app`, binary `fogde`, deb package `fogde`, and a version that keeps increasing.

## Decisions (Pär, 2026-10-26)

| # | Decision | Choice | Why |
|---|---|---|---|
| D1 | GPUI Kit pin | `gpui-kit = "=0.8.1"`, which pins `gpui-pre =0.3.10` on wgpu 30.0.1. No fork or patch of wgpu: live with the Vulkan panic until a wgpu release and a GPUI Kit pin carry the fix, then upgrade | It's the latest stable. The Vulkan surface-drop panic (wgpu #9277, reported against 29, so `=0.8.0` has it too) crashes a Linux window after a failed present, such as a lost GPU device after a driver reset or resume. The fix (wgpu #10230) touches only `wgpu-hal/src/vulkan/swapchain/native.rs` and is only on wgpu's main branch. Windows (Direct3D 11) and macOS (Metal) don't use wgpu |
| D2 | Notes editor | First cut: Kit `Editor` in source mode with styled spans (no hidden markup, uniform rows). Live preview after that, as its own phase on Kit's `editor_host` (pre-1.0) or a custom element | Kit's editor decorations are paint-only: no hidden characters, larger headings or widgets |
| D3 | Tiling | Our own split-tree layout (port `tiles.ts`, `layouts.ts` and `drag.ts` with their tests), drawn with resizable splits. No tabs, ever; not Kit's `DockArea` | `DockArea` always turns a centre drop into tabs (fixed 0.35/0.65 thresholds), while Fogde swaps tiles and spans outer edges |
| D4 | Updater | `cargo-packager-updater` 0.2.3, or port Tauri's MIT install code onto `minisign-verify` if its gaps bite. Our code refuses any feed version that isn't newer than the running one | It's the standard non-Tauri updater, from CrabNebula (Tauri's maintainers): minisign-signed artifacts checked against a pinned public key, served over HTTPS from GitHub Releases. That is the same level of protection as Fogde's Tauri updater today. What it lacks is a signature bound to the version, so a tampered feed could offer an older signed build; the refusal above closes that. `gpui-updater-pre` binds versions but has one maintainer and an unverified NSIS install |
| D5 | Attention | The chime as the same two tones (880 and 1320 Hz, 0.3 s) through `rodio`. Notifications through `notify-rust`, not GPUI's new notification API | Well-known crates: notify-rust is what Tauri and GPUI's own Linux backend use, and rodio is the standard Rust audio crate. GPUI's API is new and has a Windows panic (Zed #65799). Cost: an ALSA dev package on Linux |
| D6 | Window material | macOS: `Blurred` (NSVisualEffectView) instead of Liquid Glass. Windows: `Blurred` (the acrylic accent) on the same builds as today. Linux: opaque | GPUI has no Liquid Glass or Acrylic variant. Mica is the fallback if the accent misbehaves (Zed #65402) |

Pär approved running Herdr to capture frames for test fixtures.

## Target architecture

```
apps/desktop           package and binary fogde
  main.rs              Tokio runtime, host::start, gpui_kit::application().run
  runtime.rs           Tokio handle as a GPUI Global, spawn → GPUI Task (gpui_tokio pattern, Apache-2.0)
  app.rs               root view: onboarding state or the workspace screen
  state/               per-machine entities replacing React Query: machines, workspaces, notes, usage
  window/              title bar per OS, material, window bounds, menus, keymap
  theme/               Kit ThemeSet light/dark, Fogde tokens Global, fonts, icons AssetSource
  workspaces/  terminals/  notes/  onboarding/  settings/  usage/  palette/
crates/client          typed JSON-RPC WebSocket client: calls, events, keepalive, reconnect
crates/terminal        screen model: frames → alacritty grid, grapheme fixes, key encoding, paste
crates/protocol        serde in both directions; no ts-rs
crates/host …          unchanged, apart from dropping the TS export
```

- **Async:** host, sqlx, processes and the WebSocket client stay on Tokio. GPUI entities get data through bounded `tokio::sync::mpsc` channels drained by a `cx.spawn` loop kept as a `Task` on the entity, with one `cx.notify()` per batch. Nothing on GPUI's executor awaits Tokio IO.
- **Client-side persistence:** localStorage and the Tauri window-state plugin are replaced by one serde file in `~/Fogde/.app/`. It holds layouts per view, collapsed sidebar items, the notes sort order, recent notes, window bounds, and the material (so the first paint is right).
- **The 300-line file limit** applies to all new Rust files.

## Dependencies

| Need | Crate | Licence | Notes |
|---|---|---|---|
| UI | `gpui-kit =0.8.1` (+ `test-support` in dev) | Apache-2.0 | Import GPUI only through `gpui_kit::*`; leave `gpui-fast` off |
| Terminal grid | `alacritty_terminal` 0.26.0 + `vte` 0.15 | Apache-2.0 | `scrolling_history: 0`, since the host owns scrollback; no-op sync timeout |
| Graphemes | `unicode-segmentation` 1.13, `unicode-width` | MIT/Apache | Fix layer matching Herdr's widths |
| WebSocket | `tokio-tungstenite` 0.30 | MIT | Already in the workspace |
| Folder picker | `rfd` 0.17 | MIT | GPUI's picker can't set a starting folder, and fails on Linux without a portal |
| Notifications | `notify-rust` 4.18 (`wait_for_response` for clicks) | MIT/Apache | D5. Windows click handling counts only while the toast is on screen, as today |
| Chime | `rodio` | MIT/Apache | D5 |
| Packaging | `cargo-packager` 0.11.8 (git rev for the AppImage EGL fix #493) | MIT/Apache | NSIS, `.app`/DMG (lipo universal ourselves), AppImage, deb |
| Updater | `cargo-packager-updater` 0.2.3 | MIT/Apache | D4 |
| Windows resources | `winresource` 0.1 | MIT | Icon and VERSIONINFO only; GPUI already embeds the manifest |
| Fonts and icons | Manrope, JetBrains Mono (OFL), Phosphor SVGs (MIT) | — | Register fonts before the first window; ~42 icons plus brand SVGs |

cargo-deny will see Unicode-3.0 and MPL-2.0, both allowed without asking. Confirm the full graph has no new strong copyleft before Phase 1 ends.

## Phases

Ordered by risk: prove the terminal and the async bridge before porting the many screens. Each phase lists what to build, the tests to port, and when it's done.

### Phase 0: shared groundwork (no UI)

- [x] Clear out the old app:
  - Delete `apps/desktop` (React, Tauri shell, capabilities, Tauri configs), and drop the Tauri crates from `[workspace.dependencies]`.
  - Delete `rpc/export.rs`, `apps/desktop/src/generated`, and ts-rs from `protocol` and `host`.
  - `justfile`: drop every npm step. Rewrite `lines` without Node. `dev` and `build` wait until there is an app, and packaging comes in Phase 8.
  - `ci.yml`: drop Node, the webkit2gtk packages and the generated-diff step; add GPUI's Linux packages.
  - README prerequisites: drop Node and Tauri, add GPUI's per-OS build needs.
  - Keep `release.yml` untouched until Phase 8; it only runs on `main`.
- [x] Ask Pär which `apps/desktop/AGENTS.md` rules carry over to GPUI, and rewrite that file before any UI code. Root AGENTS.md still points to it.
- [x] `protocol`: derive `Serialize` and `Deserialize` on every params, result and event type, and widen the `Method`/`Event` bounds.
- [x] `crates/client`: typed `call::<M>()` and `listen::<E>()`.
  - Lazy connect with a 30 s open timeout, plus a generation counter so stale sockets drop.
  - `ConnectionError`/`RpcError` with the `offline` retry rule.
  - Server clients via `server.connect` then `servers/{id}` on the own host.
  - Keepalive: `health.ping` every 15 s with a 10 s deadline that extends while the send queue drains. Backoff `min(30 s, 1 s·2ⁿ)` with 50–100 % jitter, reset after success.
  - Port `client.test.ts` and `keepalive.test.ts` against a real local WebSocket server.
- [x] `crates/terminal`:
  - `Screen` wraps `Term`. Applying a frame means base64 decode, resize to the frame's cols/rows, `ESC c` + `?7l` on full frames, feed the processor, and collect damage.
  - Grapheme fix layer, so a Herdr cell stays one cell for 👨‍👩‍👧, 👍🏽, ❤️, 🇸🇪, CJK and combining marks.
  - xterm default key encoder: arrows, Home/End, PgUp/PgDn, Delete, F-keys, `CSI 1;m X` modifier forms, Ctrl+letter, Alt as ESC prefix, Backspace `\x7f`, Shift+Tab.
  - Port `paste.ts` (bracket and `pastePath`) and the `frames.test.ts` cases.
- [ ] Capture real Herdr frames as test fixtures (a Claude Code TUI, Codex, emoji and CJK output, box drawing, full and delta frames). Skipped in Phase 0 by Pär: the terminal tests use frames built to Herdr v0.9.5's encoder format. Any capture sets `HERDR_SOCKET_PATH` on every `herdr` CLI call; without it the CLI reaches the installed Fogde's Herdr.

Done when the three crates' tests pass on Windows, macOS and Linux in CI.

### Phase 1: spike, and the go/no-go gate

- [x] `apps/desktop` crate:
  - Tokio runtime and GPUI bridge; `host::start` in-process; the own client connected.
  - Fonts registered; a minimal light/dark theme.
  - A plain list of machines → workspaces → terminals. Clicking a terminal opens one tile.
- [x] Terminal element:
  - Measure the cell (`em_advance`, rounded line height) and fit cols/rows. `terminals.watch`/`resize`, debounced 150 ms, sent only on change.
  - Background runs, then text runs (`shape_line` with `force_width`; `calt`, `liga` and `dlig` off; a wide character ends its run), then box and block characters as quads.
  - Cursor shapes, with an outline when unfocused.
  - Render via `Entity::cached`, notifying once per frame batch.
- [x] Input: key encoder → `terminals.input`, Shift+Enter → `terminals.key`, an IME `EntityInputHandler` with preedit at the cursor.
- [x] Mouse: selection by click count with copy on select, and wheel scrolling to `terminals.scroll`.
- [x] `just check` covers the new crate (clippy pedantic, workspace lints, deny, shear, typos).

**Gate 1 (Pär runs it on Windows, macOS and Linux; Linux on Omarchy, Arch with Hyprland on Wayland, on real hardware):**
- Claude Code and Codex look the same as in the installed Fogde.
- Typing latency, plus CPU and RAM with several tiles open.
- Emoji, CJK IME, and HiDPI or fractional scaling.

Stop or continue based on this.

### Phase 2: app shell and design system

- [x] Window:
  - Default 1200×800, minimum 720×480, centred.
  - Title bar: macOS transparent with traffic lights at the current inset; Windows custom minimize/maximize/close with a red close hover; Linux native decorations, with the bar shown only for the sidebar toggle.
  - Material switch (D6); bounds saved and restored.
- [x] Theme:
  - Port `styles/theme.css` into a Kit ThemeSet for light and dark.
  - Fogde-only tokens live in a Global: `surface-1…8`, overlay, tint, hover, selected, shadows, terminal ANSI and scrollbar colours, success/warning/caution.
  - Type roles display/title/body/caption/code; system light/dark following.
- [x] Icons: an `AssetSource` with the Phosphor subset and brand SVGs, falling back to Kit's assets for `IconName` paths.
- [x] Shared pieces:
  - Small buttons; error toasts that never time out (`report`); warning toasts.
  - Dialog, alert dialog, menu, context menu, tooltip only on icon buttons, a field with an error under it (Kit has no validation state).
  - Spinner loading state (no skeletons).
- [x] State:
  - Machine entities with connection state, retry on window focus and on network return, and reconnect refresh.
  - Event fan-out per machine, with no events crossing machines.
- [x] Persistence file, status bar, empty sidebar frame.

### Phase 3: workspaces, tiles, palette, shortcuts

- [x] Tile layout model in Rust: tile kinds terminal/start/notes/hold, one tile per group, centre swap through hold, group edge move, outer edge spanning a side, `sync` with optimistic `workspaces.place`, `dropTile` on unseen layouts, `keepMachines`, `forget`, and following note renames.
  - Port `tiles.test.ts`, `note-drop.test.ts` and `selection.test.ts`.
  - Unit-test the keyboard move neighbour geometry and the tile numbering.
- [x] Split renderer with resizable dividers.
- [x] Drag and drop: typed GPUI drag payloads (tile, terminal row, note), the `canDrop` rules, and the trash target.
- [x] Sidebar tree:
  - Machine groups: context menu, and reconnect/spinner/new workspace on the right.
  - Workspaces: collapsible with persisted state, a status dot when collapsed, muted when the folder is missing, menus.
  - Rows: terminals sorted by attention, notes, start, worktrees.
  - The whole path to the focused tile is highlighted. Below 768 px the sidebar becomes a sheet.
- [x] Start page and New tile; new workspace (`rfd` locally, `FolderDialog` remotely); new and remove worktree; delete-workspace and busy-terminal-close confirmations.
- [x] Tile bar: drag handle, place label, branch, `BranchStats` with tooltips, close.
- [x] Command palette (Kit `Command`) with the current groups and order, running the chosen action after it closes.
- [x] Keymap:
  - Actions bound with `secondary` on macOS and Ctrl+Shift elsewhere: K, G, W, T, Alt+arrows, 1–9; Ctrl/Cmd+B for the sidebar.
  - Contexts so dialogs block everything except the palette.
  - The macOS menu via `set_menus`: Close Tile ⌘W, Close Window ⌘⇧W, Edit, View, Window.
- [x] Workspace cache rules: port `workspaces.test.ts` (no undo by stale lists) and the `use-viewing` behaviour (`terminals.view`).

### Phase 4: terminal complete

- [x] Watch and unwatch serialised per client, re-watch on reconnect, unwatch only when attached, connected and releasing.
- [x] Status overlays (opening, ended, failed with Reopen), the "The terminal stopped" toast, and workspace refresh on ready/ended/failed.
- [x] Host-owned scrollback: wheel pixel accumulation with the line and page modes, then `terminals.scroll` with the cell under the pointer. A custom scrollbar driven by `terminals.scrolled`, with a thumb drag sending `terminals.seek` once per frame. Port `scrollbar.test.ts`.
- [x] Links: OSC 8 and plain URLs (`RegexSearch`), opening only `http(s)` on Ctrl/Cmd+click.
- [x] Clipboard:
  - The "Copied" notice for 1.5 s; Linux primary selection.
  - Non-macOS Ctrl+C with a selection copies; Ctrl+V pastes.
  - Text paste is bracketed. Images (png/jpeg/gif/webp, up to 10 MiB) go through `terminals.pasteImage` in order. File drops take the same path, with a toast for non-image files.
- [x] Theme refresh, input enabled only while watching, focus when the tile activates.
- [x] Accessibility: `Role::Terminal` with one text node per row, the cursor as the text selection, and a polite live region. Only when `is_a11y_active()` or the screen-reader setting is on.

### Phase 5: onboarding, machines, settings

- [x] Onboarding as an app state enum replacing the router: the `_onboarded` guard rules, step order (server skipped for this computer), step bars with screen-reader text, toasts closed between steps.
- [x] Steps:
  - **Where.**
  - **Server:** SSH host chips, check → untrusted/keyRejected/ready, trust and key help, setUp.
  - **GitHub:** github.com and Enterprise, device code with copy, trusted-URL check. Port `sign-in.test.ts`.
  - **Agents:** install, then sign in with a done/browser/code/paste prompt, plus cancel.
  - **Repos folder:** async check, browse.
  - **Clone:** list per account, filter, per-repo status, finish.
- [x] `FolderDialog` for remote folders: list, up, editable path, new folder, delete only empty folders. Port `folders.test.ts`.
- [x] Settings dialog: Appearance, Tile bar, Status bar, Terminal shell, Notes folder per machine, Alerts, Accessibility, and Machines (rename, connect/disconnect, remove, add). Optimistic saves with rollback.

### Phase 6: notes

- [x] Notes view: a hideable tree pane with the six sort orders, ancestors of the open note auto-expanded, a context menu (new, copy path, rename, delete to `.trash`), and the empty and error states.
- [x] Logic ports with their tests: `tree.test.ts`, `links.test.ts`, `merge.test.ts`, `session.test.ts`, `note-renames.test.ts`, and wikilink parsing from `editor.test.ts`.
  - Session: save after 600 ms, flush on blur and close, version-checked writes, three-way merge of agent edits, conflicts keep the user's text with a toast, a note that was only opened is never written, and CRLF is preserved.
- [x] Recent notes (20 kept, 8 shown) and palette note search ranking.
- [x] Editor per D2:
  - **First cut:** Kit `Editor` with Markdown highlighting, styled headings, emphasis, code, links and wikilinks; Mod-B and Mod-I; Tab indent; Ctrl/Cmd-click to follow links; a 48rem centred column.
  - **Then live preview:** markup hidden off the active lines, bullets, task checkboxes, rules, link text only.
- [x] Note drag: onto a terminal pastes the quoted path; onto notes or start tiles shows the note; onto an edge opens a new tile; same machine only.

### Phase 7: attention, notifications, usage

- [x] Attention: `terminals.state` patches the workspace entity; a change into blocked or done, unless that tile is visible and focused, plays the chime (D5), shows a system notification whose click reveals the terminal, and requests window attention. Check that GPUI has a request-attention call; otherwise use a small per-platform helper.
- [x] `terminals.failed` and `usage.changed` handling.
- [x] Usage meters: the newest reading across machines, session window preferred, floored % left, reset time, tooltip sentences, refreshed every 30 s.

### Phase 8: packaging, updates, CI, cutover

- [x] Embed both `fogde-host` musl binaries with `include_bytes!`, removing the per-OS resource lookup.
- [x] cargo-packager config:
  - NSIS per-user, matching Tauri's install folder and registry keys.
  - `.app` and DMG from a lipo'd universal binary, ad-hoc signed.
  - AppImage and deb.
  - Icons generated from the existing SVGs, replacing `tauri icon`.
- [x] Updater (D4): hourly check in production builds, download in the background, "Update ready" button, install and restart.
- [x] `release.yml`:
  - `cargo build` plus `cargo packager` per OS, signing with the existing key under the new variable names, uploading with `gh release upload`.
  - `latest.json` serves both apps: Tauri's `darwin-*` and cargo-packager's `macos-*` keys, `format` on every entry, plus `linux-x86_64-deb`.
- [x] `ci.yml`: GPUI's Linux packages instead of webkit2gtk; no Node; headless GPUI tests on all three OSes.
- [ ] Pär tests a passive upgrade from an installed Tauri build on Windows, macOS and Linux (AppImage and deb) before the first release.
- [x] `justfile` `build` and `icons` on cargo-packager; README development section.
- [ ] Before merging:
  - Update the Fogde notes (the Desktop UI note's Tauri facts become GPUI facts).
  - Delete this doc.

**Gate 2:** feature parity with the shipped app on all three operating systems (Pär's daily use plus a pass on Omarchy), then merge into `main`.

## Risks

| Risk | Mitigation |
|---|---|
| Weekly `gpui-pre` snapshots break APIs even in patch releases (0.8.1: `KeyBinding::new`, `Modifiers::secondary`, `ScrollWheel`) | Exact pins and a committed lock file; upgrade on purpose, one bump at a time |
| One publisher (`huacnlee`) for every snapshot and Kit crate | If publishing stalls, run Kit's `script/bump-gpui.ts` ourselves or fork |
| Terminal fidelity: alacritty splits graphemes that Herdr keeps whole | Fix layer plus captured-frame tests; fall back to libghostty-vt (needs Zig in CI) or a small vte grid matching Herdr's subset |
| Accessibility is young: Zed ships AccessKit off, and duplicate nodes can panic (Kit #3295) | Keep it on, test with Narrator and VoiceOver at the gates, and watch for the panics |
| Linux GPU: no GL fallback, so machines without a working Vulkan driver (VMs, older Intel GPUs) render through llvmpipe; the Vulkan surface panic; X11 window-bounds bugs (Zed #64447, #64448) | The panic stays until an upstream release (D1); check each GPUI Kit bump for it. Every gate runs on Omarchy (Hyprland, Wayland). X11 is covered only by CI's headless tests on Ubuntu. Hyprland draws no window decorations, so check that the client-side title bar works there. State the Vulkan driver requirement in the README. Work around X11 bounds by restoring only sizes that fit the current displays |
| Windows: no presenting under sustained typing (Zed #61469), phantom Alt (#62404), notification RefCell panic (#65799) | Watch them at Gate 1; `notify-rust` fallback for notifications |
| Live-preview editor size | D2 phases it after the source-mode editor |
| Updater migration from Tauri installs is untested | Passive-upgrade test in Phase 8 before shipping |
| Agents are less fluent in GPUI than React | Read Kit's `examples/ai_recipes`, `skills` and the reference apps below before each phase |

## Read before building

- The old app on `main` (`git show main:<path>`), and `gpui-inventory.md`, which lists all 72 RPC methods, 8 events and each feature's rules.

- GPUI Kit (Apache-2.0):
  - `crates/kit/TESTING.md`
  - `website/docs` (task, window, keybinding, accessibility, system-notification, packaging, auto-update)
  - `examples/{dock,editor,markdown,stream-markdown,ai_recipes}`
  - `crates/story`
  - `crates/base/src/text/editor_host` for live preview
  - Clone `longbridge/gpui-kit` at tag `v0.8.1` into the session's scratchpad to read it.
- [penso/herdr-gpui](https://github.com/penso/herdr-gpui) (Apache-2.0): a native Herdr client with Tokio. The closest app to Fogde.
- Terminals:
  - [crabtalk/bezel](https://github.com/crabtalk/bezel) `bezel-terminal` (MIT): grid rendering, glyph quads, the key table.
  - [zortax/gpui-terminal](https://github.com/zortax/gpui-terminal) (MIT OR Apache-2.0).
- Zed `crates/gpui_tokio` (Apache-2.0): vendor it with its notice, or write the equivalent.
- Apps on the same Kit:
  - vicanso/zedis (Apache-2.0): updater, multiple windows.
  - fulgur-app/Fulgur (Apache-2.0).
  - manyougz/velotype (Apache-2.0): Markdown editor.
  - wabzqem/gpui-kit-markdown-editor (Apache-2.0): `editor_host`.
- Herdr's frame encoder, `src/protocol/render_ansi.rs` at v0.9.5: the exact VT subset the terminal must handle.
- Tauri updater plugin 2.14.0 and the NSIS template: what installed apps expect from an update.

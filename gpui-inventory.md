# React app behaviour inventory

Companion to `gpui-migration.md`: what the React frontend on `main` does, captured 2026-10-26 so the GPUI port matches it. Paths are under `apps/desktop/` on `main` (read them with `git show main:<path>`). Delete this file with the plan.

## Transport and machines

**Client** (`src/lib/client.ts`)
- JSON-RPC 2.0 over a WebSocket at `${url}?token=${token}`.
- Connects lazily on the first call, with a 30 s open timeout.
- A generation counter drops stale sockets; pending calls reject when the socket closes.
- Errors are `ConnectionError` or `RpcError(code)`. A failure is retryable unless it's an `RpcError` whose code isn't `offline` (-32001).

**Clients per machine** (`src/lib/host.ts`, `src/lib/machines.ts`)
- The own host's address comes from the Tauri `connection` command and is called through `callOwn`.
- Server clients call own `server.connect(id)`, then open `${url}servers/${id}` on the same host.
- The local machine id is `"local"`. Only connected machines get a sidebar group, events and usage.
- Disconnecting or removing a machine resets its client and drops every query under its id. Query keys are prefixed with the machine id.
- UI labels say "on {machine}" when there is more than one machine.

**Keepalive** (`src/lib/keepalive.ts`)
- `health.ping` every 15 s with a 10 s deadline. The deadline extends while the send buffer is draining (large uploads).
- A missed ping marks the client lost and retryable.
- Reconnect backoff is `min(30 s, 1 s·2ⁿ)` with 50–100 % jitter, reset after a successful connection.

**Reconnects**
- Retry on return (`hooks/use-retry-on-return.ts`): the `online` event or window focus reconnects every retryable disconnected machine.
- When a machine connects, its workspaces reload.

## RPC surface

**Methods used only on the own host**
- `accessibility.get/set`
- `machines.list/addComputer/connect/disconnect/remove/rename`
- `notifications.get/set`
- `onboarding.done/finish`
- `server.hosts/check/setUp/connect`
- `shell.get/list/set` (global)
- `stats.get/set`
- `theme.get/set/getMaterial/setMaterial`
- `usage.getShown/setShown`

**Methods used per machine**
- `agents.list/install/startSignIn/finishSignIn/cancelSignIn`
- `folders.list/create/remove`
- `github.accounts/startSignIn/finishSignIn/cancelSignIn`
- `notes.list/read/write/rename/delete/folder/setFolder/createNote/createFolder`
- `repos.folder/setFolder/checkFolder/list/clone`
- `terminals.create/close/watch/unwatch/resize/input/key/pasteImage/scroll/seek/view`
- `usage.list`
- `workspaces.list/create/delete/place`
- `worktrees.create/remove`
- `health.ping`

**Events**

| Event | Effect |
|---|---|
| `terminals.frame` | Draw the frame |
| `terminals.scrolled` | Update the scrollbar |
| `terminals.ended` | Mark the terminal ended |
| `terminals.failed` | Toast "Could not start {agent}" and reload workspaces |
| `terminals.state` | Patch status; drives attention and notifications |
| `workspaces.changed` | Reload workspaces |
| `usage.changed` | Replace the usage cache |
| `notes.changed` | Reload `notes.list` and `notes.read` |

## Routing and onboarding

- **Guard** (`_onboarded.tsx`):
  - Not onboarded, with exactly one machine: go to `/onboarding/github?machine=`.
  - Not onboarded otherwise, or with no machines: go to `/onboarding/where`.
- **Onboarded layout:** sidebar, inset content on `surface-3`, status bar (Settings on the left, usage meters on the right) and the command palette.
- **Step order** lives in `src/lib/onboarding.ts`; the server step is skipped for this computer.
- **Onboarding layout:** step bars with "Step X of Y" for screen readers. Toasts close on every step change.
- **Where:** "This computer" calls `machines.addComputer`; "A server" goes to the server step. When already onboarded, the screen is titled "Add a machine" and has a Back button.
- **Server step:**
  - SSH target input with chips from `server.hosts`.
  - `server.check {target, trust}` returns:
    - `untrusted`: show the fingerprint and "Trust this server", then re-check with the trust value.
    - `keyRejected`: show the keys ssh offered and "Try again".
    - `ready`: call `server.setUp`, which returns the machine id, then go to the GitHub step.
  - Busy labels are "Connecting…" and "Setting up…".
- **GitHub step:**
  - github.com, plus GitHub Enterprise through a host form.
  - Sign-in is `github.startSignIn`, then the trusted-URL check (`lib/sign-in.ts`), then `openUrl`, then `github.finishSignIn`.
  - The device code shows with a copy button and "Waiting for approval".
  - Continue needs at least one account.
- **Agents step:**
  - Install when missing, otherwise Connect.
  - `agents.startSignIn` returns a prompt of kind `done`, `browser`, `code` (device code with a hint) or `paste` (an input). Link domains are checked against `signInDomains` in `lib/agents.ts`.
  - Then `agents.finishSignIn {agent, code}`; Cancel calls `agents.cancelSignIn`.
  - Continue needs one agent signed in.
- **Repos folder:** validated asynchronously with `repos.checkFolder` and saved with `repos.setFolder`. Browse uses the native picker locally and `FolderDialog` on servers.
- **Clone:**
  - `repos.list(host)` per account, with a filter.
  - Ticking a checkbox runs `repos.clone` and shows Cloning or Cloned.
  - Continue calls `onboarding.finish` and goes to `/`.
- **Sign-in safety:** a sign-in URL outside the trusted domains cancels the sign-in with `UntrustedLink`.

## Sidebar and workspaces

- **Search button** opens the palette and shows its key hint.
- **Machine group:**
  - Desktop or HardDrives icon with the machine name.
  - Context menu: "New workspace".
  - On the right: Reconnect or Retry when down (the tooltip shows the error), a spinner while connecting, otherwise "New workspace" on hover.
- **Workspace item:**
  - Collapsible, with the collapsed state persisted. A status dot shows when collapsed. The name is muted when the folder is missing.
  - Click selects it, or toggles it when it's already current.
  - Context and "…" menus: New tile, New worktree (repos only), Delete.
- **Rows under a workspace:**
  - Terminal: sorted by attention, draggable, menu Close.
  - Note tile: draggable, Close.
  - Start ("New tile"): Close.
  - Worktree: branch or folder name, New tile, Remove, with its own collapsible rows.
- **Layout:** the footer holds the update button. Below 768 px the sidebar becomes a sheet.
- **Status dots:**

| Status | Colour | Label |
|---|---|---|
| working | warning | "Working" |
| blocked | destructive | "Needs your input" |
| done | success | "Finished" |
| idle | no dot | — |

  "Needs you" means blocked or done; "busy" means working or blocked.
- **Start page:**
  - No workspace: "Create a workspace".
  - Missing folder: "Folder not found" with Delete workspace.
  - Otherwise: "Continue working in {place} on {machine}" with the path, a Terminal button, one button per signed-in agent (`terminals.create {workspace, folder, agent}`), Notes, an inline New worktree form (`worktrees.create {repo, name}`) or Remove worktree.
  - Shown only while connected.
- **Dialogs:**
  - New workspace: native directory picker locally, `FolderDialog` on servers, then `workspaces.create`.
  - Delete workspace: terminals close and files are untouched.
  - Closing a busy terminal asks for confirmation.
- **Tile bar:**
  - Drag handle, icon, title, status dot.
  - Place label (folder relative to the checkout, full path in a tooltip) and branch.
  - Branch stats `↓N base`, `+added −removed`, `↑ahead ↓behind`, each switched by settings, with sentence tooltips.
  - Close button.
- **Cache rules:**
  - `workspaces.place` is optimistic.
  - `worktrees.remove` is refused while its terminals are open.
  - Close and delete forget the saved layout.
  - `addTerminal` and `removeTerminal` patch the cache, then invalidate; a stale list arriving later must not undo them.
- **Viewing** (`use-viewing.ts`): send `terminals.view(id | null)` to each machine. The id is sent only while the window is focused, and re-sent on reconnect.
- **`FolderDialog`:** `folders.list(path)` returns `{path, parent, folders}`. It has Up, an editable path, New folder, and delete per row (only empty folders).

## Tiles

| Kind | Id | Params |
|---|---|---|
| `terminal` | `${machine}:${id}` | `{machine, id}` |
| `start` | `start` | `{folder}` |
| `notes` | `notes:${uuid}` | `{machine, note?}` |
| `hold` | `hold` | Invisible; temporary swap placeholder |

**Layout rules**
- No floating groups. Edge drop zones are 15 %.
- Group headers are hidden; each tile draws its own bar.
- Exactly one panel per group: extras move to the right.
- An empty layout shows the start page.

**Persistence**
- Saved as `fogde.tiles`: `workspace`, `layouts` per view key `machine/id[/checkout]`, and `closed` sidebar keys.
- Saved on every layout change except while loading.
- Switching workspace clears the layout and loads the saved one, removing empty groups and separating stacks.
- `keepMachines` drops layouts of unknown machines. `forget` drops a workspace's and its worktrees' layouts.
- `dropTile` edits a saved layout that isn't on screen.
- Notes params follow note and folder renames.

**Sync**
- Every terminal in the view gets a tile, added to the right and inactive if a tile is already active.
- Tiles of gone terminals close, unless that terminal is being placed.
- A pending `show(selection)` focuses its tile once it exists.

**Opening**
- `openStart` opens the start tile beside the active one, or updates it.
- After `terminals.create`, the new terminal replaces the start tile in its group.
- `openNotes` replaces the start tile or opens beside the active one.

**Drag sources**
- Tile bars (the start tile only when it shares the layout), sidebar terminal rows, note-tile rows.
- Payload is either `{machine, id}` or `{tile, view?}`.

**Drop rules (`canDrop`)**
- A terminal must belong to the current workspace's machine.
- A new terminal goes on an edge, or anywhere when the layout is empty.
- An existing tile goes on another group, or on an outer edge when there are two or more groups.

**Drop behaviour**
- Centre: swap through `hold`.
- Group edge: move.
- Outer edge: a new group spanning that side.
- A sidebar terminal not on screen: optimistic `workspaces.place {terminal, workspace, checkout}` plus placement.
- Trash (bottom centre, shown while dragging): a terminal closes, asking first if busy; other tiles close.

**Keyboard moves** need two or more groups. The target is the nearest group in that direction (gap ≥ -1 px, overlap > 0, smallest gap then largest overlap). The tile swaps with it, or moves to a new edge group when there is none.

**Tile numbers** for Mod+1…9 order groups by left, then top. `closeActive()` calls the active tile bar's close.

## Terminal tile

**Setup**
- Display only: disable stdin and keep no scrollback.
- Bar cursor, drawn as an outline when inactive. No bold-as-bright. Transparent background.
- Font comes from the `type-code` role. Colours: foreground, cursor, cursor accent `surface-1`, selection, 16 ANSI colours.
- Screen-reader mode follows `accessibility.get().screenReader` live.
- The terminal opens only after the fonts load.

**Watch**
- `terminals.watch {id, size}`, then enable input. `onData` sends `terminals.input {id, data}`.
- Input is disabled on `terminals.ended` or a disconnect.
- Watch and unwatch are serialised per client. `terminals.unwatch` is sent only when attached, connected and releasing; a disconnect stops without releasing.
- The terminal re-watches when the client reconnects.

**Frames**
- `data` is base64, written strictly in order.
- When cols/rows differ, resize to the frame. A full frame is preceded by `ESC c` then `ESC[?7l`.
- Tests cover a wide grapheme at the last column.

**Size**
- Measured cols/rows; NaN or below 1 means no size.
- Resize observation is debounced 150 ms. `terminals.resize {id, size}` is sent only while watching and when the size changed.
- A theme change re-applies the appearance and re-checks the size.

**Scroll**
- The host owns scrollback.
- Wheel pixels accumulate, with line and page modes converted by the cell height. Each whole line sends `terminals.scroll {direction, lines, column, row}`, where column and row are the cell under the pointer.
- `terminals.scrolled {offset, max, rows}` drives a custom scrollbar:
  - Thumb height `max(2rem, rows/(rows+max))`; thumb position `1 − offset/max`, so offset 0 is the bottom.
  - Dragging the thumb sends `terminals.seek {id, offset}` at most once per animation frame.
  - Hidden when `max` is 0. Colours are `terminal-scrollbar(-hover|-active)`.

**Selection and clipboard**
- Copy on select: on mouse-up with a selection, copy it and show "Copied" for 1.5 s.
- Text paste is always bracketed (CRLF becomes LF, embedded paste markers stripped).
- Image paste (png, jpeg, gif, webp, up to 10 MiB) goes as `terminals.pasteImage {id, image, data}`, queued in order. Dropped files take the same path; a non-image file toasts an error.

**Keys**
- Shift+Enter alone, when not composing, sends `terminals.key {key: "shiftEnter"}`.
- On Windows and Linux, Ctrl+V is a paste and Ctrl+C with a selection is a copy that clears the selection; anything with Alt or Meta passes to the terminal. macOS passes everything to the terminal.
- Global shortcuts still work inside the terminal; the sidebar toggle Ctrl+B doesn't.

**Links**
- OSC 8 and plain URLs open only on Ctrl/Cmd+click, and only `http(s)`. A failure toasts.

**Overlays**
- Opening: "Opening terminal…" with a spinner.
- Ended: "The terminal ended" with Reopen.
- Failed: "Could not show the terminal" with Reopen.
- Shown only while connected.
- `terminals.ended` with an error also toasts "The terminal stopped". Ready, ended and failed all reload the workspaces.

**Focus**: focusing the tile focuses the terminal. On ready, the terminal takes focus if its tile is active.

## Notes

**View**
- Tree pane 64 units wide, hideable. New note and New folder go in the open note's folder, or the root.
- Six sort orders (name, modified, created; each ascending or descending), saved as `fogde.notes`. Folders always sort by name with numeric collation.
- Ancestors of the open note auto-expand, and expanded folders follow renames.
- Context menu: New note and New folder (folders only), Copy path, Rename, Delete ("moves to the .trash folder").
- States: "Opening notes…", "Could not open the notes folder" (Try again), "No note open", "This note was moved or deleted".

**Editor**
- GFM plus wikilinks. History, line wrap, a centred column at most 48rem wide (24/32 px padding).
- Mod-B toggles `**`, Mod-I toggles `*`, Tab indents. CRLF is preserved. An empty note gets focus.

**Live preview**
- Markup is hidden except on active lines (selection lines while focused).
- Headings ATX 1–6 and Setext 1–2 map to the display, title and semibold roles. Fenced and indented code get a muted background; blockquotes get a left border and muted text.
- Emphasis, strong, strikethrough and inline code are styled with their marks hidden.
- Widgets: `•` bullets, task checkboxes that toggle `[ ]`/`[x]` on click, horizontal rules.
- `[text](url)` shows only the text. `[[target|label]]` shows only the label. Bare URLs become links.

**Wikilinks**
- Parsing skips `![[` embeds, newlines, nested `[` and empty `[[]]`.
- Resolution drops `#anchor` and `.md`, then matches case-insensitively on the whole path or a suffix. Ties go to the linking note's folder, then the shortest path, then alphabetical order.
- `[x](y.md)` resolves relative to the note, falling back to wikilink resolution. Otherwise `http(s)` and `mailto` open externally.
- Windows paths compare in lower case with either separator.

**Session** (`lib/session.ts`, `lib/merge.ts`)
- Save after a 600 ms pause; flush on blur and on close.
- `notes.write {path, text, version}` returns `saved {version}` (own versions are remembered) or `changed`, which re-reads.
- When a new version arrives:
  - Defer it while a save is in flight; ignore own versions.
  - Otherwise run a three-way merge. Each side's change is one diff range, and they merge only when the ranges don't overlap.
  - On conflict, keep the user's text with the toast "Kept your version of the note".
  - Apply external text as a minimal diff that doesn't trigger a save.
- A failed save doesn't retry until the next edit. A note that was only opened is never written.

**Renames** move cached texts and recent notes, and are broadcast so tiles follow them (prefix moves too).

**Recent notes** are kept per machine (20 kept, 8 shown) as `fogde.notes.recent`.

**Palette search**: every word must match `name folder`. Rank prefix matches, then contains, then most recently modified. At most 20 results.

**Note drops** (same machine only)

| Dropped on | Result |
|---|---|
| Terminal centre | Paste the quoted path, bracketed; quoted unless it matches `[\w./\\:~-]+` |
| Notes tile centre | Show the note there |
| Start tile centre | Notes tile in its place |
| Elsewhere | New notes tile at that position |

## Chrome, palette, settings

**Title bar**
- 40 px, draggable.
- macOS leaves room for the traffic lights. Windows draws minimize, maximize and close (red on hover). Linux uses native decorations and hides the bar unless the sidebar toggle is shown.
- The sidebar toggle icon is filled when the sidebar is open.

**Palette**
- "Go to or run…", with looping keyboard navigation.
- Groups in order:
  1. **Needs you:** "Go to the next agent that needs you", then each waiting terminal.
  2. **New in {workspace}:** New tile, Open notes, New terminal or each agent, New worktree.
  3. **{worktree}:** Remove worktree.
  4. **Recent notes**, then **Notes**.
  5. **Terminals:** all of them, "on {machine}" when there are several machines.
  6. **Workspaces.**
  7. **Tile:** Close; Move left, right, up and down; Go to tile 1–9.
- An action runs after the close animation. Focus is restored only when no action ran.

**Status bar**: 40 px, Settings on the left, usage meters on the right.

**Usage meters**
- One meter per agent switched on in `usage.getShown`, using the newest reading across machines.
- Expired windows are dropped; the session window is preferred.
- Shows the agent icon, a bar, the floored "% left" and the time until reset (session reset capped at now + 5 h).
- Tooltip reads like "5-hour limit 62% left, resets in 2h 3m (Monday 3:00 PM)". Refreshes every 30 s.

**Updates**
- Check hourly in production builds and download right away.
- The footer then shows "Update ready"; clicking it installs and relaunches.
- One toast "Could not check for updates" on failure.

**Settings**

| Section | Controls |
|---|---|
| Appearance | Theme System/Light/Dark; Translucent switch (only when the window can be translucent) |
| Tile bar | Behind, lines, upstream switches |
| Status bar | Claude, Codex, Copilot usage switches |
| Terminal | Shell select: "System default" plus `shell.list` |
| Notes folder | Per connected machine |
| Alerts | Notifications switch, Sound switch |
| Accessibility | Screen reader switch |
| Machines | Connection dot, name, target; menu Rename, Connect/Disconnect, Remove (connection controls hidden for the last connected machine); "+" opens onboarding Where |

Saves are optimistic and roll back on error.

**Toasts**: errors never time out; there are also warnings.

**Context menus**: machine group, workspace, worktree, terminal row, note-tile row, start row, note entries.

## Attention and notifications

- `terminals.state` patches the cache, and reloads when the folder changed or the terminal is unknown.
- On a change into blocked or done from a status that wasn't one of those, unless the window is focused with that tile visible:
  - Play the chime if sound is on: 880 Hz then 1320 Hz, 0.3 s.
  - Send a notification "{name}: Needs your input in / Finished in {workspace}". A click brings the window forward and reveals the terminal.
  - Request window attention.

## Shortcuts

Mod is ⌘ on macOS and Ctrl+Shift elsewhere. Shortcuts are caught before the focused element sees them.

| Keys | Action |
|---|---|
| Mod+K | Palette (also inside dialogs) |
| Mod+G | Next agent that needs you, wrapping around |
| Mod+W | Close tile |
| Mod+T | New tile, focusing its first button |
| Mod+Alt+arrows | Move tile |
| Mod+1–9 | Focus tile N |
| Ctrl/Cmd+B | Sidebar (not inside a terminal) |
| macOS menu | Close Tile ⌘W, Close Window ⌘⇧W, Edit, View (fullscreen), Window |

Every shortcut except Mod+K is ignored inside a dialog or alert dialog.

## Window and persistence

- **Window:** 1200×800, minimum 720×480, centred.
  - Windows: no decorations; translucent with Acrylic on build 22523 or later.
  - macOS: overlay title bar, translucent, Liquid Glass with a sidebar fallback.
  - Linux: opaque.
  - The material setting switches the effect on and off. The window remembers position, size and maximized state.
- **Translucent backgrounds** mix `surface-1` at the tint (30 % light, 20 % dark) with transparency.
- **Floating surfaces** (dialogs, menus, select, toasts) blur at 40 px with 150 % saturation.
- **Client-side storage:** `fogde.tiles`, `fogde.notes`, `fogde.notes.recent`, and `material` (for the first paint; the host value wins).

## Theme

**Type roles**

| Role | Size / line height | Weight |
|---|---|---|
| display | 28/34 | 700 |
| title | 14/20 | 550 |
| body | 13/20 | 400 |
| caption | 12/16 | 400 |
| code | 13/20, mono | 400 |

**Fonts:** Manrope Variable and JetBrains Mono Variable.

**Tokens**
- `surface-1…8`, `surface-overlay`, `tint`, `hover`, `selected`.
- `shadow-1…8`: light uses a 1 px ring plus drops; dark uses inset highlights.
- Terminal selection, scrollbar and 16 ANSI colours for light and dark.
- shadcn semantic colours (oklch): background, foreground, card, popover, primary, secondary, muted, accent (each with a foreground), destructive, success, warning, caution, border, input, field, ring.
- Radius 0.625rem.

**Motion:** `ease-out cubic-bezier(0.23, 1, 0.32, 1)`. Onboarding steps slide 24 px. Reduced motion is respected.

**Icons (Phosphor)**
- Check, CaretRight, CaretDown, CaretUp, Notebook, Info, Trash, PlusCircle, Plus, MagnifyingGlass, GitBranch, DotsThree, SidebarSimple, Sidebar, TerminalWindow, Square, Spinner, FolderSimple, Folder, FolderSimplePlus, FolderPlus, FileText, FilePlus, Desktop, HardDrives, ArrowUp, ArrowDown, ArrowLeft, ArrowRight, ArrowClockwise, ArrowCircleUp, XCircle, X, Warning, CheckCircle, Sun, Moon, Minus, SortAscending, GearSix, Copy, Bell.
- SidebarSimple and Trash also have filled versions.

**Brand icons:** claude-spark, plus light and dark copilot and openai-blossom.

**Names**
- Agents: Claude Code, Codex, GitHub Copilot.
- Accounts: Claude, ChatGPT, GitHub Copilot.

## Tests to port

| Test file | What it covers |
|---|---|
| `features/notes/lib/editor.test.ts` | Wikilink parsing; CRLF kept |
| `features/notes/lib/links.test.ts` | Resolution order; relative `.md`; Windows paths |
| `features/notes/lib/merge.test.ts` | Range diff; merge; conflict; CRLF |
| `features/notes/lib/session.test.ts` | Debounced save; agent changes; merge then save; ignoring its own late reload; conflict; no retry after a failed save; an opened-only note is never written |
| `features/notes/lib/tree.test.ts` | Name and time sorting; `foldersAbove` |
| `features/terminals/lib/frames.test.ts` | Wide graphemes |
| `features/terminals/lib/scrollbar.test.ts` | Thumb maths |
| `features/terminals/lib/size.test.ts` | NaN measurement sends no size |
| `features/terminals/lib/watch.test.ts` | A stalled watch doesn't block another client; cleanup order |
| `features/workspaces/api/workspaces.test.ts` | Stale lists; add and close stick; state patches; unknown terminal reloads |
| `features/workspaces/lib/note-drop.test.ts` | Note drop targets |
| `features/workspaces/lib/selection.test.ts` | `nextWaiting`; a removed worktree falls back to its workspace |
| `features/workspaces/lib/tiles.test.ts` | Swap; one tile per terminal; separate stacks; outer edge; drop rules; activation; sync; unseen layouts; renames |
| `lib/client.test.ts` | Per-machine isolation; reconnect; no crossed events; host errors aren't offline; retry on return; a rejected key isn't retried |
| `lib/folders.test.ts` | `inside` and `relative` with Windows paths |
| `lib/keepalive.test.ts` | Ping; drop and reconnect; upload drain; backoff; reset; non-retryable failures |
| `lib/note-renames.test.ts` | Prefix moves |
| `lib/paste.test.ts` | Bracketing; injection safety; path quoting |
| `lib/sign-in.test.ts` | Trusted domains and Enterprise hosts |

Logic without tests that's worth testing in Rust:
- Status aggregation and checkouts
- Shortcut matching
- Wheel maths
- Neighbour geometry for keyboard moves
- Usage time formatting
- Palette note ranking
- Recent notes
- Onboarding step order

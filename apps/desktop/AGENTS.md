# Desktop app

## Code

- Use GPUI Kit components, building a custom one only when Kit lacks it.
- Import GPUI only through `gpui_kit::*`, never `gpui-pre` directly.
- Never await Tokio IO on GPUI's executor: an entity receives data over a bounded `tokio::sync::mpsc` channel drained by a `cx.spawn` task kept on the entity, with one `cx.notify()` per batch.

## UI

- Show loading with a spinner, never a skeleton.
- Show each state in one place; never repeat it across the sidebar, main area and status bar.
- Show validation errors under their input and every other error as a toast.
- Never use a middle dot (·) as a separator; write the relation in words ("app on Hetzner VPS") or separate with layout.
- Use tooltips only for icon-only controls or information not visible elsewhere; never repeat visible text.
- Keep the window chrome transparent; raise surfaces with the `surface-*` background and shadow tokens.
- App-owned spacing uses multiples of 4 px, or 2 px; app-owned text uses only the type roles; app-owned colours use theme tokens as they are, never with changed opacity or literal values. GPUI Kit's own styles are exempt.
- Meet WCAG 2.2 AA: every action works by keyboard with a visible focus state, and icon-only controls have accessible labels.

## Motion

- Animate only a change of state or place, never a keyboard-triggered action or anything used many times a day.
- Keep every animation under 300 ms. With reduced motion, keep fades and drop movement.

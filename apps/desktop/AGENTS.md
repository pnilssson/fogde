# Frontend

## Code

- Name files in kebab-case.
- Parse data we don't own (storage, drag data, form input) with zod schemas, and keep client state shared outside React in zustand stores.
- Use installed shadcn/ui components, installing any that are missing.

## UI

- Show loading with a spinner or skeleton, whichever fits the content.
- Show each state in one place; never repeat it across the top bar, sidebar and main area.
- Show validation errors under their input and every other error as a toast.
- Never use a middle dot (·) as a separator; write the relation in words ("app on Hetzner VPS") or separate with layout.
- Keep the window chrome transparent; raise surfaces with the `surface-*` background and shadow tokens.
- App-owned spacing uses Tailwind values divisible by four, or `2`; app-owned text uses only the `type-*` roles; app-owned colours use theme tokens as they are (`bg-hover`, `text-muted-foreground`, `text-success`), never opacity modifiers like `/70` or arbitrary values. shadcn/ui and third-party styles are exempt.
- Meet WCAG 2.2 AA: every action works by keyboard with a visible focus state, and icon-only controls have accessible labels.

## Motion

- Animate only a change of state or place. Never animate keyboard-triggered actions or anything used many times a day; hover changes colour only.
- Animate only `transform` and `opacity`: ease-out to enter and leave, ease-in-out to move, never ease-in. Use the shared ease-out token and shadcn/ui's transitions as published.
- Stay under 300 ms: tooltips 125–200 ms, menus 150–250 ms, dialogs at most 300 ms, exits faster than entries. Enter from `scale-95` and `opacity-0`, never `scale-0`; popovers grow from their trigger.
- Prefer CSS transitions; use `motion/react` only for layout, presence and gestures. With reduced motion, keep fades and drop movement.

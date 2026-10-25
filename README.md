<picture>
  <source media="(prefers-color-scheme: dark)" srcset=".github/assets/banner-dark.png">
  <img alt="Fogde" src=".github/assets/banner-light.png">
</picture>

Named after the *fogde*, the bailiff who ran a district on the crown's behalf in medieval Sweden, Fogde runs your coding agents, repos, and notes on your behalf.

Fogde is a minimal desktop workspace for developers who build with coding agents. Talk to agents from one place, see the moment an agent needs you, follow your GitHub repos, and keep your notes close, on your own machine or on your own server.

## Your district

You are the crown. Fogde keeps the district in order while your agents work the fields:

- **Every agent in one hall.** Claude Code, Codex and GitHub Copilot run side by side in tiles, and each tile bar tells you which agent is busy and which one is waiting on you.
- **A plot for every task.** Give each task its own worktree with its own tiles, and see its branch and changes at a glance, whether Fogde made the worktree or you did.
- **The ledger in plain sight.** The status bar shows how much of your Claude, Codex and Copilot limits is left and when each one resets, so you always know which agent still has strength left in the day.
- **Notes on the table.** Open Markdown notes in tiles next to your agents, edit them live, and drop a note on a terminal to hand the agent its path.
- **A keep of your own.** Run the host on a server over SSH and pick up your agents from any desk you sit at.

Unlike Jösse Eriksson, the fogde whose iron grip on Dalarna set off the Engelbrekt rebellion in 1434, this one answers to you.

## Status

Early development. The bailiff was sworn in recently and is still walking the district, so expect the roads to move.

## Development

Requires [rustup](https://rustup.rs), [just](https://just.systems), [cargo-binstall](https://github.com/cargo-bins/cargo-binstall), [Zig](https://ziglang.org/download/), [Node.js](https://nodejs.org) LTS and the [Tauri system prerequisites](https://tauri.app/start/prerequisites/). The pinned Rust toolchain installs itself on first use. Install the check tools and frontend dependencies once:

```sh
cargo binstall cargo-deny cargo-shear cargo-nextest typos-cli zizmor cargo-zigbuild sqlx-cli
npm ci --prefix apps/desktop
```

Run the desktop app with `just dev`. It keeps its data in `~/Fogde-dev`, apart from an installed Fogde in `~/Fogde`.

Run `just check` before committing. It is the bailiff's inspection: formatting, Clippy, ESLint, TypeScript, tests, the file length limit, cargo-deny, cargo-shear, typos, zizmor and Knip.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Fogde by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

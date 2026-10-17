# Fogde

Named after the *fogde*, the bailiff who ran a district on the crown's behalf in medieval Sweden, Fogde runs your coding agents, repos, and notes on your behalf.

Fogde is a minimal desktop workspace for developers who build with coding agents. Talk to Claude Code and Codex from one place, see the moment an agent needs you, follow your GitHub repos, and keep your notes close, on your own machine or on your own server.

## Status

Early development.

## Development

Requires [rustup](https://rustup.rs), [just](https://just.systems), [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) and [Node.js](https://nodejs.org) LTS. The pinned Rust toolchain installs itself on first use. Install the check tools and frontend dependencies once:

```sh
cargo binstall cargo-deny cargo-shear typos-cli zizmor
npm ci --prefix apps/desktop
```

Run `just check` before committing. It runs formatting, Clippy, tests, the file length limit, cargo-deny, cargo-shear, typos, zizmor and Knip.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in Fogde by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

default:
    @just --list

fmt:
    cargo fmt --all

lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

deny:
    cargo deny check

shear:
    cargo shear

typos:
    typos

zizmor:
    zizmor .

knip:
    npm run --prefix apps/desktop knip

[script("node")]
lines:
    const { execFileSync } = require("node:child_process");
    const { existsSync, readFileSync } = require("node:fs");

    const limit = 300;
    const sources = /\.(rs|ts|tsx|js|mjs|css)$/;

    const files = execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard"], { encoding: "utf8" })
      .split("\n")
      .filter((file) => sources.test(file) && existsSync(file));

    const long = files
      .map((file) => ({ file, count: readFileSync(file, "utf8").trimEnd().split("\n").length }))
      .filter(({ count }) => count > limit);

    for (const { file, count } of long) {
      console.error(`${file}: ${count} lines, limit is ${limit}`);
    }

    if (long.length > 0) {
      process.exit(1);
    }

check: lint lines test deny shear typos zizmor knip

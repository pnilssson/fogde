set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

default:
    @just --list

dev: server
    npm run --prefix apps/desktop tauri dev

# The host for 64-bit Linux servers, which the app installs over SSH. Zig cross-compiles its C parts.
server:
    cargo zigbuild -p host --bin fogde-host --release --target x86_64-unknown-linux-musl
    cargo zigbuild -p host --bin fogde-host --release --target aarch64-unknown-linux-musl

build: server
    npm run --prefix apps/desktop tauri build -- --config src-tauri/tauri.release.conf.json

fmt:
    cargo fmt --all
    npm run --prefix apps/desktop format

lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    npm run --prefix apps/desktop format:check
    npm run --prefix apps/desktop lint
    npm run --prefix apps/desktop typecheck

test:
    cargo nextest run --workspace
    npm run --prefix apps/desktop test

# The scratch database that the SQL queries are checked against.
db := "sqlite:target/sqlx.db"

# Refreshes the checked queries in .sqlx after a query or migration changes.
sqlx:
    cargo sqlx database reset -y --source crates/host/migrations --database-url {{db}}
    cargo sqlx prepare --workspace --database-url {{db}}

sqlx-check:
    cargo sqlx database reset -y --source crates/host/migrations --database-url {{db}}
    cargo sqlx prepare --workspace --check --database-url {{db}}

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
      .filter((file) => sources.test(file) && existsSync(file) && !file.startsWith("apps/desktop/src/components/ui/"));

    const long = files
      .map((file) => ({ file, count: readFileSync(file, "utf8").trimEnd().split("\n").length }))
      .filter(({ count }) => count > limit);

    for (const { file, count } of long) {
      console.error(`${file}: ${count} lines, limit is ${limit}`);
    }

    if (long.length > 0) {
      process.exit(1);
    }

check: lint lines test sqlx-check deny shear typos zizmor knip

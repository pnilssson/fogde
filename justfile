set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

default:
    @just --list

[env("FOGDE_HOME", home_directory() / "Fogde-dev")]
dev: server
    cargo run -p fogde

# The host for 64-bit Linux servers, which the app embeds and installs over SSH. Zig cross-compiles its C parts.
server:
    cargo zigbuild -p host --bin fogde-host --release --target x86_64-unknown-linux-musl
    cargo zigbuild -p host --bin fogde-host --release --target aarch64-unknown-linux-musl

build: server
    cargo build -p fogde --release
    cargo packager -p fogde --release --out-dir target/packages

# The app icons, from a full-bleed tile and a macOS tile with the margin macOS expects around it.
icons:
    cargo run -p icons -- apps/desktop/app-icon.svg apps/desktop/app-icon-macos.svg apps/desktop/icons

fmt:
    cargo fmt --all

lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo nextest run --workspace

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

limit := "300"

[script("sh")]
[unix]
lines:
    long=$(git ls-files --cached --others --exclude-standard -- '*.rs' | while IFS= read -r file; do
      [ -f "$file" ] && awk -v file="$file" -v limit={{limit}} 'NF { last = FNR } END { if (last > limit) print file ": " last " lines, limit is " limit }' "$file"
    done)
    [ -z "$long" ] || { echo "$long" >&2; exit 1; }

[extension(".ps1")]
[script("powershell.exe", "-NoLogo", "-NoProfile", "-File")]
[windows]
lines:
    $long = git ls-files --cached --others --exclude-standard -- '*.rs' |
      Where-Object { Test-Path $_ } |
      ForEach-Object {
        $count = ((Get-Content -Raw $_).TrimEnd() -split "`n").Count
        if ($count -gt {{limit}}) { "${_}: $count lines, limit is {{limit}}" }
      }
    if ($long) {
      $long | ForEach-Object { [Console]::Error.WriteLine($_) }
      exit 1
    }

check: lint lines test sqlx-check deny shear typos zizmor

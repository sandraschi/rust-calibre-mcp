set shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

default: check test

check:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo test --workspace

fmt:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo fmt --all

fmt-check:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo fmt --all --check

run-mcp:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -q -p rust-calibre-mcp -- mcp

run-cli *ARGS:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -q -p rust-calibre-mcp -- {{ARGS}}

run-app:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -p calibre-app

smoke:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo run -p calibre-app -- --smoke

build-release:
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"; cargo build --workspace --release

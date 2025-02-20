# Basic development tasks, run with `just`. Less strict than `just ci`.
default: test reformat lint

# Run things the same way CI does
ci: test lint check-format

# Install all dependencies that are not already installed (requires `cargo-binstall`)
install-all-dependencies: install-lint-dependencies install-docs-dependencies

# Run a local webserver for testing the docs.
serve-docs:
    npm run --prefix docs start

# Build the docs, checking for broken links
build-docs:
    npm run --prefix docs build

test:
    cargo t

lint:
    cargo clippy -- -D warnings
    cargo-deny check

# Reformat all files, requires `npx` and `install-lint-dependencies`
reformat: reformat-rust reformat-toml reformat-docs

reformat-rust:
    cargo +nightly fmt

reformat-toml:
    taplo format

reformat-docs:
    npx prettier *.md --write
    npm --prefix docs run reformat

check-format: check-rust-format check-toml-format check-docs-format

check-rust-format:
    cargo +nightly fmt -- --check

check-toml-format:
    taplo format --check

check-docs-format:
    npx prettier *.md --list-different
    npm --prefix docs run check-format

# Install dependencies for `lint`, `default`, `check-format`, `reformat`, and some of `ci`. Requires `cargo-binstall`
install-lint-dependencies:
    cargo binstall --no-confirm cargo-deny taplo-cli {{binstall_args}}

install-docs-dependencies:
    npm install --prefix docs

binstall_args := ""

# Contributing

## Development

Install the latest stable Rust toolchain with rustfmt and Clippy. Create a short-lived branch from the latest `main`, such as `feat/wallet-import`, `fix/token-transfer`, or `chore/dependencies`.

Run the local checks before opening a pull request:

```sh
cargo fmt --all -- --check
cargo check --locked
cargo clippy --locked --all-targets -- -D warnings -A dead-code
cargo test --locked
cargo package --locked
```

Use `cargo fmt --all` to fix formatting. Tests belong under `tests/` and must not require live RPC endpoints or real funds.

## Code Guidelines

- Keep changes focused and consistent with the surrounding code.
- Reuse existing helpers and avoid unnecessary abstractions.
- Add tests for behavior changes and bug fixes.
- Never commit credentials, private keys, wallet CSV files, or local `.env` files.

## Pull Requests And Releases

- Explain what changed and why, including compatibility changes.
- Increase the version in `Cargo.toml` above `main`: patch for fixes, minor for compatible features, and major for breaking changes.
- Ensure CI passes, address review feedback, and resolve conversations before merging.
- Pull requests are squash-merged by the maintainer.

A successful merge to `main` publishes the crate and creates the matching Git tag and GitHub release. Publishing requires the repository's `CARGO_REGISTRY_TOKEN` secret.

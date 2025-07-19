# Contribution Guidelines

This repository contains a Rust project. Please make sure the following commands succeed before committing any changes:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Documentation updates should accompany code changes when relevant. The CI workflow mirrors these checks on GitHub.

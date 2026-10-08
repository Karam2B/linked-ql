---
name: cargo-check-and-test
description: >-
  Runs cargo check and cargo test and requires both to pass before finishing
  Rust work. Use when changing Rust code, Cargo.toml, patches, or tests in
  this repository, unless the prompt says not to run them.
---

# Cargo check and test

You have to run `cargo check` and `cargo test` and make sure they pass unless specified by the prompt.

Run both from the repository that contains `Cargo.toml`. If either fails, fix the failure and run them again. Do not finish while either command is failing.

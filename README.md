# The Emerald ParaTime

[![CI lint status][github-ci-lint-badge]][github-ci-lint-link]
[![CI audit status][github-ci-audit-badge]][github-ci-audit-link]

<!-- markdownlint-disable line-length -->
[github-ci-lint-badge]: https://github.com/oasisprotocol/emerald-paratime/workflows/ci-lint/badge.svg
[github-ci-lint-link]: https://github.com/oasisprotocol/emerald-paratime/actions?query=workflow:ci-lint+branch:main
[github-ci-audit-badge]: https://github.com/oasisprotocol/emerald-paratime/workflows/ci-audit/badge.svg
[github-ci-audit-link]: https://github.com/oasisprotocol/emerald-paratime/actions?query=workflow:ci-audit+branch:main
<!-- markdownlint-enable line-length -->

This is the Emerald ParaTime, an official EVM-compatible
[Oasis Protocol Foundation]'s ParaTime for the [Oasis Network] built using the
[Oasis SDK].

[Oasis Protocol Foundation]: https://oasisprotocol.org/
[Oasis Network]: https://docs.oasis.dev/oasis-network-primer/
[Oasis SDK]: https://github.com/oasisprotocol/oasis-sdk

## Note

* **This ParaTime currently depends on an unreleased version of [Oasis SDK].**

## Building

### Prerequisites

#### Rust

Ensure you have [Rust] and [rustup] installed on your system.
For more details, see [Oasis Core's Prerequisites] documentation, the Rust
section.

The version of the Rust toolchain we use for the Emerald ParaTime is specified
in the [rust-toolchain] file.

The rustup-installed versions of `cargo`, `rustc` and other tools will
[automatically detect this file and use the appropriate version of the Rust
toolchain][rust-toolchain-precedence] when invoked from the Emerald ParaTime git
checkout directory.

To install the appropriate version of the Rust toolchain, make sure you are
in an Emerald ParaTime git checkout directory and run:

```
rustup show
```

This will automatically install the appropriate Rust toolchain (if not
present) and output something similar to:

```
...

active toolchain
----------------

nightly-2021-08-17-x86_64-unknown-linux-gnu (overridden by '/code/rust-toolchain')
rustc 1.56.0-nightly (0035d9dce 2021-08-16)
```

[Rust]: https://www.rust-lang.org/
[rustup]: https://rustup.rs/
[Oasis Core's Prerequisites]:
  https://docs.oasis.dev/oasis-core/development-setup/build-environment-setup-and-building/prerequisites
[rust-toolchain]: rust-toolchain
[rust-toolchain-precedence]:
  https://github.com/rust-lang/rustup/blob/master/README.md#override-precedence

### Non-SGX Binary

To build the non-SGX binary of the Emerald ParaTime, checkout the appropriate
version and run:

```
cargo build --release
```

The resulting ELF binary is located at `target/release/emerald-paratime`.

_NOTE: The non-SGX binary is dynamically linked so it may not be portable
between machines with different versions of shared libraries._

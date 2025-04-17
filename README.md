<p align="center">
  <img src="./icons/rpl-icon.svg" width="50%"/>
</p>

# RPL

This is the main source code repository of RPL (Rust Pattern Language). It contains the toolchain and documentation of RPL.

## What is RPL?

RPL is a domain-specific language for modeling Rust code patterns.

The toolchain of RPL, which is a custom configuration of Rust compiler, enables accurate identification of code instances that demonstrate semantic equivalence to existing patterns.

## Features

-   Model Rust code patterns based on MIR, just like writing real Rust code.✨
-   Enable programmatic semantic equivalence checks based on CFG and DDG with graph matching algorithms. 🔍
-   Support pattern customization to simplify modeling and predicate integration to ensure precise matching. 🛠️
-   Provide clear, user-friendly error messages and actionable suggestions. 💡

## Quick Start

-   Clone the repository and enter the directory: `git clone https://github.com/stuuupidcat/RPL.git && cd RPL`

-   Install RPL as a cargo subcommand: `cargo install --path .`

-   Run RPL in your own Rust project: `cargo +nightly-2025-02-14 rpl`

> Just like using `cargo clippy` to check your Rust code.

## RPL Language Reference

See [this website](https://stuuupidcat.github.io/RPL/) for the language reference of RPL (Work in progress).

## Getting Help

Feel free to open an issue or contact us via [email](stuuupidcat@163.com) if you have any questions.

## Contributing

See [this document](./CONTRIBUTING.md) for contribution-related instructions.

## License

This project is licensed under the [GNU General Public License v3.0](./LICENSE).

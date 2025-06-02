# Installation

First, use `cargo install --path .` in current directory to install RPL as a cargo subcommand.

Then, use `cargo +nightly-2025-02-14 rpl` to run RPL in your own repository to detect errors, where `+nightly-2025-02-14` is the current toolchain RPL is using. You may upate this argument if RPL is switched into a new toolchain.

> Every three months (or so), the toolchain will be updated to the latest nightly version. You can check the current toolchain by running `rustc -V` in the RPL repository.

# Tests

-   Use `cargo tests` (short for `cargo test --all`) to run all tests.
-   Use `cargo uitest` (short for `cargo test --test compile-test`) to run compile tests.
-   Use `cargo uibless` (short for `cargo test --test compile-test --bless`) to run compile tests (in bless mode), which means the `.stderr` file will be automatically fixed according to the compiler outputs.

## Test Directives

You can control the way a test is built and interpreted through adding test directives.

See <https://rustc-dev-guide.rust-lang.org/tests/ui.html> and <https://rustc-dev-guide.rust-lang.org/tests/directives.html> for more information.

The currently mostly used one is `//@ ignore-on-host`.

# Debugging

## Logs based on `tracing`

We use the [`tracing`](https://docs.rs/tracing/latest/tracing/) crate for logging, the same as in rustc.

Run `RPL_LOG=info cargo run -b rpl-driver -- path/to/test.rs` to run rpl driver for logs, where `RPL_LOG=info` is short for `RUSTC_LOG=rpl=info` that enables `info` level logs for all RPL crates.

For `debug` and `trace` level logs, unfortunately you cannot use the nightly rustc toolchain because it disables log levels below `info` for performance. Instead, you have to use a custom rustc built from source. You have to make sure that your custom rustc toolchain is the same as that used in RPL, or otherwise it might not compile.

The recommended workflow to setup a custom rustc is:

-   Get the latest nightly rust toolchain by running `rustup toolchain install nightly`/`rustup update nightly`;
-   Type `rustc -V` in the RPL repository to show the current toolchain it is using (currently, it is `rustc 1.86.0-nightly (a567209da 2025-02-13)`), and remember the revision `a567209da` in it;
-   Change your directory where you would like to put the rust source code, and clone the rust repository using `git clone https://github.com/rust-lang/rust.git && cd rust`;
-   Checkout to the given commit, and it is recommended to use `git worktree add ../rust-nightly-2025-02-13 a567209da && cd ../rust-nightly-2025-02-13`;
-   Run `git log` to make sure that the first commit is the same as that produced by `rustc -V` in the RPL repository;
-   Run `./x setup` and choose `b) compiler: Contribute to the compiler itself` when it asks you `What do you want to do with x.py?` (the `compiler` profile will enable debugging by default), and choose the default options for other settings;
-   Run `./x dist --stage 1` to build a custom toolchain from source, it might take minutes or hours to finish(On a MacBookPro M1, it takes about 22 minutes); Here `--stage 1` means you only need to build a compiler from a nightly rustc (called the bootstrap compiler) instead of your own buildings;
-   Run `rustup toolchain link nightly-2025-02-13-stage1 build/host/stage1` to link your custom stage1 compiler to a new toolchain named `nightly-2025-02-13-stage1`;
-   Now go back to the RPL repository, you will be able to run `RPL_LOG=debug cargo +nightly-2025-02-13-stage1 run --bin rpl-driver` to build and run RPL using your custom rustc with the debug log level;

Additionally, when using `tracing` in a crate, **do not** specify it as a dependency in `Cargo.toml`, instead, use `#[macro_use] extern crate tracing;` to use that one shipped with the rustc toolchain.

### Troubleshooting: dylib not found

`RPL_LOG=debug cargo +nightly-2025-02-13-stage1 run --bin rpl-driver` might not work, and you will probably see an error like this:

```
   Compiling rpl_patterns v0.1.0
error: ./target/debug/deps/librpl_macros-735c931c29d05061.so: librustc_driver-649529b68e4c03de.so: cannot open shared object file: No such file or directory
  --> crates/rpl_patterns/src/lib.rs:16:1
   |
16 | extern crate rpl_macros;
   | ^^^^^^^^^^^^^^^^^^^^^^^^
```

I haven't found a good solution but here is the hacks:

-   Go to the directory where your `rust-nightly-2025-02-13` locates and run `ln -sf build/host/stage1 ~/.rustup/toolchains/nightly-2025-02-13-stage1` to link your custom toolchain to directory that is easy to find;
-   Add
    -   `export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$HOME/.rustup/toolchains/nightly-2025-02-13-stage1/lib/rustlib/x86_64-unknown-linux-gnu/lib`(for Linux) or
    -   `export DYLD_LIBRARY_PATH=$DYLD_LIBRARY_PATH:$HOME/.rustup/toolchains/nightly-2025-02-13-stage1/lib/rustlib/aarch64-apple-darwin/lib`(for MacOS)
        to the end of the `~/.bashrc` file (or `~/.zshrc` if you use ZSH)
-   Run `source ~/.bashrc` (or `source ~/.zshrc` if you use ZSH);
-   Now return to the RPL repository and you may succeed to run `RPL_LOG=debug cargo +nightly-2025-02-13-stage1 run -b rpl-driver` and you will see the debugging logs printed on the screen;

### Troubleshooting: `'iostream' file not found`

When you run `./x dist --stage 1`, you may encounter an error like this:

```
/rust-nightly-2025-02-13/src/tools/libcxx-version/main.cpp:8:10: fatal error: 'iostream' file not found
```

Here is a related PR: https://github.com/rust-lang/rust/pull/125411

To fix this(on MacOS), run `sudo rm -rf /Library/Developer/CommandLineTools` and then run `xcode-select --install` to reinstall the command line tools.

### View logs

The rustc logs are scope trees with indentations, it is helpful if your code editor support folding by indentations.

In Vim, you can set fold method to `indent` and use `za` to toggle the folds on and off.

```vim
set shiftwidth=2
set foldenable
set foldmethod=indent
```

In VsCode, folding by indentation is supported by default.

## Debugging with VSCode and lldb

When use the debug mode in VSCode, you may encounter the following error:

```
dyld[18303]: Library not loaded: @rpath/librustc_driver-c038cb8a39fd51a8.dylib
  Referenced from: <3AB4A695-133F-36BC-90CA-8C8D77B22C5E> /path/to/RPL/target/debug/rpl-driver
  Reason: no LC_RPATH's found
```

To fix this, you can add the following to your `launch.json`(There may be some slight differences considering your OS and environment):

```json
"env": {
    "DYLD_LIBRARY_PATH": "${env:HOME}/.rustup/toolchains/nightly-2025-02-14-aarch64-apple-darwin/lib",
    "LD_LIBRARY_PATH": "${env:HOME}/.rustup/toolchains/nightly-2025-02-14-aarch64-unknown-linux-gnu/lib"
},
```

## Pass environment variables to RPL

Pass pattern path to RPL by setting the `RPL_ARGS` environment variable.

For example:

```
RPL_PATS=~/home/code/projects/RPL/docs/patterns/CVE-2019-16138-MIR.rpl RUSTFLAGS=-Zinline-mir=false cargo rpl
```

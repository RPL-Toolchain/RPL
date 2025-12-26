use std::fs::{File, read_to_string};
use std::io::Write as _;
use std::process::{Command, Stdio};

use quote::quote;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo::rerun-if-changed=src/grammar/RPL.pest");
    let parser = {
        use pest_typed_generator::derive_typed_parser;
        let input = quote! {
            /// Underlying definition of the parser written with Pest.
            #[derive(TypedParser)]
            #[grammar = "grammar/RPL.pest"]
            #[emit_rule_reference]
            pub struct Grammar;
        };
        derive_typed_parser(input.clone(), false, true)
    };

    let rustfmt = find_rustfmt_path()?;

    let child = Command::new(rustfmt)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.as_ref().unwrap();
    stdin.write_all(
        b"#![allow(warnings)]\n/// Underlying definition of the RPL parser written with Pest.\npub struct Grammar;",
    )?;
    write!(stdin, "{}", parser)?;
    let output = child.wait_with_output()?;
    if let Ok(s) = read_to_string("src/parser.rs") {
        if s == String::from_utf8_lossy(&output.stdout) {
            eprintln!("No changes in parser.rs, skipping write.");
            return Ok(()); // No changes, no need to write
        }
    }
    File::create("src/parser.rs")?.write_all(&output.stdout)?;

    Ok(())
}

fn find_rustfmt_path() -> Result<String, Box<dyn std::error::Error>> {
    let rustup_home = std::env::var("RUSTUP_HOME")?;

    let toolchains_dir = format!("{}/toolchains", rustup_home);
    let toolchains = std::fs::read_dir(toolchains_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();

    for toolchain in toolchains {
        if !toolchain
            .file_stem()
            // Remember to update the date in the condition below if you change the nightly toolchain date.
            // FIXME: This is a temporary solution to use a specific nightly toolchain.
            .is_some_and(|stem| stem.to_str().is_some_and(|stem| stem.starts_with("nightly-2025-02-14")))
        {
            continue; // Use only the nightly toolchain from 2025-02-14
        }
        let rustfmt_executable = if cfg!(windows) { "rustfmt.exe" } else { "rustfmt" };
        let rustfmt_candidate = toolchain.join("bin").join(rustfmt_executable);
        if rustfmt_candidate.exists() {
            return Ok(rustfmt_candidate.to_str().unwrap().to_string());
        }
    }

    let err = "Could not find rustfmt in any toolchain";
    Err(err.into())
}

#![allow(rustc::diagnostic_outside_of_impl)]
#![allow(rustc::untranslatable_diagnostic)]
#![feature(rustc_private)]
#![feature(let_chains)]
#![feature(os_str_display)]
// warn on lints, that are included in `rust-lang/rust`s bootstrap
#![warn(rust_2018_idioms, unused_lifetimes)]
// warn on rustc internal lints
#![warn(rustc::internal)]
#![recursion_limit = "256"]

// FIXME: switch to something more ergonomic here, once available.
// (Currently there is no way to opt into sysroot crates without `extern crate`.)
extern crate rustc_driver;
extern crate rustc_log;
extern crate rustc_session;
#[allow(unused_extern_crates)]
extern crate tracing;

use std::env;
use std::fs::read_to_string;
use std::ops::Deref;
use std::path::Path;
use std::process::exit;

use anstream::println;
use rpl_interface::{DefaultCallbacks, RplCallbacks, RustcCallbacks};
use rustc_session::EarlyDiagCtxt;
use rustc_session::config::ErrorOutputType;

/// If a command-line option matches `find_arg`, then apply the predicate `pred` on its value. If
/// true, then return it. The parameter is assumed to be either `--arg=value` or `--arg value`.
fn arg_value<'a, T: Deref<Target = str>>(
    args: &'a [T],
    find_arg: &str,
    pred: impl Fn(&str) -> bool,
) -> Option<&'a str> {
    let mut args = args.iter().map(Deref::deref);
    while let Some(arg) = args.next() {
        let mut arg = arg.splitn(2, '=');
        if arg.next() != Some(find_arg) {
            continue;
        }

        match arg.next().or_else(|| args.next()) {
            Some(v) if pred(v) => return Some(v),
            _ => {},
        }
    }
    None
}

#[test]
fn test_arg_value() {
    let args = &["--bar=bar", "--foobar", "123", "--foo"];

    assert_eq!(arg_value(&[] as &[&str], "--foobar", |_| true), None);
    assert_eq!(arg_value(args, "--bar", |_| false), None);
    assert_eq!(arg_value(args, "--bar", |_| true), Some("bar"));
    assert_eq!(arg_value(args, "--bar", |p| p == "bar"), Some("bar"));
    assert_eq!(arg_value(args, "--bar", |p| p == "foo"), None);
    assert_eq!(arg_value(args, "--foobar", |p| p == "foo"), None);
    assert_eq!(arg_value(args, "--foobar", |p| p == "123"), Some("123"));
    assert_eq!(arg_value(args, "--foobar", |p| p.contains("12")), Some("123"));
    assert_eq!(arg_value(args, "--foo", |_| true), None);
}

#[allow(unused)]
fn consume_arg_values(args: &mut Vec<String>, find_arg: &str) -> Vec<String> {
    let mut found_values = Vec::new();
    let find_arg_with_eq = format!("{}=", find_arg);
    let mut i = 0;

    while i < args.len() {
        let arg = args[i].clone();
        if let Some(stripped) = arg.strip_prefix(&find_arg_with_eq) {
            args.remove(i);
            found_values.push(stripped.to_string());
        } else if arg == find_arg {
            args.remove(i);
            if i < args.len() {
                found_values.push(args.remove(i));
            }
        } else {
            i += 1;
        }
    }
    found_values
}

#[test]
fn test_consume_arg_values() {
    let args = ["--foo=bar", "--bar", "x", "--foo", "baz", "--foo=qux"];
    let mut args = args.iter().map(ToString::to_string).collect();
    assert_eq!(consume_arg_values(&mut args, "--foo"), vec!["bar", "baz", "qux"]);
    assert_eq!(args, vec!["--bar", "x"]);
}

#[allow(clippy::ignored_unit_patterns)]
fn display_help() {
    println!("{}", help_message());
}

const BUG_REPORT_URL: &str = "https://github.com/RPL-Toolchain/RPL/issues";

fn logger_config() -> rustc_log::LoggerConfig {
    let mut cfg = rustc_log::LoggerConfig::from_env("RUSTC_LOG");

    if let Ok(var) = env::var("RPL_LOG") {
        if let Ok(level) = var.parse::<tracing::Level>() {
            // RPL_LOG serves as default for RUSTC_LOG, if that is not set.
            match cfg.filter {
                Err(env::VarError::NotPresent | env::VarError::NotUnicode(_)) => {
                    // We try to be a bit clever here: if `RPL_LOG` is just a single level
                    // used for everything, we only apply it to the parts of rustc that are
                    // CTFE-related. Otherwise, we use it verbatim for `RUSTC_LOG`.
                    // This way, if you set `RPL_LOG=info`, you get only the right parts of
                    // rustc traced, but you can also do `RPL_LOG=rpl=info,rustc_const_eval::interpret=debug`.
                    cfg.filter = Ok(format!("rpl={level}"));
                },
                Ok(ref mut filter) => {
                    filter.push_str(",rpl=");
                    filter.push_str(&var);
                },
            }
        } else {
            EarlyDiagCtxt::new(ErrorOutputType::default()).early_fatal(format!(
                "RPL_LOG must be a valid tracing level, like `info` or `debug`: {var}"
            ));
        }
    }
    cfg
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::ignored_unit_patterns)]
pub fn main() {
    let early_dcx = EarlyDiagCtxt::new(ErrorOutputType::default());

    rustc_driver::init_logger(&early_dcx, logger_config());

    rustc_driver::install_ice_hook(BUG_REPORT_URL, |handler| {
        // FIXME: this macro calls unwrap internally but is called in a panicking context!  It's not
        // as simple as moving the call from the hook to main, because `install_ice_hook` doesn't
        // accept a generic closure.
        let version_info = rustc_tools_util::get_version_info!();
        // If backtraces are enabled, also print the query stack
        // let backtrace = env::var_os("RUST_BACKTRACE").is_some_and(|x| &x != "0");

        // let num_frames = if backtrace { None } else { Some(2) };
        // rpl_interface::interface::try_print_query_stack(handler, num_frames, None);
        handler.handle().note(format!("RPL-driver version: {version_info}"));
    });

    exit(rustc_driver::catch_with_exit_code(move || {
        let mut orig_args: Vec<String> = rustc_driver::args::raw_args(&early_dcx);

        let has_sysroot_arg = |args: &mut [String]| -> bool {
            if arg_value(args, "--sysroot", |_| true).is_some() {
                return true;
            }
            // https://doc.rust-lang.org/rustc/command-line-arguments.html#path-load-command-line-flags-from-a-path
            // Beside checking for existence of `--sysroot` on the command line, we need to
            // check for the arg files that are prefixed with @ as well to be consistent with rustc
            for arg in args.iter() {
                if let Some(arg_file_path) = arg.strip_prefix('@') {
                    if let Ok(arg_file) = read_to_string(arg_file_path) {
                        let split_arg_file: Vec<String> = arg_file.lines().map(ToString::to_string).collect();
                        if arg_value(&split_arg_file, "--sysroot", |_| true).is_some() {
                            return true;
                        }
                    }
                }
            }
            false
        };

        let sys_root_env = std::env::var("SYSROOT").ok();
        let pass_sysroot_env_if_given = |args: &mut Vec<String>, sys_root_env| {
            if let Some(sys_root) = sys_root_env {
                if !has_sysroot_arg(args) {
                    args.extend(vec!["--sysroot".into(), sys_root]);
                }
            };
        };

        // make "rpl-driver --rustc" work like a subcommand that passes further args to "rustc"
        // for example `rpl-driver --rustc --version` will print the rustc version that rpl-driver
        // uses
        if let Some(pos) = orig_args.iter().position(|arg| arg == "--rustc") {
            orig_args.remove(pos);
            orig_args[0] = "rustc".to_string();

            let mut args: Vec<String> = orig_args.clone();
            pass_sysroot_env_if_given(&mut args, sys_root_env);

            // return rustc_driver::RunCompiler::new(&args, &mut DefaultCallbacks).run();
            return rustc_driver::run_compiler(&args, &mut DefaultCallbacks);
        }

        if orig_args.iter().any(|a| a == "--version" || a == "-V") {
            let version_info = rustc_tools_util::get_version_info!();

            println!("{version_info}");
            exit(0);
        }

        // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
        // We're invoking the compiler programmatically, so we ignore this/
        let wrapper_mode = orig_args.get(1).map(Path::new).and_then(Path::file_stem) == Some("rustc".as_ref());
        if wrapper_mode {
            // we still want to be able to invoke it normally though
            orig_args.remove(1);
        }

        if !wrapper_mode && (orig_args.iter().any(|a| a == "--help" || a == "-h") || orig_args.len() == 1) {
            display_help();
            exit(0);
        }

        let mut args: Vec<String> = orig_args.clone();
        pass_sysroot_env_if_given(&mut args, sys_root_env);

        let pattern_paths = match env::var("RPL_PATS") {
            Ok(val) => Some(val.split(':').map(ToString::to_string).collect()),
            Err(env::VarError::NotPresent) => None,
            Err(env::VarError::NotUnicode(var)) => {
                early_dcx.early_fatal(format!("RPL_PATS is not valid unicode: {}", var.display()))
            },
        };

        let mut no_deps = false;
        let rpl_args_var = env::var(rpl_interface::RPL_ARGS_ENV).ok();
        let rpl_args = rpl_args_var
            .as_deref()
            .unwrap_or_default()
            .split("__RPL_HACKERY__")
            .filter_map(|s| match s {
                "" => None,
                "--no-deps" => {
                    no_deps = true;
                    None
                },
                _ => Some(s.to_string()),
            })
            .chain(vec!["--cfg".into(), "rpl".into()])
            .collect::<Vec<String>>();

        // We enable RPL if one of the following conditions is met
        // - IF RPL is run on its test suite OR
        // - IF RPL is run on the main crate, not on deps (`!cap_lints_allow`) THEN
        //    - IF `--no-deps` is not set (`!no_deps`) OR
        //    - IF `--no-deps` is set and RPL is run on the specified primary package
        let cap_lints_allow = arg_value(&orig_args, "--cap-lints", |val| val == "allow").is_some()
            && arg_value(&orig_args, "--force-warn", |val| val.contains("rpl::")).is_none();
        let in_primary_package = env::var("CARGO_PRIMARY_PACKAGE").is_ok();

        let rpl_enabled = !cap_lints_allow && (!no_deps || in_primary_package);
        if rpl_enabled {
            args.extend(rpl_args);
            /* rustc_driver::RunCompiler::new(&args, &mut RplCallbacks::new(rpl_args_var))
            .set_using_internal_features(using_internal_features)
            .run() */
            rustc_driver::run_compiler(&args, &mut RplCallbacks::new(rpl_args_var, pattern_paths))
        } else {
            rustc_driver::run_compiler(&args, &mut RustcCallbacks::new(rpl_args_var))
        }
    }))
}

#[must_use]
fn help_message() -> &'static str {
    color_print::cstr!(
        "Checks a file according to RustPatLang (RPL) bug detectors.
Run <cyan>rpl-driver</> with the same arguments you use for <cyan>rustc</>

<green,bold>Usage</>:
    <cyan,bold>RPL_PATS=path/to/patterns rpl-driver</> <cyan>[OPTIONS] INPUT</>

<green,bold>Common options:</>
    <cyan,bold>-h</>, <cyan,bold>--help</>               Print this message
    <cyan,bold>-V</>, <cyan,bold>--version</>            Print version info and exit
    <cyan,bold>--rustc</>                  Pass all arguments to <cyan>rustc</>
"
    )
}

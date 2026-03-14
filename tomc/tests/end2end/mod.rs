//! End-to-end CLI flag tests for `tomc`
//!
//! Every test invokes the compiled `tomc` binary via `std::process::Command`.
//! Before any test can run, `ensure_built()` asserts that the binary has been
//! compiled, acting as an explicit precondition for the entire suite.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Precondition: tomc must compile successfully
// ---------------------------------------------------------------------------

/// Absolute path to the compiled `tomc` binary (set by Cargo for integration tests).
const TOMC_BIN: &str = env!("CARGO_BIN_EXE_tomc");

/// Canonical hello.tomi used across tests – relative to the workspace root.
fn hello_tomi() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../examples/tomc/hello.tomi")
}

/// Called at the start of every test.  Uses a `OnceLock` so the build check
/// runs once per test binary execution, not once per test function.
fn ensure_built() {
    static BUILT: OnceLock<()> = OnceLock::new();
    BUILT.get_or_init(|| {
        let status = Command::new("cargo")
            .args(["build", "--bin", "tomc"])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .status()
            .expect("failed to invoke `cargo build --bin tomc`");
        assert!(status.success(), "tomc failed to compile – end2end tests cannot run");
    });
}

/// Convenience wrapper: create a `Command` ready to invoke `tomc`.
fn tomc() -> Command {
    ensure_built();
    Command::new(TOMC_BIN)
}

// ---------------------------------------------------------------------------
// Helper assertions
// ---------------------------------------------------------------------------

fn assert_success(output: &std::process::Output, context: &str) {
    assert!(
        output.status.success(),
        "{context}: expected exit 0, got {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn assert_failure(output: &std::process::Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context}: expected non-zero exit, got {}\nstdout: {}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

// ---------------------------------------------------------------------------
// Precondition test — must pass before any other test is meaningful
// ---------------------------------------------------------------------------

#[test]
fn test_00_tomc_binary_exists_and_is_executable() {
    ensure_built();
    let bin = Path::new(TOMC_BIN);
    assert!(bin.exists(), "tomc binary not found at {}", bin.display());
    assert!(bin.is_file(), "tomc path is not a file: {}", bin.display());
}

// ---------------------------------------------------------------------------
// --version / -V
// ---------------------------------------------------------------------------

#[test]
fn test_version_long() {
    let out = tomc().arg("--version").output().unwrap();
    assert_success(&out, "--version");
    assert!(
        stdout(&out).contains("tomc"),
        "--version should print binary name, got: {}",
        stdout(&out)
    );
}

#[test]
fn test_version_short() {
    let out = tomc().arg("-V").output().unwrap();
    assert_success(&out, "-V");
    assert!(stdout(&out).contains("tomc"), "-V should print binary name");
}

// ---------------------------------------------------------------------------
// --help / -h
// ---------------------------------------------------------------------------

#[test]
fn test_help_long() {
    let out = tomc().arg("--help").output().unwrap();
    assert_success(&out, "--help");
    let s = stdout(&out);
    assert!(s.contains("Usage") || s.contains("usage"), "--help should contain usage info");
    assert!(s.contains("tomc"), "--help should mention the binary name");
}

#[test]
fn test_help_short() {
    let out = tomc().arg("-h").output().unwrap();
    assert_success(&out, "-h");
    assert!(stdout(&out).contains("tomc"), "-h output should mention the binary name");
}

// ---------------------------------------------------------------------------
// Error: missing input file / non-existent file
// ---------------------------------------------------------------------------

#[test]
fn test_no_input_file_exits_nonzero() {
    let out = tomc().output().unwrap();
    assert_failure(&out, "no input");
    let combined = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        combined.contains("no input") || combined.contains("INPUT"),
        "error message should mention missing input, got: {combined}"
    );
}

#[test]
fn test_nonexistent_file_exits_nonzero() {
    let out = tomc().arg("/nonexistent/path/file.tomi").output().unwrap();
    assert_failure(&out, "nonexistent file");
    let combined = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        combined.to_lowercase().contains("cannot read")
            || combined.to_lowercase().contains("error"),
        "should report an error for missing file, got: {combined}"
    );
}

// ---------------------------------------------------------------------------
// --explain <ERROR_CODE>
// ---------------------------------------------------------------------------

#[test]
fn test_explain_known_code_e0001() {
    let out = tomc().args(["--explain", "E0001"]).output().unwrap();
    assert_success(&out, "--explain E0001");
    let s = stdout(&out);
    assert!(s.contains("E0001"), "explain output should mention the error code");
}

#[test]
fn test_explain_known_code_e0010() {
    let out = tomc().args(["--explain", "E0010"]).output().unwrap();
    assert_success(&out, "--explain E0010");
    assert!(stdout(&out).contains("E0010"), "explain output should mention E0010");
}

#[test]
fn test_explain_all_known_codes() {
    for n in 1..=10u32 {
        let code = format!("E{n:04}");
        let out = tomc().args(["--explain", &code]).output().unwrap();
        assert_success(&out, &format!("--explain {code}"));
        assert!(
            stdout(&out).contains(&code),
            "--explain {code} output should contain the code itself"
        );
    }
}

#[test]
fn test_explain_unknown_code_exits_nonzero() {
    let out = tomc().args(["--explain", "E9999"]).output().unwrap();
    assert_failure(&out, "--explain E9999");
    let combined = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        combined.contains("unknown") || combined.contains("E9999"),
        "should report unknown code, got: {combined}"
    );
}

#[test]
fn test_explain_lowercase_code_accepted() {
    // Codes should be case-insensitive
    let out = tomc().args(["--explain", "e0001"]).output().unwrap();
    assert_success(&out, "--explain e0001 (lowercase)");
}

// ---------------------------------------------------------------------------
// --check
// ---------------------------------------------------------------------------

#[test]
fn test_check_flag_valid_source() {
    let out = tomc().args(["--check", hello_tomi().to_str().unwrap()]).output().unwrap();
    assert_success(&out, "--check");
    let s = stdout(&out);
    assert!(
        s.contains("syntax check passed") || s.contains("✓"),
        "--check should report success, got: {s}"
    );
}

#[test]
fn test_check_does_not_produce_output_file() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("should_not_exist.rs");
    let out = tomc()
        .args(["--check", "-o", out_path.to_str().unwrap(), hello_tomi().to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out, "--check with -o");
    assert!(!out_path.exists(), "--check should not write output files");
}

// ---------------------------------------------------------------------------
// --emit tokens
// ---------------------------------------------------------------------------

#[test]
fn test_emit_tokens() {
    let out = tomc().args(["--emit", "tokens", hello_tomi().to_str().unwrap()]).output().unwrap();
    assert_success(&out, "--emit tokens");
    let s = stdout(&out);
    assert!(
        s.contains("Tokens") || s.contains("Token") || s.contains("Def") || s.contains("Ident"),
        "--emit tokens should print token information, got: {s}"
    );
}

// ---------------------------------------------------------------------------
// --emit ast
// ---------------------------------------------------------------------------

#[test]
fn test_emit_ast() {
    let out = tomc().args(["--emit", "ast", hello_tomi().to_str().unwrap()]).output().unwrap();
    assert_success(&out, "--emit ast");
    let s = stdout(&out);
    assert!(
        s.contains("AST") || s.contains("Module") || s.contains("Function"),
        "--emit ast should print AST information, got: {s}"
    );
}

// ---------------------------------------------------------------------------
// --emit code  (produces a .rs file)
// ---------------------------------------------------------------------------

#[test]
fn test_emit_code_produces_rs_file() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("hello.rs");
    let out = tomc()
        .args(["--emit", "code", "-o", out_path.to_str().unwrap(), hello_tomi().to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out, "--emit code");
    assert!(out_path.exists(), "--emit code should write a .rs file");
    let rs = std::fs::read_to_string(&out_path).unwrap();
    assert!(rs.contains("fn main"), "generated Rust should contain fn main");
}

// ---------------------------------------------------------------------------
// --emit metadata
// ---------------------------------------------------------------------------

#[test]
fn test_emit_metadata() {
    let out = tomc().args(["--emit", "metadata", hello_tomi().to_str().unwrap()]).output().unwrap();
    assert_success(&out, "--emit metadata");
    // metadata emit should produce some output (no crash is the minimum guarantee)
}

// ---------------------------------------------------------------------------
// --emit bin  (produces a native binary)
// ---------------------------------------------------------------------------

#[test]
fn test_emit_bin_produces_executable() {
    let tmp = tempfile::tempdir().unwrap();
    let bin_path = tmp.path().join("hello_end2end");
    let out = tomc()
        .args(["--emit", "bin", "-o", bin_path.to_str().unwrap(), hello_tomi().to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out, "--emit bin");
    assert!(bin_path.exists(), "--emit bin should produce a binary");

    // Run the binary and verify output
    let run = Command::new(&bin_path).output().unwrap();
    assert!(run.status.success(), "compiled binary should exit 0");
    let run_out = String::from_utf8_lossy(&run.stdout);
    assert!(
        run_out.contains("Hello, World!"),
        "compiled binary should print Hello, World!, got: {run_out}"
    );
}

// ---------------------------------------------------------------------------
// -o / --output  (custom output path)
// ---------------------------------------------------------------------------

#[test]
fn test_output_flag_short() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("custom_output.rs");
    let out = tomc()
        .args(["-o", out_path.to_str().unwrap(), "--emit", "code", hello_tomi().to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out, "-o (short)");
    assert!(out_path.exists(), "-o should write to the specified path");
}

#[test]
fn test_output_flag_long() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("long_output.rs");
    let out = tomc()
        .args([
            "--output",
            out_path.to_str().unwrap(),
            "--emit",
            "code",
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "--output (long)");
    assert!(out_path.exists(), "--output should write to the specified path");
}

// ---------------------------------------------------------------------------
// --edition
// ---------------------------------------------------------------------------

#[test]
fn test_edition_2024() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("edition.rs");
    let out = tomc()
        .args([
            "--edition",
            "2024",
            "--emit",
            "code",
            "-o",
            out_path.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "--edition 2024");
}

// ---------------------------------------------------------------------------
// --album-type
// ---------------------------------------------------------------------------

#[test]
fn test_album_type_lib() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("lib.rs");
    let out = tomc()
        .args([
            "--album-type",
            "lib",
            "--emit",
            "code",
            "-o",
            out_path.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "--album-type lib");
    assert!(out_path.exists(), "--album-type lib should produce .rs output");
}

#[test]
fn test_album_type_bin() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("bin_out.rs");
    let out = tomc()
        .args([
            "--album-type",
            "bin",
            "--emit",
            "code",
            "-o",
            out_path.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "--album-type bin");
    assert!(out_path.exists(), "--album-type bin with --emit code should produce .rs output");
}

// ---------------------------------------------------------------------------
// -C codegen options
// ---------------------------------------------------------------------------

#[test]
fn test_codegen_opt_level_0() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("opt0");
    let out = tomc()
        .args([
            "-C",
            "opt-level=0",
            "--emit",
            "bin",
            "-o",
            bin.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-C opt-level=0");
}

#[test]
fn test_codegen_opt_level_3() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("opt3");
    let out = tomc()
        .args([
            "-C",
            "opt-level=3",
            "--emit",
            "bin",
            "-o",
            bin.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-C opt-level=3");
}

#[test]
fn test_codegen_opt_level_invalid_exits_nonzero() {
    let out = tomc()
        .args(["-C", "opt-level=5", "--emit", "code", hello_tomi().to_str().unwrap()])
        .output()
        .unwrap();
    assert_failure(&out, "-C opt-level=5 (invalid)");
}

#[test]
fn test_codegen_overflow_checks_no() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("ovf");
    let out = tomc()
        .args([
            "-C",
            "overflow-checks=no",
            "--emit",
            "bin",
            "-o",
            bin.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-C overflow-checks=no");
}

#[test]
fn test_codegen_overflow_checks_yes() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("ovf.rs");
    let out = tomc()
        .args([
            "-C",
            "overflow-checks=yes",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-C overflow-checks=yes");
}

#[test]
fn test_codegen_debug_info_yes() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("dbg");
    let out = tomc()
        .args([
            "-C",
            "debug-info=yes",
            "--emit",
            "bin",
            "-o",
            bin.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-C debug-info=yes");
}

#[test]
fn test_codegen_debug_info_no() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("nodbg.rs");
    let out = tomc()
        .args([
            "-C",
            "debug-info=no",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-C debug-info=no");
}

#[test]
fn test_codegen_lto_yes() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("lto");
    let out = tomc()
        .args([
            "-C",
            "lto=yes",
            "--emit",
            "bin",
            "-o",
            bin.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-C lto=yes");
}

#[test]
fn test_codegen_lto_no() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("nolto.rs");
    let out = tomc()
        .args([
            "-C",
            "lto=no",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-C lto=no");
}

#[test]
fn test_codegen_unknown_opt_exits_nonzero() {
    let out = tomc()
        .args(["-C", "unknown-option=42", "--emit", "code", hello_tomi().to_str().unwrap()])
        .output()
        .unwrap();
    assert_failure(&out, "-C unknown-option (invalid)");
    let combined = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        combined.contains("unknown codegen option") || combined.contains("unknown"),
        "should report unknown option, got: {combined}"
    );
}

#[test]
fn test_codegen_multiple_opts_combined() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("multi");
    let out = tomc()
        .args([
            "-C",
            "opt-level=2",
            "-C",
            "overflow-checks=no",
            "-C",
            "debug-info=yes",
            "--emit",
            "bin",
            "-o",
            bin.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-C multiple options combined");
}

// ---------------------------------------------------------------------------
// -W / -D / -A  lint flags
// ---------------------------------------------------------------------------

#[test]
fn test_warn_lint_accepted() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("warn.rs");
    let out = tomc()
        .args([
            "-W",
            "unused-variables",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-W unused-variables");
}

#[test]
fn test_deny_lint_accepted() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("deny.rs");
    let out = tomc()
        .args([
            "-D",
            "dead-code",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-D dead-code");
}

#[test]
fn test_allow_lint_accepted() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("allow.rs");
    let out = tomc()
        .args([
            "-A",
            "warnings",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-A warnings");
}

#[test]
fn test_all_known_lints_with_warn() {
    let known = [
        "unused-variables",
        "unused-parameters",
        "unused-imports",
        "dead-code",
        "unreachable-code",
        "unused-mut",
        "warnings",
    ];
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("alllints.rs");
    let mut cmd = tomc();
    for lint in &known {
        cmd.args(["-W", lint]);
    }
    cmd.args(["--emit", "code", "-o", rs.to_str().unwrap(), hello_tomi().to_str().unwrap()]);
    let out = cmd.output().unwrap();
    assert_success(&out, "-W all known lints");
}

#[test]
fn test_unknown_lint_does_not_exit_nonzero() {
    // Unknown lints emit a warning but must not cause a non-zero exit
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("unknown_lint.rs");
    let out = tomc()
        .args([
            "-W",
            "not-a-real-lint",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-W unknown-lint (should warn, not error)");
    // The warning should appear on stderr
    assert!(
        stderr(&out).contains("unknown lint") || stderr(&out).contains("warning"),
        "should warn about unknown lint on stderr, got: {}",
        stderr(&out)
    );
}

// ---------------------------------------------------------------------------
// -v / --verbose
// ---------------------------------------------------------------------------

#[test]
fn test_verbose_short() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("verbose_out.rs");
    let out = tomc()
        .args(["-v", "--emit", "code", "-o", rs.to_str().unwrap(), hello_tomi().to_str().unwrap()])
        .output()
        .unwrap();
    assert_success(&out, "-v");
    assert!(
        stderr(&out).contains("info:") || stderr(&out).contains("tomc"),
        "-v should print verbose info to stderr, got: {}",
        stderr(&out)
    );
}

#[test]
fn test_verbose_long() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("verbose_long.rs");
    let out = tomc()
        .args([
            "--verbose",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "--verbose");
    assert!(
        stderr(&out).contains("info:") || stderr(&out).contains("tomc"),
        "--verbose should print verbose info to stderr, got: {}",
        stderr(&out)
    );
}

// ---------------------------------------------------------------------------
// --color
// ---------------------------------------------------------------------------

#[test]
fn test_color_never() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("color_never.rs");
    let out = tomc()
        .args([
            "--color",
            "never",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "--color never");
}

#[test]
fn test_color_always() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("color_always.rs");
    let out = tomc()
        .args([
            "--color",
            "always",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "--color always");
}

#[test]
fn test_color_auto() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("color_auto.rs");
    let out = tomc()
        .args([
            "--color",
            "auto",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "--color auto");
}

// ---------------------------------------------------------------------------
// -t / --target
// ---------------------------------------------------------------------------

#[test]
fn test_target_rust_short() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("target_rust.rs");
    let out = tomc()
        .args([
            "-t",
            "rust",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "-t rust");
    assert!(rs.exists(), "-t rust --emit code should produce a .rs file");
}

#[test]
fn test_target_rust_long() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("target_rust_long.rs");
    let out = tomc()
        .args([
            "--target",
            "rust",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "--target rust");
    assert!(rs.exists(), "--target rust --emit code should produce a .rs file");
}

// ---------------------------------------------------------------------------
// Combined flags
// ---------------------------------------------------------------------------

#[test]
fn test_combined_verbose_check() {
    let out =
        tomc().args(["--verbose", "--check", hello_tomi().to_str().unwrap()]).output().unwrap();
    assert_success(&out, "--verbose --check");
    let combined = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        combined.contains("syntax check passed")
            || combined.contains("✓")
            || combined.contains("info:"),
        "verbose check output expected, got: {combined}"
    );
}

#[test]
fn test_combined_color_verbose_emit_code() {
    let tmp = tempfile::tempdir().unwrap();
    let rs = tmp.path().join("combined.rs");
    let out = tomc()
        .args([
            "--color",
            "never",
            "--verbose",
            "--emit",
            "code",
            "-o",
            rs.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "--color never --verbose --emit code");
    assert!(rs.exists());
}

#[test]
fn test_combined_lint_codegen_emit_bin() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("combined_bin");
    let out = tomc()
        .args([
            "-W",
            "unused-variables",
            "-D",
            "dead-code",
            "-A",
            "warnings",
            "-C",
            "opt-level=1",
            "-C",
            "overflow-checks=yes",
            "--emit",
            "bin",
            "-o",
            bin.to_str().unwrap(),
            hello_tomi().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_success(&out, "combined lints + codegen + emit bin");
    assert!(bin.exists(), "combined flags should still produce a binary");
}

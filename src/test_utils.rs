use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Copy)]
pub enum Backend {
    TreeWalk,
    Vm,
}

enum Expectation {
    Output(Vec<String>),
    CompileError,
    RuntimeError { output: Vec<String> },
}

fn parse_expectations(source: &str) -> Expectation {
    let mut output_lines = Vec::new();
    let mut has_runtime_error = false;
    let mut has_compile_error = false;

    for line in source.lines() {
        if let Some(idx) = line.find("// expect: ") {
            let value = &line[idx + "// expect: ".len()..];
            output_lines.push(value.to_string());
        } else if line.contains("// expect runtime error:") {
            has_runtime_error = true;
        } else if (line.contains("// [line") && line.contains("] Error"))
            || line.contains("// Error at")
        {
            has_compile_error = true;
        }
    }

    if has_compile_error {
        Expectation::CompileError
    } else if has_runtime_error {
        Expectation::RuntimeError {
            output: output_lines,
        }
    } else {
        Expectation::Output(output_lines)
    }
}

pub fn run_test(bin: &str, backend: Backend, path: &str) {
    // stringify!(r#if) produces "r#if", strip the prefix for file paths
    let path = path.replace("r#", "");
    let path = Path::new(&path);
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", path.display()));

    let expectation = parse_expectations(&source);

    let mut cmd = Command::new(bin);
    if matches!(backend, Backend::Vm) {
        cmd.arg("--vm");
    }
    cmd.arg(path);

    let output = cmd
        .output()
        .unwrap_or_else(|e| panic!("Failed to run rlox: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout_lines: Vec<&str> = if stdout.is_empty() {
        vec![]
    } else {
        stdout.trim_end_matches('\n').split('\n').collect()
    };

    match expectation {
        Expectation::Output(expected) => {
            assert!(
                output.status.success(),
                "Expected success for {}, got failure.\nstderr:\n{stderr}",
                path.display()
            );
            assert_eq!(
                stdout_lines,
                expected.iter().map(String::as_str).collect::<Vec<_>>(),
                "Output mismatch for {}",
                path.display()
            );
        }
        Expectation::CompileError => {
            assert!(
                !output.status.success(),
                "Expected compile error for {}, but got success.\nstdout:\n{stdout}",
                path.display()
            );
            assert!(
                !stderr.is_empty(),
                "Expected stderr output for compile error in {}",
                path.display()
            );
        }
        Expectation::RuntimeError { output: expected } => {
            assert!(
                !output.status.success(),
                "Expected runtime error for {}, but got success.\nstdout:\n{stdout}",
                path.display()
            );
            assert!(
                !stderr.is_empty(),
                "Expected stderr output for runtime error in {}",
                path.display()
            );
            if !expected.is_empty() {
                let actual_prefix = &stdout_lines[..stdout_lines.len().min(expected.len())];
                assert_eq!(
                    actual_prefix,
                    expected
                        .iter()
                        .map(String::as_str)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    "Output before runtime error mismatch for {}",
                    path.display()
                );
            }
        }
    }
}

pub fn run_examples(backend: Backend, dir: &str) {
    let examples: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("Failed to read directory {dir}: {e}"))
        .flatten()
        .filter(|f| f.file_name().to_string_lossy().ends_with(".lox"))
        .map(|f| f.path())
        .collect();

    assert!(!examples.is_empty(), "No .lox files found in {dir}");

    for path in &examples {
        let result = match backend {
            Backend::TreeWalk => rlox_tree_walk::run_file(path),
            Backend::Vm => rlox_vm::run_file(path),
        };
        result.unwrap_or_else(|e| panic!("{} failed: {e}", path.display()));
    }
}

#[macro_export]
macro_rules! lox_tests {
    ($category:literal, [$($(#[$attr:meta])* $name:ident),* $(,)?]) => {
        $(
            #[test]
            $(#[$attr])*
            fn $name() {
                $crate::test_utils::run_test(
                    env!("CARGO_BIN_EXE_rlox"),
                    super::BACKEND,
                    concat!("tests/sources/", $category, "/", stringify!($name), ".lox"),
                );
            }
        )*
    };
    ([$($(#[$attr:meta])* $name:ident),* $(,)?]) => {
        $(
            #[test]
            $(#[$attr])*
            fn $name() {
                $crate::test_utils::run_test(
                    env!("CARGO_BIN_EXE_rlox"),
                    super::BACKEND,
                    concat!("tests/sources/", stringify!($name), ".lox"),
                );
            }
        )*
    };
}

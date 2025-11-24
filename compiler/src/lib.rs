//! QED Compiler Library
//!
//! Provides compilation from .qed source to LLVM IR and executable binaries.
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(clippy::new_without_default)]

pub mod ast;
pub mod codegen;
pub mod ir;
pub mod parser;
pub mod types;

pub use ast::Program;
pub use codegen::CodeGen;
pub use parser::parse;
pub use types::TypeChecker;

use std::fs;
use std::path::Path;
use std::process::Command;

/// Compile a .qed source file to an executable
pub fn compile_file(source_path: &Path, output_path: &Path, keep_ir: bool) -> Result<(), String> {
    // Read source file
    let source = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read source file: {}", e))?;

    // Parse
    let program = parse(&source).map_err(|e| format!("Parse error: {}", e))?;

    // Type check
    let mut type_checker = TypeChecker::new();
    type_checker
        .check_program(&program)
        .map_err(|e| format!("Type error: {}", e))?;

    // Generate LLVM IR
    let mut codegen = CodeGen::new();
    let ir = codegen.codegen_program(&program)?;

    // Write IR to file
    let ir_path = output_path.with_extension("ll");
    fs::write(&ir_path, ir).map_err(|e| format!("Failed to write IR file: {}", e))?;

    // Validate runtime library exists
    let runtime_lib = Path::new("target/release/libqed_runtime.a");
    if !runtime_lib.exists() {
        return Err(format!(
            "Runtime library not found at {}. \
             Please run 'cargo build --release -p qed-runtime' first.",
            runtime_lib.display()
        ));
    }

    // Compile IR to executable using clang
    let output = Command::new("clang")
        .arg(&ir_path)
        .arg("-o")
        .arg(output_path)
        .arg("-L")
        .arg("target/release")
        .arg("-lqed_runtime")
        .output()
        .map_err(|e| format!("Failed to run clang: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Clang compilation failed:\n{}", stderr));
    }

    // Remove temporary IR file unless user wants to keep it
    if !keep_ir {
        fs::remove_file(&ir_path).ok();
    }

    Ok(())
}

/// Compile source string to LLVM IR string (for testing)
pub fn compile_to_ir(source: &str) -> Result<String, String> {
    let program = parse(source).map_err(|e| format!("Parse error: {}", e))?;

    let mut type_checker = TypeChecker::new();
    type_checker
        .check_program(&program)
        .map_err(|e| format!("Type error: {}", e))?;

    let mut codegen = CodeGen::new();
    codegen.codegen_program(&program)
}

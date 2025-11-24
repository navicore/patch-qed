//! LLVM IR Code Generation via Text
//!
//! Generates LLVM IR as text (.ll files) and invokes clang to produce executables.
//! This approach is simpler and more portable than using FFI bindings (inkwell).

use crate::ast::Program;
use std::fmt::Write as _;

pub struct CodeGen {
    output: String,
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            output: String::new(),
        }
    }

    /// Generate LLVM IR for a complete program
    pub fn codegen_program(&mut self, _program: &Program) -> Result<String, String> {
        // Start with runtime function declarations
        self.emit_runtime_declarations();

        // TODO: Generate code for types, relations, rules, queries

        // Generate main function
        self.emit_main();

        Ok(self.output.clone())
    }

    /// Emit declarations for runtime functions
    fn emit_runtime_declarations(&mut self) {
        writeln!(self.output, "; QED Runtime Function Declarations").unwrap();
        writeln!(self.output).unwrap();

        // Arena functions
        writeln!(
            self.output,
            "declare ptr @qed_arena_new(i64)  ; Create new arena"
        )
        .unwrap();
        writeln!(
            self.output,
            "declare void @qed_arena_free(ptr)  ; Free arena"
        )
        .unwrap();
        writeln!(
            self.output,
            "declare ptr @qed_arena_alloc(ptr, i64, i64)  ; Allocate in arena"
        )
        .unwrap();
        writeln!(self.output).unwrap();

        // Table functions
        writeln!(
            self.output,
            "declare ptr @qed_table_new(i64)  ; Create new table"
        )
        .unwrap();
        writeln!(
            self.output,
            "declare void @qed_table_free(ptr)  ; Free table"
        )
        .unwrap();
        writeln!(
            self.output,
            "declare void @qed_table_insert(ptr, i64, ptr, ptr)  ; Insert into table"
        )
        .unwrap();
        writeln!(self.output).unwrap();
    }

    /// Emit main function
    fn emit_main(&mut self) {
        writeln!(self.output, "; Main entry point").unwrap();
        writeln!(self.output, "define i32 @main() {{").unwrap();
        writeln!(self.output, "entry:").unwrap();

        // TODO: Initialize query context and execute queries
        writeln!(self.output, "  ; TODO: Initialize and execute queries").unwrap();

        writeln!(self.output, "  ret i32 0").unwrap();
        writeln!(self.output, "}}").unwrap();
    }
}

impl Default for CodeGen {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codegen_basic() {
        let mut codegen = CodeGen::new();
        let program = Program { items: vec![] };

        let ir = codegen.codegen_program(&program);
        assert!(ir.is_ok());

        let ir_text = ir.unwrap();
        assert!(ir_text.contains("@main"));
        assert!(ir_text.contains("@qed_arena_new"));
    }
}

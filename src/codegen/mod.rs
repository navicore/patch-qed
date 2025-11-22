/// LLVM IR code generation
///
/// This module compiles leem IR to LLVM IR using inkwell.
/// Requires the "llvm" feature to be enabled.

use crate::ir::*;
use anyhow::Result;

// When LLVM feature is enabled, use inkwell
#[cfg(feature = "llvm")]
mod llvm_backend {
    use super::*;
    use inkwell::context::Context;
    use inkwell::module::Module;
    use inkwell::builder::Builder;
    use inkwell::types::BasicTypeEnum;
    use inkwell::values::FunctionValue;
    use std::collections::HashMap;

    /// Code generator for LLVM
    pub struct CodeGen<'ctx> {
        context: &'ctx Context,
        module: Module<'ctx>,
        builder: Builder<'ctx>,
        llvm_types: HashMap<String, BasicTypeEnum<'ctx>>,
        functions: HashMap<String, FunctionValue<'ctx>>,
    }

    impl<'ctx> CodeGen<'ctx> {
        pub fn new(context: &'ctx Context, module_name: &str) -> Self {
            let module = context.create_module(module_name);
            let builder = context.create_builder();

            CodeGen {
                context,
                module,
                builder,
                llvm_types: HashMap::new(),
                functions: HashMap::new(),
            }
        }

        pub fn generate(&mut self, _program: &IrProgram) -> Result<()> {
            // TODO: Implement full code generation
            Ok(())
        }

        pub fn to_string(&self) -> String {
            self.module.print_to_string().to_string()
        }

        pub fn write_to_file(&self, path: &std::path::Path) -> Result<()> {
            self.module.print_to_file(path)
                .map_err(|e| anyhow::anyhow!("Failed to write LLVM IR: {}", e))
        }
    }
}

#[cfg(feature = "llvm")]
pub use llvm_backend::CodeGen;

// Stub implementation when LLVM is not available
#[cfg(not(feature = "llvm"))]
pub struct CodeGen;

#[cfg(not(feature = "llvm"))]
impl CodeGen {
    pub fn generate(_program: &IrProgram) -> Result<()> {
        Err(anyhow::anyhow!(
            "LLVM support not enabled. Rebuild with --features llvm"
        ))
    }
}

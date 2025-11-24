use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "qedc")]
#[command(about = "The qed logic programming language compiler", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a qed program to native code
    Compile {
        /// Input .qed file
        input: PathBuf,

        /// Output binary path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Emit LLVM IR instead of binary
        #[arg(long)]
        emit_llvm: bool,

        /// Optimization level (0-3)
        #[arg(short = 'O', default_value = "2")]
        opt_level: u8,
    },

    /// Type-check a qed program without compiling
    Check {
        /// Input .qed file
        input: PathBuf,
    },

    /// Show proof tree for a query
    Explain {
        /// Input .qed file
        input: PathBuf,

        /// Query to explain
        query: String,
    },

    /// Start interactive REPL
    Repl {
        /// Optional program to load
        input: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            input,
            output,
            emit_llvm,
            opt_level,
        } => {
            println!("Compiling {:?}...", input);
            compile_program(&input, output.as_ref(), emit_llvm, opt_level)?;
        }
        Commands::Check { input } => {
            println!("Checking {:?}...", input);
            check_program(&input)?;
        }
        Commands::Explain { input, query } => {
            println!("Explaining query '{}' in {:?}...", query, input);
            explain_query(&input, &query)?;
        }
        Commands::Repl { input } => {
            println!("Starting REPL...");
            start_repl(input.as_ref())?;
        }
    }

    Ok(())
}

fn compile_program(
    input: &PathBuf,
    output: Option<&PathBuf>,
    emit_llvm: bool,
    _opt_level: u8,
) -> Result<()> {
    let output_path = output.cloned().unwrap_or_else(|| input.with_extension(""));

    qedc::compile_file(input, &output_path, emit_llvm).map_err(|e| anyhow::anyhow!(e))?;

    println!("Successfully compiled {:?} -> {:?}", input, output_path);
    Ok(())
}

fn check_program(input: &PathBuf) -> Result<()> {
    // TODO: Implement type checking
    println!("Type checking not yet implemented");
    println!("  Input: {:?}", input);
    Ok(())
}

fn explain_query(input: &PathBuf, query: &str) -> Result<()> {
    // TODO: Implement query explanation
    println!("Query explanation not yet implemented");
    println!("  Input: {:?}", input);
    println!("  Query: {}", query);
    Ok(())
}

fn start_repl(input: Option<&PathBuf>) -> Result<()> {
    // TODO: Implement REPL
    println!("REPL not yet implemented");
    if let Some(path) = input {
        println!("  Loading: {:?}", path);
    }
    Ok(())
}

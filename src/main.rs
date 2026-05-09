//! gen-isa CLI.
//!
//! See `gen-isa --help` for usage.

use clap::{Parser, Subcommand};
use gen_isa::{RealFs, ScaffoldRequest, scaffold, scaffold::dry_run_paths};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "gen-isa",
    version,
    about = "ISA-specific Rust crate scaffolder"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Generate the five-crate skeleton for an ISA.
    Scaffold {
        /// Lowercase ISA slug (e.g. `ibm1130`, `cdp1802`, `riscv32i`).
        #[arg(long)]
        slug: String,
        /// Human-readable display name (e.g. `"IBM 1130"`).
        #[arg(long)]
        display_name: String,
        /// Rust type name in PascalCase (e.g. `Ibm1130`).
        #[arg(long)]
        type_name: String,
        /// Output directory; the five crate dirs are created as children.
        #[arg(long)]
        out: PathBuf,
        /// Path prefix for sw-langtools framework deps.
        #[arg(long, default_value = "../../sw-langtools")]
        framework_path: String,
        /// Print files that would be written; do not touch disk.
        #[arg(long)]
        dry_run: bool,
        /// Overwrite existing files.
        #[arg(long)]
        force: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Command::Scaffold {
            slug,
            display_name,
            type_name,
            out,
            framework_path,
            dry_run,
            force,
        } => {
            let req = ScaffoldRequest {
                slug,
                display_name,
                type_name,
                out_dir: out,
                framework_path,
                force,
            };
            if dry_run {
                match dry_run_paths(&req) {
                    Ok(paths) => {
                        for p in paths {
                            println!("{}", p.display());
                        }
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("error: {e}");
                        ExitCode::FAILURE
                    }
                }
            } else {
                let mut fs = RealFs::new();
                match scaffold(&req, &mut fs) {
                    Ok(paths) => {
                        for p in paths {
                            println!("wrote {}", p.display());
                        }
                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("error: {e}");
                        ExitCode::FAILURE
                    }
                }
            }
        }
    }
}

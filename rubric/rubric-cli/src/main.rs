mod runner;
mod config;

use clap::{Parser, Subcommand};
use anyhow::Result;
use rubric_core::Rule;
use rubric_rules::TrailingWhitespace;
use crate::config::Config;

#[derive(Parser)]
#[command(name = "rubric", version, about = "A fast Ruby linter and formatter")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lint Ruby files and report violations
    Check {
        /// Path to lint (file or directory)
        #[arg(default_value = ".")]
        path: std::path::PathBuf,

        /// Apply safe auto-fixes
        #[arg(long)]
        fix: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path, fix } => {
            if fix {
                eprintln!("Note: --fix is not yet implemented. Run `rubric check` without --fix.");
                return Ok(());
            }

            let config = Config::load(&std::env::current_dir()?)?;
            let _ = config; // will be used when rule registry is built

            let rules: Vec<Box<dyn Rule + Send + Sync>> = vec![
                Box::new(TrailingWhitespace),
                // M2 cops (TrailingNewlines, IndentationWidth, LineLength, EmptyLines,
                // SpaceAfterComma, SpaceBeforeComment, FrozenStringLiteralComment,
                // StringLiterals, TrailingCommaInArguments) will be wired in M3
                // when a rule registry and config-driven enabling/disabling is added.
            ];

            let files = runner::collect_ruby_files(&path);
            if files.is_empty() {
                println!("No Ruby files found.");
                return Ok(());
            }

            let mut results = runner::run_all_files(&files, &rules);
            // Sort by path for deterministic output
            results.sort_by(|a, b| a.0.cmp(&b.0));

            let mut total_violations = 0;
            for (file, diagnostics) in &results {
                let source = std::fs::read_to_string(file)?;
                let ctx = rubric_core::LintContext::new(file.as_path(), &source);
                for diag in diagnostics {
                    let (line, col) = ctx.offset_to_line_col(diag.range.start);
                    println!(
                        "{}:{}:{}: [{}] {} ({})",
                        file.display(),
                        line,
                        col,
                        format!("{:?}", diag.severity).to_uppercase(),
                        diag.message,
                        diag.rule
                    );
                }
                total_violations += diagnostics.len();
            }

            if total_violations > 0 {
                eprintln!("\n{} violation(s) found.", total_violations);
                std::process::exit(1);
            } else {
                println!("No violations found.");
            }
        }
    }

    Ok(())
}

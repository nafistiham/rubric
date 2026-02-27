mod runner;

use clap::{Parser, Subcommand};
use anyhow::Result;
use rubric_core::Rule;
use rubric_rules::TrailingWhitespace;

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
                eprintln!("Note: --fix is not yet implemented in this version. Run `rubric check` without --fix to see violations.");
                return Ok(());
            }

            let rules: Vec<Box<dyn Rule>> = vec![
                Box::new(TrailingWhitespace),
            ];

            let files = runner::collect_ruby_files(&path);

            if files.is_empty() {
                println!("No Ruby files found.");
                return Ok(());
            }

            let mut total_violations = 0;

            for file in &files {
                let source = std::fs::read_to_string(file)?;
                let ctx = rubric_core::LintContext::new(file, &source);
                let diagnostics = runner::run_rules_on_source(&ctx, &rules);
                for diag in &diagnostics {
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

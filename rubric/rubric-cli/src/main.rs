mod runner;
mod config;

use clap::{Parser, Subcommand};
use anyhow::Result;
use rubric_core::{Rule, FixSafety};
use rubric_rules::{
    TrailingWhitespace,
    TrailingNewlines, IndentationWidth, LineLength, EmptyLines,
    SpaceAfterComma, SpaceBeforeComment,
    FrozenStringLiteralComment, StringLiterals, TrailingCommaInArguments,
};
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
    /// Format Ruby files (apply all safe Layout and Style fixes)
    Fmt {
        /// Path to format (file or directory)
        #[arg(default_value = ".")]
        path: std::path::PathBuf,
    },
}

fn build_rules() -> Vec<Box<dyn Rule + Send + Sync>> {
    vec![
        Box::new(TrailingWhitespace),
        Box::new(TrailingNewlines),
        Box::new(IndentationWidth),
        Box::new(LineLength),
        Box::new(EmptyLines),
        Box::new(SpaceAfterComma),
        Box::new(SpaceBeforeComment),
        Box::new(FrozenStringLiteralComment),
        Box::new(StringLiterals),
        Box::new(TrailingCommaInArguments),
    ]
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path, fix } => {
            let config = Config::load(&std::env::current_dir()?)?;
            let _ = config; // will be used when rule registry is built

            let rules = build_rules();

            let files = runner::collect_ruby_files(&path);
            if files.is_empty() {
                println!("No Ruby files found.");
                return Ok(());
            }

            let mut results = runner::run_all_files(&files, &rules);
            // Sort by path for deterministic output
            results.sort_by(|a, b| a.0.cmp(&b.0));

            if fix {
                let mut total_fixed = 0;
                for (file, source, diagnostics) in &results {
                    let fixes: Vec<_> = diagnostics.iter()
                        .filter_map(|d| {
                            rules.iter().find(|r| r.name() == d.rule)?.fix(d)
                        })
                        .collect();
                    if !fixes.is_empty() {
                        let corrected = rubric_core::apply_fixes(source, &fixes);
                        std::fs::write(file, corrected)?;
                        println!("{}: fixed {} violation(s)", file.display(), fixes.len());
                        total_fixed += fixes.len();
                    }
                }
                if total_fixed == 0 {
                    println!("No violations to fix.");
                }
            } else {
                let mut total_violations = 0;
                for (file, source, diagnostics) in &results {
                    let ctx = rubric_core::LintContext::new(file.as_path(), source);
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

        Commands::Fmt { path } => {
            let config = Config::load(&std::env::current_dir()?)?;
            let _ = config;

            let rules = build_rules();

            let files = runner::collect_ruby_files(&path);
            if files.is_empty() {
                println!("No Ruby files found.");
                return Ok(());
            }

            let mut results = runner::run_all_files(&files, &rules);
            results.sort_by(|a, b| a.0.cmp(&b.0));

            let mut total_fixed = 0;
            for (file, source, diagnostics) in &results {
                // fmt only applies Safe fixes
                let fixes: Vec<_> = diagnostics.iter()
                    .filter_map(|d| {
                        let fix = rules.iter().find(|r| r.name() == d.rule)?.fix(d)?;
                        if fix.safety == FixSafety::Safe {
                            Some(fix)
                        } else {
                            None
                        }
                    })
                    .collect();
                if !fixes.is_empty() {
                    let corrected = rubric_core::apply_fixes(source, &fixes);
                    std::fs::write(file, corrected)?;
                    println!("{}: fixed {} violation(s)", file.display(), fixes.len());
                    total_fixed += fixes.len();
                }
            }
            if total_fixed == 0 {
                println!("No violations to fix.");
            }
        }
    }

    Ok(())
}

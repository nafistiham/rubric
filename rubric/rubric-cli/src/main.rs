mod runner;
mod config;
mod commands;

use clap::{Parser, Subcommand};
use anyhow::Result;
use rubric_core::{Rule, FixSafety};
use rubric_rules::{
    TrailingWhitespace,
    TrailingNewlines, IndentationWidth, LineLength, EmptyLines,
    SpaceAfterComma, SpaceBeforeComment, SpaceAroundOperators, SpaceInsideParens,
    SpaceInsideArrayLiteralBrackets, SpaceInsideHashLiteralBraces,
    MultilineMethodCallIndentation, HashAlignment, ClosingParenthesisIndentation,
    LeadingCommentSpace, SpaceAroundBlockParameters, FirstHashElementIndentation,
    EmptyLinesAroundClassBody, EmptyLinesAroundModuleBody, EmptyLinesAroundMethodBody,
    EmptyLineBetweenDefs, ExtraSpacing, SpaceAfterMethodName, SpaceAfterColon,
    SpaceAroundKeyword, RescueEnsureAlignment, EndAlignment, CaseIndentation,
    IndentationConsistency, SpaceInsideStringInterpolation, SpaceBeforeBlockBraces,
    MultilineOperationIndentation,
    FrozenStringLiteralComment, StringLiterals, TrailingCommaInArguments, HashSyntax, SymbolArray,
    WordArray, NegatedIf, UnlessElse, RedundantReturn, GuardClause, SafeNavigation,
    BlockDelimiters, Lambda, ProcNew, TrailingCommaInArrayLiteral, SymbolProc,
    OptionalArguments, MutableConstant, IfUnlessModifier, WhileUntilModifier, WhileUntilDo,
    AndOr, NotKeyword, RedundantBegin, RedundantSelf, TernaryParentheses,
    ZeroLengthPredicate, YodaCondition, RaiseArgs, SignalException, StderrPuts,
    UselessAssignment, UnusedMethodArgument,
    AmbiguousOperator, AmbiguousBlockAssociation, AssignmentInCondition, DuplicateHashKey,
    EmptyBlock, EmptyExpression, FloatOutOfRange, SuppressedException, UselessComparison,
    UnreachableCode,
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
    /// Convert .rubocop.yml to rubric.toml
    Migrate {
        /// Path to .rubocop.yml (default: .rubocop.yml)
        #[arg(default_value = ".rubocop.yml")]
        input: std::path::PathBuf,
        /// Output path (default: rubric.toml)
        #[arg(long, default_value = "rubric.toml")]
        output: std::path::PathBuf,
    },
}

/// Apply all FixSafety::Safe fixes from `results` to disk.
/// Returns the number of files that had at least one fix applied.
fn apply_safe_fixes(
    results: &[(std::path::PathBuf, String, Vec<rubric_core::Diagnostic>)],
    rules: &[Box<dyn Rule + Send + Sync>],
) -> anyhow::Result<usize> {
    let mut total_fixed = 0;
    for (file, source, diagnostics) in results {
        let fixes: Vec<_> = diagnostics
            .iter()
            .filter_map(|d| {
                let fix = rules.iter().find(|r| r.name() == d.rule)?.fix(d)?;
                if fix.safety == FixSafety::Safe { Some(fix) } else { None }
            })
            .collect();
        if !fixes.is_empty() {
            let corrected = rubric_core::apply_fixes(source, &fixes);
            std::fs::write(file, corrected)?;
            println!("{}: fixed {} violation(s)", file.display(), fixes.len());
            total_fixed += 1;
        }
    }
    Ok(total_fixed)
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
        Box::new(SpaceAroundOperators),
        Box::new(SpaceInsideParens),
        Box::new(SpaceInsideArrayLiteralBrackets),
        Box::new(SpaceInsideHashLiteralBraces),
        Box::new(MultilineMethodCallIndentation),
        Box::new(HashAlignment),
        Box::new(ClosingParenthesisIndentation),
        Box::new(LeadingCommentSpace),
        Box::new(SpaceAroundBlockParameters),
        Box::new(FirstHashElementIndentation),
        Box::new(EmptyLinesAroundClassBody),
        Box::new(EmptyLinesAroundModuleBody),
        Box::new(EmptyLinesAroundMethodBody),
        Box::new(EmptyLineBetweenDefs),
        Box::new(ExtraSpacing),
        Box::new(SpaceAfterMethodName),
        Box::new(SpaceAfterColon),
        Box::new(SpaceAroundKeyword),
        Box::new(RescueEnsureAlignment),
        Box::new(EndAlignment),
        Box::new(CaseIndentation),
        Box::new(IndentationConsistency),
        Box::new(SpaceInsideStringInterpolation),
        Box::new(SpaceBeforeBlockBraces),
        Box::new(MultilineOperationIndentation),
        Box::new(FrozenStringLiteralComment),
        Box::new(StringLiterals),
        Box::new(TrailingCommaInArguments),
        Box::new(HashSyntax),
        Box::new(SymbolArray),
        Box::new(WordArray),
        Box::new(NegatedIf),
        Box::new(UnlessElse),
        Box::new(RedundantReturn),
        Box::new(GuardClause),
        Box::new(SafeNavigation),
        Box::new(BlockDelimiters),
        Box::new(Lambda),
        Box::new(ProcNew),
        Box::new(TrailingCommaInArrayLiteral),
        Box::new(SymbolProc),
        Box::new(OptionalArguments),
        Box::new(MutableConstant),
        Box::new(IfUnlessModifier),
        Box::new(WhileUntilModifier),
        Box::new(WhileUntilDo),
        Box::new(AndOr),
        Box::new(NotKeyword),
        Box::new(RedundantBegin),
        Box::new(RedundantSelf),
        Box::new(TernaryParentheses),
        Box::new(ZeroLengthPredicate),
        Box::new(YodaCondition),
        Box::new(RaiseArgs),
        Box::new(SignalException),
        Box::new(StderrPuts),
        Box::new(UselessAssignment),
        Box::new(UnusedMethodArgument),
        Box::new(AmbiguousOperator),
        Box::new(AmbiguousBlockAssociation),
        Box::new(AssignmentInCondition),
        Box::new(DuplicateHashKey),
        Box::new(EmptyBlock),
        Box::new(EmptyExpression),
        Box::new(FloatOutOfRange),
        Box::new(SuppressedException),
        Box::new(UselessComparison),
        Box::new(UnreachableCode),
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
                let total_fixed = apply_safe_fixes(&results, &rules)?;
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

        Commands::Migrate { input, output } => {
            commands::migrate::run(&input, &output)?;
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

            let total_fixed = apply_safe_fixes(&results, &rules)?;
            if total_fixed == 0 {
                println!("No violations to fix.");
            }
        }
    }

    Ok(())
}

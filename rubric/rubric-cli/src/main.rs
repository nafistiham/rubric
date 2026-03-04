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
    EndOfLine, EmptyLinesAroundBlockBody, SpaceAroundEqualsInParameterDefault,
    SpaceInLambdaLiteral, SpaceInsideBlockBraces, SpaceInsideRangeLiteral,
    SpaceInsideReferenceBrackets, FirstArgumentIndentation, FirstArrayElementIndentation,
    FirstParameterIndentation, MultilineArrayBraceLayout, MultilineHashBraceLayout,
    MultilineMethodCallBraceLayout, MultilineMethodDefinitionBraceLayout, BlockAlignment,
    ConditionPosition, DefEndAlignment, ElseAlignment, HeredocIndentation,
    IndentationStyle, SpaceBeforeSemicolon,
    FrozenStringLiteralComment, StringLiterals, TrailingCommaInArguments, HashSyntax, SymbolArray,
    WordArray, NegatedIf, UnlessElse, RedundantReturn, GuardClause, SafeNavigation,
    BlockDelimiters, Lambda, ProcNew, TrailingCommaInArrayLiteral, SymbolProc,
    OptionalArguments, MutableConstant, IfUnlessModifier, WhileUntilModifier, WhileUntilDo,
    AndOr, NotKeyword, RedundantBegin, RedundantSelf, TernaryParentheses,
    ZeroLengthPredicate, YodaCondition, RaiseArgs, SignalException, StderrPuts,
    ClassAndModuleChildren, Documentation, EmptyMethod, SingleLineMethods,
    AccessModifierDeclarations, ConditionalAssignment, NegatedWhile, PercentLiteralDelimiters,
    PreferredHashMethods, ReturnNil, Send as SendCop, StringConcatenation, StructInheritance,
    TrailingUnderscoreVariable, ClassMethods, ModuleFunction, ParallelAssignment,
    RedundantCondition,
    UselessAssignment, UnusedMethodArgument,
    AmbiguousOperator, AmbiguousBlockAssociation, AssignmentInCondition, DuplicateHashKey,
    EmptyBlock, EmptyExpression, FloatOutOfRange, SuppressedException, UselessComparison,
    UnreachableCode,
    UnusedBlockArgument, UselessSetterCall, AmbiguousRegexpLiteral, BigDecimalNew,
    BooleanSymbol, CircularArgumentReference, ConstantDefinitionInBlock, DeprecatedClassMethods,
    DuplicateBranch, DuplicateMethods, DuplicateRequire, EmptyConditionalBody, EmptyEnsure,
    EmptyInterpolation, EnsureReturn, FlipFlop, FormatParameterMismatch,
    ImplicitStringConcatenation, IneffectiveAccessModifier, MultipleComparison,
    NestedMethodDefinition, NoReturnInBeginEndBlock, NonLocalExitFromIterator,
    OrderedMagicComments, ParenthesesAsGroupedExpression, RaiseException, RandOne,
    RedundantSplatExpansion, SelfAssignment, ShadowingOuterLocalVariable, StructNewOverride,
    TopLevelReturnWithArgument, UnderscorePrefixedVariableName, UriEscapeUnescape,
    UselessElseWithoutRescue, Void,
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

fn build_rules_with_config(config: &Config) -> Vec<Box<dyn Rule + Send + Sync>> {
    build_rules()
        .into_iter()
        .filter(|r| {
            // If the rule is explicitly listed in the config, that wins.
            if let Some(rule_cfg) = config.rules.get(r.name()) {
                return rule_cfg.enabled;
            }
            // When disabled_by_default is set, unlisted rules are all off.
            if config.linter.disabled_by_default {
                return false;
            }
            // Otherwise fall back to the rule's own default (allows cops that
            // RuboCop ships disabled to stay off unless the user opts in).
            r.default_enabled()
        })
        .collect()
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
        // New Layout cops
        Box::new(EndOfLine),
        Box::new(EmptyLinesAroundBlockBody),
        Box::new(SpaceAroundEqualsInParameterDefault),
        Box::new(SpaceInLambdaLiteral),
        Box::new(SpaceInsideBlockBraces),
        Box::new(SpaceInsideRangeLiteral),
        Box::new(SpaceInsideReferenceBrackets),
        Box::new(FirstArgumentIndentation),
        Box::new(FirstArrayElementIndentation),
        Box::new(FirstParameterIndentation),
        Box::new(MultilineArrayBraceLayout),
        Box::new(MultilineHashBraceLayout),
        Box::new(MultilineMethodCallBraceLayout),
        Box::new(MultilineMethodDefinitionBraceLayout),
        Box::new(BlockAlignment),
        Box::new(ConditionPosition),
        Box::new(DefEndAlignment),
        Box::new(ElseAlignment),
        Box::new(HeredocIndentation),
        Box::new(IndentationStyle),
        Box::new(SpaceBeforeSemicolon),
        // New Style cops
        Box::new(ClassAndModuleChildren),
        Box::new(Documentation),
        Box::new(EmptyMethod),
        Box::new(SingleLineMethods),
        Box::new(AccessModifierDeclarations),
        Box::new(ConditionalAssignment),
        Box::new(NegatedWhile),
        Box::new(PercentLiteralDelimiters),
        Box::new(PreferredHashMethods),
        Box::new(ReturnNil),
        Box::new(SendCop),
        Box::new(StringConcatenation),
        Box::new(StructInheritance),
        Box::new(TrailingUnderscoreVariable),
        Box::new(ClassMethods),
        Box::new(ModuleFunction),
        Box::new(ParallelAssignment),
        Box::new(RedundantCondition),
        // New Lint cops
        Box::new(UnusedBlockArgument),
        Box::new(UselessSetterCall),
        Box::new(AmbiguousRegexpLiteral),
        Box::new(BigDecimalNew),
        Box::new(BooleanSymbol),
        Box::new(CircularArgumentReference),
        Box::new(ConstantDefinitionInBlock),
        Box::new(DeprecatedClassMethods),
        Box::new(DuplicateBranch),
        Box::new(DuplicateMethods),
        Box::new(DuplicateRequire),
        Box::new(EmptyConditionalBody),
        Box::new(EmptyEnsure),
        Box::new(EmptyInterpolation),
        Box::new(EnsureReturn),
        Box::new(FlipFlop),
        Box::new(FormatParameterMismatch),
        Box::new(ImplicitStringConcatenation),
        Box::new(IneffectiveAccessModifier),
        Box::new(MultipleComparison),
        Box::new(NestedMethodDefinition),
        Box::new(NoReturnInBeginEndBlock),
        Box::new(NonLocalExitFromIterator),
        Box::new(OrderedMagicComments),
        Box::new(ParenthesesAsGroupedExpression),
        Box::new(RaiseException),
        Box::new(RandOne),
        Box::new(RedundantSplatExpansion),
        Box::new(SelfAssignment),
        Box::new(ShadowingOuterLocalVariable),
        Box::new(StructNewOverride),
        Box::new(TopLevelReturnWithArgument),
        Box::new(UnderscorePrefixedVariableName),
        Box::new(UriEscapeUnescape),
        Box::new(UselessElseWithoutRescue),
        Box::new(Void),
    ]
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Check { path, fix } => {
            let config_dir = if path.is_dir() {
                path.clone()
            } else {
                path.parent().unwrap_or(&path).to_path_buf()
            };
            let config = Config::load(&config_dir)
                .or_else(|_| Config::load(&std::env::current_dir()?))?;

            let rules = build_rules_with_config(&config);

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

            let rules = build_rules_with_config(&config);

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

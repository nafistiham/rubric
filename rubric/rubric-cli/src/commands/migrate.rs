//! `rubric migrate` — converts `.rubocop.yml` to `rubric.toml`.

use anyhow::{Context, Result};
use std::path::Path;

pub(crate) const KNOWN_COPS: &[&str] = &[
    "Layout/TrailingWhitespace",
    "Layout/TrailingNewlines",
    "Layout/IndentationWidth",
    "Layout/LineLength",
    "Layout/EmptyLines",
    "Layout/SpaceAfterComma",
    "Layout/SpaceBeforeComment",
    "Layout/SpaceAroundOperators",
    "Layout/SpaceInsideParens",
    "Layout/SpaceInsideArrayLiteralBrackets",
    "Layout/SpaceInsideHashLiteralBraces",
    "Layout/MultilineMethodCallIndentation",
    "Layout/HashAlignment",
    "Layout/ClosingParenthesisIndentation",
    "Layout/LeadingCommentSpace",
    "Layout/SpaceAroundBlockParameters",
    "Layout/FirstHashElementIndentation",
    "Layout/EmptyLinesAroundClassBody",
    "Layout/EmptyLinesAroundModuleBody",
    "Layout/EmptyLinesAroundMethodBody",
    "Layout/EmptyLineBetweenDefs",
    "Layout/ExtraSpacing",
    "Layout/SpaceAfterMethodName",
    "Layout/SpaceAfterColon",
    "Layout/SpaceAroundKeyword",
    "Layout/RescueEnsureAlignment",
    "Layout/EndAlignment",
    "Layout/CaseIndentation",
    "Layout/IndentationConsistency",
    "Layout/SpaceInsideStringInterpolation",
    "Layout/SpaceBeforeBlockBraces",
    "Layout/MultilineOperationIndentation",
    "Style/FrozenStringLiteralComment",
    "Style/StringLiterals",
    "Style/TrailingCommaInArguments",
    "Style/HashSyntax",
    "Style/SymbolArray",
    "Style/WordArray",
    "Style/NegatedIf",
    "Style/UnlessElse",
    "Style/RedundantReturn",
    "Style/GuardClause",
    "Style/SafeNavigation",
    "Style/BlockDelimiters",
    "Style/Lambda",
    "Style/Proc",
    "Style/TrailingCommaInArrayLiteral",
    "Style/SymbolProc",
    "Style/OptionalArguments",
    "Style/MutableConstant",
    "Style/IfUnlessModifier",
    "Style/WhileUntilModifier",
    "Style/WhileUntilDo",
    "Style/AndOr",
    "Style/Not",
    "Style/RedundantBegin",
    "Style/RedundantSelf",
    "Style/TernaryParentheses",
    "Style/ZeroLengthPredicate",
    "Style/YodaCondition",
    "Style/RaiseArgs",
    "Style/SignalException",
    "Style/StderrPuts",
    "Lint/UselessAssignment",
    "Lint/UnusedMethodArgument",
    "Lint/AmbiguousOperator",
    "Lint/AmbiguousBlockAssociation",
    "Lint/AssignmentInCondition",
    "Lint/DuplicateHashKey",
    "Lint/EmptyBlock",
    "Lint/EmptyExpression",
    "Lint/FloatOutOfRange",
    "Lint/SuppressedException",
    "Lint/UselessComparison",
    "Lint/UnreachableCode",
    // New Layout cops
    "Layout/EndOfLine",
    "Layout/EmptyLinesAroundBlockBody",
    "Layout/SpaceAroundEqualsInParameterDefault",
    "Layout/SpaceInLambdaLiteral",
    "Layout/SpaceInsideBlockBraces",
    "Layout/SpaceInsideRangeLiteral",
    "Layout/SpaceInsideReferenceBrackets",
    "Layout/FirstArgumentIndentation",
    "Layout/FirstArrayElementIndentation",
    "Layout/FirstParameterIndentation",
    "Layout/MultilineArrayBraceLayout",
    "Layout/MultilineHashBraceLayout",
    "Layout/MultilineMethodCallBraceLayout",
    "Layout/MultilineMethodDefinitionBraceLayout",
    "Layout/BlockAlignment",
    "Layout/ConditionPosition",
    "Layout/DefEndAlignment",
    "Layout/ElseAlignment",
    "Layout/HeredocIndentation",
    "Layout/IndentationStyle",
    "Layout/SpaceBeforeSemicolon",
    // New Style cops
    "Style/ClassAndModuleChildren",
    "Style/Documentation",
    "Style/EmptyMethod",
    "Style/SingleLineMethods",
    "Style/AccessModifierDeclarations",
    "Style/ConditionalAssignment",
    "Style/NegatedWhile",
    "Style/PercentLiteralDelimiters",
    "Style/PreferredHashMethods",
    "Style/ReturnNil",
    "Style/Send",
    "Style/StringConcatenation",
    "Style/StructInheritance",
    "Style/TrailingUnderscoreVariable",
    "Style/ClassMethods",
    "Style/ModuleFunction",
    "Style/ParallelAssignment",
    "Style/RedundantCondition",
    // New Lint cops
    "Lint/UnusedBlockArgument",
    "Lint/UselessSetterCall",
    "Lint/AmbiguousRegexpLiteral",
    "Lint/BigDecimalNew",
    "Lint/BooleanSymbol",
    "Lint/CircularArgumentReference",
    "Lint/ConstantDefinitionInBlock",
    "Lint/DeprecatedClassMethods",
    "Lint/DuplicateBranch",
    "Lint/DuplicateMethods",
    "Lint/DuplicateRequire",
    "Lint/EmptyConditionalBody",
    "Lint/EmptyEnsure",
    "Lint/EmptyInterpolation",
    "Lint/EnsureReturn",
    "Lint/FlipFlop",
    "Lint/FormatParameterMismatch",
    "Lint/ImplicitStringConcatenation",
    "Lint/IneffectiveAccessModifier",
    "Lint/MultipleComparison",
    "Lint/NestedMethodDefinition",
    "Lint/NoReturnInBeginEndBlock",
    "Lint/NonLocalExitFromIterator",
    "Lint/OrderedMagicComments",
    "Lint/ParenthesesAsGroupedExpression",
    "Lint/RaiseException",
    "Lint/RandOne",
    "Lint/RedundantSplatExpansion",
    "Lint/SelfAssignment",
    "Lint/ShadowingOuterLocalVariable",
    "Lint/StructNewOverride",
    "Lint/TopLevelReturnWithArgument",
    "Lint/UnderscorePrefixedVariableName",
    "Lint/UriEscapeUnescape",
    "Lint/UselessElseWithoutRescue",
    "Lint/Void",
];

pub fn run(rubocop_path: &Path, output_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(rubocop_path)
        .with_context(|| format!("Could not read {}", rubocop_path.display()))?;

    let yaml: serde_yaml::Value = serde_yaml::from_str(&content)
        .with_context(|| "Failed to parse .rubocop.yml")?;

    let mapping = yaml.as_mapping()
        .ok_or_else(|| anyhow::anyhow!(
            "{} is not a valid .rubocop.yml: expected a YAML mapping at the top level",
            rubocop_path.display()
        ))?;

    let mut known_lines = Vec::<String>::new();
    let mut unknown_lines = Vec::<String>::new();

    for (key, value) in mapping {
        let cop_name = match key.as_str() {
            Some(s) => s,
            None => continue,
        };

        // Skip top-level Rubocop config keys (not cops)
        if matches!(cop_name, "AllCops" | "inherit_from" | "require" | "inherit_gem") {
            continue;
        }

        if KNOWN_COPS.contains(&cop_name) {
            let enabled = value
                .get("Enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(true); // RuboCop defaults to enabled when key is absent
            known_lines.push(format!(
                "[rules.\"{cop_name}\"]\nenabled = {enabled}"
            ));
        } else {
            unknown_lines.push(format!(
                "# UNKNOWN: {cop_name} (not yet implemented in Rubric)"
            ));
        }
    }

    // Build the output TOML
    let mut output = String::new();
    output.push_str("# Generated by `rubric migrate` from .rubocop.yml\n\n");
    output.push_str("[linter]\nenabled = true\n\n");
    output.push_str("[formatter]\nenabled = true\n\n");

    if !known_lines.is_empty() {
        output.push_str("# Migrated cops\n");
        for line in &known_lines {
            output.push_str(line);
            output.push_str("\n\n");
        }
    }

    if !unknown_lines.is_empty() {
        output.push_str("# Cops not yet implemented in Rubric:\n");
        for line in &unknown_lines {
            output.push_str(line);
            output.push('\n');
        }
    }

    std::fs::write(output_path, &output)
        .with_context(|| format!("Could not write to {}", output_path.display()))?;
    println!("Written to {}", output_path.display());
    println!("  {} cops migrated", known_lines.len());
    println!(
        "  {} cops not yet implemented (commented out)",
        unknown_lines.len()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn migrates_known_cops() {
        let dir = TempDir::new().unwrap();
        let rubocop_yml = dir.path().join(".rubocop.yml");
        let rubric_toml = dir.path().join("rubric.toml");

        fs::write(
            &rubocop_yml,
            r#"
Layout/TrailingWhitespace:
  Enabled: true

Layout/LineLength:
  Enabled: false
  Max: 120

Style/StringLiterals:
  Enabled: true
  EnforcedStyle: single_quotes
"#,
        )
        .unwrap();

        run(&rubocop_yml, &rubric_toml).unwrap();

        let output = fs::read_to_string(&rubric_toml).unwrap();
        // More specific: assert the full cop block is present
        assert!(output.contains("[rules.\"Layout/TrailingWhitespace\"]\nenabled = true"),
            "expected TrailingWhitespace block with enabled = true, got:\n{output}");
        assert!(output.contains("[rules.\"Layout/LineLength\"]\nenabled = false"),
            "expected LineLength block with enabled = false, got:\n{output}");
        assert!(output.contains("[rules.\"Style/StringLiterals\"]\nenabled = true"),
            "expected StringLiterals block with enabled = true, got:\n{output}");
    }

    #[test]
    fn unknown_cops_become_comments() {
        let dir = TempDir::new().unwrap();
        let rubocop_yml = dir.path().join(".rubocop.yml");
        let rubric_toml = dir.path().join("rubric.toml");

        fs::write(
            &rubocop_yml,
            r#"
Metrics/MethodLength:
  Enabled: true
  Max: 10
"#,
        )
        .unwrap();

        run(&rubocop_yml, &rubric_toml).unwrap();

        let output = fs::read_to_string(&rubric_toml).unwrap();
        assert!(output.contains("# UNKNOWN: Metrics/MethodLength"));
        assert!(!output.contains("[rules.\"Metrics/MethodLength\"]"));
    }

    #[test]
    fn allcops_key_is_skipped() {
        let dir = TempDir::new().unwrap();
        let rubocop_yml = dir.path().join(".rubocop.yml");
        let rubric_toml = dir.path().join("rubric.toml");

        fs::write(
            &rubocop_yml,
            r#"
AllCops:
  TargetRubyVersion: 3.2
  Exclude:
    - 'vendor/**'

Layout/TrailingWhitespace:
  Enabled: true
"#,
        )
        .unwrap();

        run(&rubocop_yml, &rubric_toml).unwrap();

        let output = fs::read_to_string(&rubric_toml).unwrap();
        assert!(!output.contains("AllCops"));
        assert!(output.contains("[rules.\"Layout/TrailingWhitespace\"]"));
    }
}

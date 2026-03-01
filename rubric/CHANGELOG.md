# Changelog

## [0.1.0] - 2026-03-02

### Added

- 150 cops across Style (49), Layout (53), and Lint (48) departments
- `rubric check` — lint Ruby files with parallel processing (Rayon)
- `rubric check --fix` — auto-apply safe fixes
- `rubric fmt` — format files (all safe Layout/Style fixes)
- `rubric migrate` — convert `.rubocop.yml` to `rubric.toml`
- `rubric.toml` configuration file support
- Criterion benchmark suite
- Gem packaging with cross-compiled platform gem support (x86_64-linux, aarch64-linux, x86_64-darwin, arm64-darwin)
- GitHub Actions release workflow for automated cross-platform builds and RubyGems publish

### Departments

**Style (49 cops):** FrozenStringLiteralComment, StringLiterals, TrailingCommaInArguments, TrailingCommaInArrayLiteral, HashSyntax, SymbolArray, WordArray, Lambda, Proc, BlockDelimiters, GuardClause, IfUnlessModifier, NegatedIf, UnlessElse, RedundantReturn, RedundantSelf, RedundantBegin, SafeNavigation, TernaryParentheses, ZeroLengthPredicate, AndOr, Not, RaiseArgs, SignalException, StderrPuts, SymbolProc, OptionalArguments, MutableConstant, WhileUntilModifier, WhileUntilDo, YodaCondition, ClassAndModuleChildren, Documentation, EmptyMethod, SingleLineMethods, AccessModifierDeclarations, ConditionalAssignment, NegatedWhile, PercentLiteralDelimiters, PreferredHashMethods, ReturnNil, Send, StringConcatenation, StructInheritance, TrailingUnderscoreVariable, ClassMethods, ModuleFunction, ParallelAssignment, RedundantCondition

**Layout (53 cops):** TrailingWhitespace, TrailingNewlines, IndentationWidth, LineLength, EmptyLines, SpaceAfterComma, SpaceBeforeComment, SpaceAroundOperators, SpaceInsideParens, SpaceInsideArrayLiteralBrackets, SpaceInsideHashLiteralBraces, MultilineMethodCallIndentation, HashAlignment, ClosingParenthesisIndentation, LeadingCommentSpace, SpaceAroundBlockParameters, FirstHashElementIndentation, EmptyLinesAroundClassBody, EmptyLinesAroundModuleBody, EmptyLinesAroundMethodBody, EmptyLineBetweenDefs, ExtraSpacing, SpaceAfterMethodName, SpaceAfterColon, SpaceAroundKeyword, RescueEnsureAlignment, EndAlignment, CaseIndentation, IndentationConsistency, SpaceInsideStringInterpolation, SpaceBeforeBlockBraces, MultilineOperationIndentation, EndOfLine, EmptyLinesAroundBlockBody, SpaceAroundEqualsInParameterDefault, SpaceInLambdaLiteral, SpaceInsideBlockBraces, SpaceInsideRangeLiteral, SpaceInsideReferenceBrackets, FirstArgumentIndentation, FirstArrayElementIndentation, FirstParameterIndentation, MultilineArrayBraceLayout, MultilineHashBraceLayout, MultilineMethodCallBraceLayout, MultilineMethodDefinitionBraceLayout, BlockAlignment, ConditionPosition, DefEndAlignment, ElseAlignment, HeredocIndentation, IndentationStyle, SpaceBeforeSemicolon

**Lint (48 cops):** UselessAssignment, UnusedMethodArgument, AmbiguousOperator, AmbiguousBlockAssociation, AssignmentInCondition, DuplicateHashKey, EmptyBlock, EmptyExpression, FloatOutOfRange, SuppressedException, UselessComparison, UnreachableCode, UnusedBlockArgument, UselessSetterCall, AmbiguousRegexpLiteral, BigDecimalNew, BooleanSymbol, CircularArgumentReference, ConstantDefinitionInBlock, DeprecatedClassMethods, DuplicateBranch, DuplicateMethods, DuplicateRequire, EmptyConditionalBody, EmptyEnsure, EmptyInterpolation, EnsureReturn, FlipFlop, FormatParameterMismatch, ImplicitStringConcatenation, IneffectiveAccessModifier, MultipleComparison, NestedMethodDefinition, NoReturnInBeginEndBlock, NonLocalExitFromIterator, OrderedMagicComments, ParenthesesAsGroupedExpression, RaiseException, RandOne, RedundantSplatExpansion, SelfAssignment, ShadowingOuterLocalVariable, StructNewOverride, TopLevelReturnWithArgument, UnderscorePrefixedVariableName, UriEscapeUnescape, UselessElseWithoutRescue, Void

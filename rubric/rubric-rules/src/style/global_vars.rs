use rubric_core::{Diagnostic, LintContext, Rule, Severity, TextRange};

pub struct GlobalVars;

/// Built-in Ruby globals that should not be flagged.
const BUILTIN_GLOBALS: &[&str] = &[
    "$0",
    "$&",
    "$'",
    "$`",
    "$+",
    "$~",
    "$=",
    "$!",
    "$@",
    "$/",
    "$\\",
    "$,",
    "$;",
    "$<",
    "$>",
    "$.",
    "$_",
    "$$",
    "$?",
    "$:",
    "$\"",
    "$LOAD_PATH",
    "$LOADED_FEATURES",
    "$PROGRAM_NAME",
    "$stderr",
    "$stdout",
    "$stdin",
    "$VERBOSE",
    "$DEBUG",
    "$-v",
    "$-w",
    "$-d",
    "$-i",
    "$-p",
    "$-l",
    "$-a",
    "$F",
    "$*",
    "$SAFE",
    // English-named aliases (from `require 'english'`) — RuboCop does not flag these
    "$LAST_MATCH_INFO",
    "$MATCH",
    "$PREMATCH",
    "$POSTMATCH",
    "$LAST_PAREN_MATCH",
    "$INPUT_LINE_NUMBER",
    "$NR",
    "$INPUT_RECORD_SEPARATOR",
    "$RS",
    "$OUTPUT_RECORD_SEPARATOR",
    "$ORS",
    "$OUTPUT_FIELD_SEPARATOR",
    "$OFS",
    "$INPUT_SEPARATOR",
    "$FS",
    "$OUTPUT_AUTO_FLUSH",
    "$LAST_READ_LINE",
    "$DEFAULT_OUTPUT",
    "$DEFAULT_INPUT",
    "$PID",
    "$PROCESS_ID",
    "$CHILD_STATUS",
    "$LAST_EXIT_STATUS",
    "$IGNORECASE",
    "$FILENAME",
    "$ARGV",
    // Perl-style backrefs $1-$9 are handled by PerlBackrefs; skip them here too
];

fn is_builtin(name: &str) -> bool {
    // Single-digit globals like $1..$9 are PerlBackrefs, not our concern
    if name.len() == 2 && name.as_bytes()[1].is_ascii_digit() {
        return true;
    }
    BUILTIN_GLOBALS.contains(&name)
}

impl Rule for GlobalVars {
    fn name(&self) -> &'static str {
        "Style/GlobalVars"
    }

    fn check_source(&self, ctx: &LintContext) -> Vec<Diagnostic> {
        let mut diags = Vec::new();

        for (i, line) in ctx.lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }

            let line_start = ctx.line_start_offsets[i] as usize;
            let bytes = line.as_bytes();
            let n = bytes.len();
            let mut in_string: Option<u8> = None;
            let mut j = 0;

            while j < n {
                let b = bytes[j];

                if let Some(delim) = in_string {
                    match b {
                        b'\\' => {
                            j += 2;
                            continue;
                        }
                        c if c == delim => {
                            in_string = None;
                        }
                        _ => {}
                    }
                    j += 1;
                    continue;
                }

                match b {
                    b'"' | b'\'' | b'`' => {
                        in_string = Some(b);
                    }
                    b'#' => break, // inline comment — stop
                    b'$' => {
                        // Next char must be a letter or underscore (not digit, not punctuation)
                        if j + 1 < n
                            && (bytes[j + 1].is_ascii_alphabetic() || bytes[j + 1] == b'_')
                        {
                            // Also skip $- flags (two-char: $-x)
                            let var_start = j;
                            let mut k = j + 1;
                            while k < n
                                && (bytes[k].is_ascii_alphanumeric() || bytes[k] == b'_')
                            {
                                k += 1;
                            }
                            // Handle $-x style (e.g., $-v, $-w)
                            if k == j + 1 && k < n && bytes[k] == b'-' {
                                k += 1;
                                if k < n && bytes[k].is_ascii_alphabetic() {
                                    k += 1;
                                }
                            }

                            let var_name =
                                std::str::from_utf8(&bytes[var_start..k]).unwrap_or("$var");

                            if !is_builtin(var_name) {
                                let start = (line_start + var_start) as u32;
                                let end = (line_start + k) as u32;
                                diags.push(Diagnostic {
                                    rule: self.name(),
                                    message: format!(
                                        "Do not introduce global variables (`{}`).",
                                        var_name
                                    ),
                                    range: TextRange::new(start, end),
                                    severity: Severity::Warning,
                                });
                            }
                            j = k;
                            continue;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
        }

        diags
    }
}

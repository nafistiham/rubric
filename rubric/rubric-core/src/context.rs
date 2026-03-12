use std::path::Path;

/// Per-file context passed to every rule during a lint run.
pub struct LintContext<'src> {
    pub path: &'src Path,
    pub source: &'src str,
    /// Lines of source (without newlines).
    pub lines: Vec<&'src str>,
    /// Byte offset of the start of each line (index = line number, 0-based).
    pub line_start_offsets: Vec<u32>,
}

impl<'src> LintContext<'src> {
    pub fn new(path: &'src Path, source: &'src str) -> Self {
        let mut lines: Vec<&str> = source.lines().collect();
        let mut offsets = Vec::with_capacity(lines.len());
        let mut offset: u32 = 0;
        let mut remaining = source;
        for line in &lines {
            offsets.push(offset);
            offset += line.len() as u32;
            // Consume the line from remaining, then count actual newline bytes
            remaining = &remaining[line.len()..];
            if remaining.starts_with("\r\n") {
                offset += 2;
                remaining = &remaining[2..];
            } else if remaining.starts_with('\n') {
                offset += 1;
                remaining = &remaining[1..];
            }
            // No newline at end of file — offset stays as-is
        }
        // Truncate at `__END__` — Ruby ignores everything after this sentinel.
        let end_marker = lines.iter().position(|l| *l == "__END__");
        if let Some(end_idx) = end_marker {
            lines.truncate(end_idx);
            offsets.truncate(end_idx);
        }

        Self {
            path,
            source,
            lines,
            line_start_offsets: offsets,
        }
    }

    /// Convert a byte offset to (line, column), both 1-based.
    /// If `offset` exceeds the source length, returns the last line/column.
    /// Returns (1, 1) for empty source.
    pub fn offset_to_line_col(&self, offset: u32) -> (usize, usize) {
        if self.line_start_offsets.is_empty() {
            return (1, 1);
        }
        let line_idx = self
            .line_start_offsets
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);
        // Clamp to valid range in case offset exceeds source length
        let line_idx = line_idx.min(self.line_start_offsets.len() - 1);
        let col = (offset.saturating_sub(self.line_start_offsets[line_idx])) as usize + 1;
        (line_idx + 1, col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_context_splits_lines() {
        let source = "def foo\n  bar\nend\n";
        let ctx = LintContext::new(Path::new("test.rb"), source);
        assert_eq!(ctx.lines.len(), 3);
        assert_eq!(ctx.lines[0], "def foo");
        assert_eq!(ctx.lines[1], "  bar");
        assert_eq!(ctx.lines[2], "end");
    }

    #[test]
    fn test_context_line_start_offsets() {
        let source = "ab\ncd\n";
        let ctx = LintContext::new(Path::new("test.rb"), source);
        assert_eq!(ctx.line_start_offsets[0], 0);
        assert_eq!(ctx.line_start_offsets[1], 3);
    }

    #[test]
    fn test_offset_to_line_col() {
        let source = "ab\ncd\n";
        let ctx = LintContext::new(Path::new("test.rb"), source);
        assert_eq!(ctx.offset_to_line_col(0), (1, 1)); // 'a'
        assert_eq!(ctx.offset_to_line_col(1), (1, 2)); // 'b'
        assert_eq!(ctx.offset_to_line_col(3), (2, 1)); // 'c'
    }

    #[test]
    fn test_offset_to_line_col_empty_source() {
        let ctx = LintContext::new(Path::new("test.rb"), "");
        assert_eq!(ctx.offset_to_line_col(0), (1, 1));
    }

    #[test]
    fn test_crlf_line_endings() {
        // Simulate a file with Windows-style \r\n endings
        let source = "ab\r\ncd\r\n";
        let ctx = LintContext::new(Path::new("test.rb"), source);
        // "ab" at offset 0, "cd" at offset 4 (2 chars + \r\n = 4 bytes)
        assert_eq!(ctx.line_start_offsets[0], 0);
        assert_eq!(ctx.line_start_offsets[1], 4);
        assert_eq!(ctx.offset_to_line_col(4), (2, 1)); // 'c'
    }
}

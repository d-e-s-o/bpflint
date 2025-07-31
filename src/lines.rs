use std::iter::FusedIterator;


/// An iterator over the lines in a byte slice (typically representing
/// code).
#[derive(Debug)]
pub(crate) struct Lines<'src> {
    /// The source code in question.
    code: &'src [u8],
    /// The index at which we continue forward searching for lines.
    idx: Option<usize>,
}

impl<'src> Lines<'src> {
    /// Create a new [`Lines`] object, referencing the snippet `code` and
    /// starting line discovery/iteration at index `idx`.
    pub fn new(code: &'src [u8], idx: usize) -> Self {
        debug_assert!(
            idx <= code.len(),
            "invalid index `{idx}` for slice with length `{}`",
            code.len()
        );

        Self {
            code,
            idx: Some(idx),
        }
    }

    #[track_caller]
    fn find_line_start(code: &[u8], idx: usize) -> usize {
        // SANITY: The caller has to ensure that `idx` always maps to a
        //         valid position.
        code[..idx]
            .iter()
            .rposition(|&b| b == b'\n')
            .map(|idx| idx + 1)
            .unwrap_or(0)
    }

    #[track_caller]
    fn find_line_end(code: &[u8], idx: usize) -> usize {
        // SANITY: The caller has to ensure that `idx` always maps to a
        //         valid position.
        let code = &code[idx..];
        idx + code.iter().position(|&b| b == b'\n').unwrap_or(code.len())
    }
}

impl<'src> Iterator for Lines<'src> {
    type Item = &'src [u8];

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(idx) = self.idx {
            let start = Self::find_line_start(self.code, idx);
            let end = Self::find_line_end(self.code, idx);
            let line = &self.code[start..end];

            let next_idx = end + 1;
            if next_idx >= self.code.len() {
                self.idx = None;
            } else {
                self.idx = Some(next_idx);
            }
            Some(line)
        } else {
            None
        }
    }
}

impl FusedIterator for Lines<'_> {}


#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;


    /// Check that the [`Lines::find_line_start`] function works as it
    /// should.
    #[test]
    fn line_start_finding() {
        assert_eq!(Lines::find_line_start(b"", 0), 0);
        assert_eq!(Lines::find_line_start(b"a", 0), 0);
        assert_eq!(Lines::find_line_start(b"a", 1), 0);

        let code = b"\n";
        assert_eq!(Lines::find_line_start(code, 0), 0);
        assert_eq!(Lines::find_line_start(code, 1), 1);

        let code = b"a\n";
        assert_eq!(Lines::find_line_start(code, 0), 0);
        assert_eq!(Lines::find_line_start(code, 1), 0);
        assert_eq!(Lines::find_line_start(code, 2), 2);

        let code = b"ab\ncd";
        assert_eq!(Lines::find_line_start(code, 0), 0);
        assert_eq!(Lines::find_line_start(code, 1), 0);
        assert_eq!(Lines::find_line_start(code, 2), 0);
        assert_eq!(Lines::find_line_start(code, 3), 3);
        assert_eq!(Lines::find_line_start(code, 4), 3);
        assert_eq!(Lines::find_line_start(code, 5), 3);
    }

    /// Check that the [`Lines::find_line_end`] function works as it
    /// should.
    #[test]
    fn line_end_finding() {
        assert_eq!(Lines::find_line_end(b"", 0), 0);
        assert_eq!(Lines::find_line_end(b"a", 0), 1);
        assert_eq!(Lines::find_line_end(b"a", 1), 1);

        let code = b"\n";
        assert_eq!(Lines::find_line_end(code, 0), 0);
        assert_eq!(Lines::find_line_end(code, 1), 1);

        let code = b"a\n";
        assert_eq!(Lines::find_line_end(code, 0), 1);
        assert_eq!(Lines::find_line_end(code, 1), 1);
        assert_eq!(Lines::find_line_end(code, 2), 2);

        let code = b"\na";
        assert_eq!(Lines::find_line_end(code, 0), 0);
        assert_eq!(Lines::find_line_end(code, 1), 2);
        assert_eq!(Lines::find_line_end(code, 2), 2);

        let code = b"ab\ncd";
        assert_eq!(Lines::find_line_end(code, 0), 2);
        assert_eq!(Lines::find_line_end(code, 1), 2);
        assert_eq!(Lines::find_line_end(code, 2), 2);
        assert_eq!(Lines::find_line_end(code, 3), 5);
        assert_eq!(Lines::find_line_end(code, 4), 5);
        assert_eq!(Lines::find_line_end(code, 5), 5);
    }

    /// Test that we can iterate over lines in a forward fashion.
    #[test]
    fn forward_iteration() {
        let mut lines = Lines::new(b"", 0);
        assert_eq!(lines.next(), Some(b"".as_slice()));
        assert_eq!(lines.next(), None);

        let mut lines = Lines::new(b"a", 0);
        assert_eq!(lines.next(), Some(b"a".as_slice()));
        assert_eq!(lines.next(), None);

        let mut lines = Lines::new(b"a", 1);
        assert_eq!(lines.next(), Some(b"a".as_slice()));
        assert_eq!(lines.next(), None);

        let code = indoc! { br#"
          abc
          cde
          fgh
        "# };

        let mut lines = Lines::new(code, 0);
        assert_eq!(lines.next(), Some(b"abc".as_slice()));
        assert_eq!(lines.next(), Some(b"cde".as_slice()));
        assert_eq!(lines.next(), Some(b"fgh".as_slice()));
        assert_eq!(lines.next(), None);

        let mut lines = Lines::new(code, 3);
        assert_eq!(lines.next(), Some(b"abc".as_slice()));
        assert_eq!(lines.next(), Some(b"cde".as_slice()));
        assert_eq!(lines.next(), Some(b"fgh".as_slice()));
        assert_eq!(lines.next(), None);

        let mut lines = Lines::new(code, 4);
        assert_eq!(lines.next(), Some(b"cde".as_slice()));
        assert_eq!(lines.next(), Some(b"fgh".as_slice()));
        assert_eq!(lines.next(), None);
    }

    /// Make sure that we fail [`Lines`] construction with an invalid
    /// index.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic = "invalid index `1` for slice with length `0`"]
    fn invalid_initial_index_1() {
        let _lines = Lines::new(b"", 1);
    }

    /// Make sure that we fail [`Lines`] construction with an invalid
    /// index.
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic = "invalid index `2` for slice with length `1`"]
    fn invalid_initial_index_2() {
        let _lines = Lines::new(b"a", 2);
    }
}

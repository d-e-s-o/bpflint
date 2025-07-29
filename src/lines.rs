use std::iter::FusedIterator;

/// An iterator over the lines in a byte slice (typically representing
/// code).
#[derive(Debug)]
pub(crate) struct Lines<'src> {
    /// The source code in question.
    code: &'src [u8],
    /// The index at which we continue reverse searching for lines.
    start_idx: usize,
    /// The index at which we continue forward searching for lines.
    end_idx: usize,
}

impl<'src> Lines<'src> {
    /// Create a new [`Lines`] object, referencing the snippet `code` and
    /// starting line discovery/iteration at index `idx`.
    pub fn new(code: &'src [u8], idx: usize) -> Self {
        debug_assert!(idx <= code.len());

        Self {
            code,
            start_idx: idx,
            end_idx: idx,
        }
    }

    fn find_line_start(&self, idx: usize) -> usize {
        // SANITY: The caller has to ensure that `idx` always maps to a
        //         valid position.
        self.code[..idx]
            .iter()
            .rposition(|&b| b == b'\n')
            .map(|idx| idx + 1)
            .unwrap_or(0)
    }

    fn find_line_end(&self, idx: usize) -> usize {
        // SANITY: The caller has to ensure that `idx` always maps to a
        //         valid position.
        idx + self.code[idx..]
            .iter()
            .position(|&b| b == b'\n')
            .unwrap_or(0)
    }
}

impl<'src> Iterator for Lines<'src> {
    type Item = &'src [u8];

    fn next(&mut self) -> Option<Self::Item> {
      if self.end_idx == self.code.len() {
        return None
      }

      let idx = self.end_idx;
      let start = self.find_line_start(idx);
      self.end_idx = self.find_line_end(end);
      Some(&self.code[start..end])
    }
}

impl DoubleEndedIterator for Lines<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
      None
    }
}

impl FusedIterator for Lines<'_> {}


#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;


    /// XXX
    #[test]
    fn forward_iteration() {
        let code = indoc! { r#"
          abc
          cde
          fgh
        "# };

        let lines = Lines::new(code, 0);
    }
}

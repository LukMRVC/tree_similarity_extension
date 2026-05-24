//! Rust port of the C++ Touzet KR-set bounded tree edit distance (TopDiff).
//!
//! Built test-first against the retained C++ oracle `cppffi::tree_topdiff_bounded`.
//! See `docs/superpowers/plans/2026-05-24-unified-tree-index-pipeline.md` (Track A).

/// Rust equivalent of C++ `node::TreeIndexTouzetKRSet`. All vectors are indexed
/// by left-to-right POSTORDER id (`0..tree_size`).
///
/// INVARIANTS (must hold for both the reference builder in this module and the
/// production `expand` in `crate::types::unified_tree_index` — differential
/// tested at the A/B seam):
///   * labels are interned to `i32` against a dict SHARED with the SED form
///   * `postl_to_size`: subtree node count; leaf == 1
///   * `postl_to_depth`: root depth == 0
///   * `postl_to_lch`: leftmost-child postorder id; LEAF == -1
///   * `list_kr`: postorder ids of keyroots (non-first children + root); order irrelevant
///   * `postl_to_kr_ancestor`: nearest keyroot ancestor postorder id
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopDiffIndex {
    pub tree_size: i32,
    pub postl_to_label_id: Vec<i32>,
    pub postl_to_size: Vec<i32>,
    pub postl_to_depth: Vec<i32>,
    pub postl_to_lch: Vec<i32>,
    pub postl_to_kr_ancestor: Vec<i32>,
    pub list_kr: Vec<i32>,
}

/// A specialised matrix where only the elements on a diagonal band matter.
/// Port of the C++ `data_structures::BandMatrix<double>` (`include/matrix.h`).
///
/// The backing store is a flat `rows * (2 * band_width + 1)` vector. Accessing
/// `(row, col)` translates the column to `col + band_width - row` (the diagonal
/// shift that reduces memory from `O(n^2)` to `O(n*k)`). Callers must ensure all
/// accessed cells are within the band, exactly as in the C++ version.
#[derive(Debug, Clone)]
pub struct BandMatrix {
    columns: usize,
    band_width: usize,
    data: Vec<f64>,
}

impl BandMatrix {
    /// Mirrors `BandMatrix(rows, band_width)` followed by `fill_with(fill)`.
    /// Backing matrix has `2 * band_width + 1` columns.
    pub fn new(rows: usize, band_width: usize, fill: f64) -> Self {
        let columns = 2 * band_width + 1;
        Self {
            columns,
            band_width,
            data: vec![fill; rows * columns],
        }
    }

    #[inline]
    fn translate(&self, row: usize, col: usize) -> usize {
        // C++: Matrix::at(row, col + band_width_ - row). The column translation
        // can transiently underflow in usize, so compute in isize then index.
        let translated = col as isize + self.band_width as isize - row as isize;
        row * self.columns + translated as usize
    }

    /// `BandMatrix::at(row, col)` for writing.
    #[inline]
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        let idx = self.translate(row, col);
        self.data[idx] = value;
    }

    /// Mutable reference equivalent to C++ `at(row, col)`.
    #[inline]
    pub fn at(&mut self, row: usize, col: usize) -> &mut f64 {
        let idx = self.translate(row, col);
        &mut self.data[idx]
    }

    /// `BandMatrix::read_at(row, col)` — const read.
    #[inline]
    pub fn read_at(&self, row: usize, col: usize) -> f64 {
        let idx = self.translate(row, col);
        self.data[idx]
    }
}

/// Bounded tree edit distance via the Touzet KR-set algorithm. Returns the exact
/// TED when it is `<= k`, otherwise `k + 1` (the over-bound convention, matching
/// `bounded_sed_struct_int` and the C++ oracle `tree_topdiff_bounded`).
///
/// STUB — implemented in Track A (plan task A5).
#[allow(dead_code, unused_variables)]
pub fn ted_k(t1: &TopDiffIndex, t2: &TopDiffIndex, k: i32) -> i32 {
    todo!("Track A task A5: Touzet KR-set ted_k port")
}

#[cfg(test)]
mod tests {
    use super::*;

    const INF: f64 = f64::INFINITY;

    #[test]
    fn band_matrix_translate_and_fill() {
        let mut m = BandMatrix::new(5, 4, INF);
        // Untouched cell reads back the fill value.
        assert_eq!(m.read_at(2, 3), INF);
        // After a write, the same cell reads back the written value.
        m.set(2, 3, 7.0);
        assert_eq!(m.read_at(2, 3), 7.0);
        // The mutable accessor maps to the same backing cell.
        assert_eq!(*m.at(2, 3), 7.0);

        // band_width = 0 edge: single-column backing store, 1 row.
        let mut z = BandMatrix::new(1, 0, INF);
        assert_eq!(z.read_at(0, 0), INF);
        z.set(0, 0, 3.5);
        assert_eq!(z.read_at(0, 0), 3.5);
    }
}

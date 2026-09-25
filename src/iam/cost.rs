use pgrx::pg_sys;
use pgrx::prelude::*;

/// CPU cost of checking one index entry, in multiples of `cpu_operator_cost`.
/// The entries are pre-parsed, so this is far below the `<~` function cost.
const ENTRY_CHECK_OPS: f64 = 50.0;

#[pg_guard]
pub unsafe extern "C-unwind" fn amcostestimate(
    root: *mut pg_sys::PlannerInfo,
    path: *mut pg_sys::IndexPath,
    loop_count: f64,
    index_startup_cost: *mut pg_sys::Cost,
    index_total_cost: *mut pg_sys::Cost,
    index_selectivity: *mut pg_sys::Selectivity,
    index_correlation: *mut f64,
    index_pages: *mut f64,
) {
    unsafe {
        let info = (*path).indexinfo;
        let entries = (*info).tuples.max(0.0);
        let pages = (*info).pages as f64;

        // Use the generic estimate for selectivity; the costs are replaced below
        // because every scan reads all entries.
        let mut costs = pg_sys::GenericCosts {
            numIndexTuples: entries,
            ..Default::default()
        };
        pg_sys::genericcostestimate(root, path, loop_count, &mut costs);

        *index_startup_cost = 0.0;
        *index_total_cost = pages * pg_sys::seq_page_cost
            + entries * (pg_sys::cpu_index_tuple_cost + ENTRY_CHECK_OPS * pg_sys::cpu_operator_cost);
        *index_selectivity = costs.indexSelectivity;
        *index_correlation = 0.0;
        *index_pages = pages;
    }
}

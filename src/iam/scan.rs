//! Bitmap-only scans: every entry is checked against every scan key.
//!
//! With `tree_search_iam.scan_threads > 1` the backend reads raw entries in
//! batches and that many pool threads check them (`parallel::run_scan`).

use pgrx::guc::{GucContext, GucFlags, GucRegistry, GucSetting};
use pgrx::itemptr::u64_to_item_pointer;
use pgrx::pg_sys;
use pgrx::prelude::*;

use super::build::rel_name;
use super::options::index_lb;
use super::parallel;
use super::storage::{self, PageSource, StreamReader};
use crate::types::{TreeQuery, UnifiedTreeIndex};

/// Default `tree_search_iam.scan_threads`.
const DEFAULT_SCAN_THREADS: i32 = 4;

static SCAN_THREADS: GucSetting<i32> = GucSetting::<i32>::new(DEFAULT_SCAN_THREADS);

/// Encoded bytes per batch handed to a scan thread.
const BATCH_BYTES: usize = 64 << 10;

/// Register `tree_search_iam.scan_threads`. Runs once per backend, from `_PG_init`.
pub fn register_guc() {
    // The default must lie within the range, so the cap never drops below it,
    // even on machines with fewer cores.
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get()) as i32;
    let max = cores.clamp(DEFAULT_SCAN_THREADS, 64);
    GucRegistry::define_int_guc(
        c"tree_search_iam.scan_threads",
        c"Threads each tree_search_iam index scan uses to check entries.",
        c"1 checks entries on the backend alone.",
        &SCAN_THREADS,
        1,
        max,
        GucContext::Userset,
        GucFlags::default(),
    );
}

#[pg_guard]
pub unsafe extern "C-unwind" fn ambeginscan(
    index: pg_sys::Relation,
    nkeys: std::ffi::c_int,
    norderbys: std::ffi::c_int,
) -> pg_sys::IndexScanDesc {
    unsafe { pg_sys::RelationGetIndexScan(index, nkeys, norderbys) }
}

#[pg_guard]
pub unsafe extern "C-unwind" fn amrescan(
    scan: pg_sys::IndexScanDesc,
    keys: pg_sys::ScanKey,
    nkeys: std::ffi::c_int,
    _orderbys: pg_sys::ScanKey,
    _norderbys: std::ffi::c_int,
) {
    unsafe {
        if !keys.is_null() && nkeys > 0 {
            std::ptr::copy(keys, (*scan).keyData, nkeys as usize);
        }
    }
}

#[pg_guard]
pub unsafe extern "C-unwind" fn amgetbitmap(
    scan: pg_sys::IndexScanDesc,
    tbm: *mut pg_sys::TIDBitmap,
) -> i64 {
    unsafe {
        let index = (*scan).indexRelation;
        let meta = storage::read_meta(index);
        if meta.stale != 0 {
            let name = rel_name(index);
            error!(
                "tree_search_iam index \"{name}\" is stale: rows were added or removed after it was built; run REINDEX INDEX {name}"
            );
        }

        let mut queries: Vec<(UnifiedTreeIndex, i32)> = Vec::new();
        for i in 0..(*scan).numberOfKeys as usize {
            let key = &*(*scan).keyData.add(i);
            // A NULL query matches nothing.
            if key.sk_flags & pg_sys::SK_ISNULL as i32 != 0 {
                return 0;
            }
            let Some(query) = TreeQuery::from_datum(key.sk_argument, false) else {
                return 0;
            };
            queries.push((query.tree.to_unified(), query.k));
        }

        let lb = index_lb(index);
        let threads = SCAN_THREADS.get().max(1) as usize;
        let mut reader = StreamReader::new(PageSource::for_scan(index, &meta));
        let mut tid = pg_sys::ItemPointerData::default();
        let mut n_matches = 0i64;
        let mut add = |raw_tids: &[u64]| {
            for raw_tid in raw_tids {
                u64_to_item_pointer(*raw_tid, &mut tid);
                pg_sys::tbm_add_tuples(tbm, &mut tid, 1, false);
            }
            n_matches += raw_tids.len() as i64;
        };

        if threads == 1 {
            let mut tree = storage::empty_tree();
            while let Some(raw_tid) = reader.next_entry_into(&mut tree) {
                pg_sys::check_for_interrupts!();
                if parallel::matches(&queries, lb, &tree) {
                    add(&[raw_tid]);
                }
            }
        } else {
            parallel::run_scan(&mut reader, queries, lb, threads, BATCH_BYTES, add, || {
                pg_sys::check_for_interrupts!()
            })
            .unwrap_or_else(|e| error!("tree_search_iam: scan thread failed: {e}"));
        }
        n_matches
    }
}

#[pg_guard]
pub unsafe extern "C-unwind" fn amendscan(_scan: pg_sys::IndexScanDesc) {}

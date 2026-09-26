//! Bitmap-only scans: every entry is checked against every scan key.
//!
//! With `tree_search_iam.scan_threads > 1` the backend reads raw entries in
//! batches and `parallel::filter_batch` checks each batch on that many threads.

use pgrx::guc::{GucContext, GucFlags, GucRegistry, GucSetting};
use pgrx::itemptr::u64_to_item_pointer;
use pgrx::pg_sys;
use pgrx::prelude::*;

use super::build::rel_name;
use super::options::index_lb;
use super::parallel::{self, Batch};
use super::storage::{self, PageSource, StreamReader};
use crate::types::{TreeQuery, UnifiedTreeIndex};

static SCAN_THREADS: GucSetting<i32> = GucSetting::<i32>::new(1);

/// Encoded bytes per batch; also bounds interrupt latency.
const BATCH_BYTES: usize = 1 << 20;

/// Register `tree_search_iam.scan_threads`. Runs once per backend, from `_PG_init`.
pub fn register_guc() {
    let max = std::thread::available_parallelism().map_or(1, |n| n.get()).min(64) as i32;
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
        let mut add = |raw_tid: u64| {
            u64_to_item_pointer(raw_tid, &mut tid);
            pg_sys::tbm_add_tuples(tbm, &mut tid, 1, false);
        };
        let mut n_matches = 0i64;

        // One thread: decode straight from the reader, skipping the batch copy.
        if threads == 1 {
            while let Some((raw_tid, tree)) = reader.next_entry() {
                pg_sys::check_for_interrupts!();
                if parallel::matches(&queries, lb, &tree) {
                    add(raw_tid);
                    n_matches += 1;
                }
            }
            return n_matches;
        }

        // Only this (backend) thread touches Postgres; worker threads exist
        // only inside `filter_batch`, so an ERROR here never leaves one running.
        let mut batch = Batch::default();
        while batch.refill(&mut reader, BATCH_BYTES) {
            pg_sys::check_for_interrupts!();
            let tids = parallel::filter_batch(&batch, &queries, lb, threads)
                .unwrap_or_else(|e| error!("tree_search_iam: scan thread failed: {e}"));
            tids.iter().for_each(|t| add(*t));
            n_matches += tids.len() as i64;
        }
        n_matches
    }
}

#[pg_guard]
pub unsafe extern "C-unwind" fn amendscan(_scan: pg_sys::IndexScanDesc) {}

//! Bitmap-only scans: every entry is checked against every scan key.

use pgrx::itemptr::u64_to_item_pointer;
use pgrx::pg_sys;
use pgrx::prelude::*;

use super::build::rel_name;
use super::options::index_lb;
use super::storage::{self, PageSource, StreamReader};
use crate::types::{TreeQuery, UnifiedTreeIndex};

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
        let mut reader = StreamReader::new(PageSource::new(index, &meta));
        let mut tid = pg_sys::ItemPointerData::default();
        let mut n_matches = 0i64;
        while let Some((raw_tid, tree)) = reader.next_entry() {
            pg_sys::check_for_interrupts!();
            if queries.iter().all(|(q, k)| lb.within(q, &tree, *k) <= *k) {
                u64_to_item_pointer(raw_tid, &mut tid);
                pg_sys::tbm_add_tuples(tbm, &mut tid, 1, false);
                n_matches += 1;
            }
        }
        n_matches
    }
}

#[pg_guard]
pub unsafe extern "C-unwind" fn amendscan(_scan: pg_sys::IndexScanDesc) {}

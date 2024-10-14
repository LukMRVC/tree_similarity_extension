// use pgrx::{prelude::*, PgRelation};
// use rustc_hash::FxHashMap;

// type Key = String;
// // Posting lists are tuples of (tuple_id, node_postorder, structural_vector)
// type Value = Vec<(u64, i32, [i32; 4])>;

// type Index = FxHashMap<Key, Value>;

// // Used to build Index access method, or e.g., the index...
// pub extern "C" fn ambuild(
//     heaprel: pg_sys::Relation,
//     indexrel: pg_sys::Relation,
//     index_info: *mut pg_sys::IndexInfo,
// ) -> *mut pg_sys::IndexBuildResult {
//     let heap_relation = unsafe { PgRelation::from_pg(heaprel) };
//     let index_relation = unsafe { PgRelation::from_pg(indexrel) };

//     unsafe {
//         if index_info
//             .as_ref()
//             .expect("Missing index info!")
//             .ii_Concurrent
//         {
//             panic!("Structural filter index cannot be build concurrently!");
//         }
//     }

//     if index_relation
//         .reltuples()
//         .expect("Unable to get number of reltuples in index!")
//         > 0f32
//     {
//         log!("Index is not empty!");
//     }

//     let mut index = Index::default();

//     let ntuples = scan_heap_for_index(&heap_relation, &index_relation, index_info, &mut index);

//     let mut result = unsafe { PgBox::<pg_sys::IndexBuildResult>::alloc0() };
//     result.heap_tuples = ntuples;
//     result.index_tuples = ntuples;

//     result.into_pg()
// }

// struct IndexBuildState<'a> {
//     index: &'a mut Index,
//     indexed_tuples: usize,
//     memcxt: pgrx::memcxt::PgMemoryContexts,
// }

// impl<'a> IndexBuildState<'a> {
//     fn new(index: &'a mut Index) -> Self {
//         IndexBuildState {
//             index,
//             indexed_tuples: 0,
//             memcxt: pgrx::memcxt::PgMemoryContexts::new("structural filter index build context"),
//         }
//     }
// }

// fn scan_heap_for_index<'a>(
//     heap_relation: &'a PgRelation,
//     index_relation: &'a PgRelation,
//     index_info: *mut pg_sys::IndexInfo,
//     index: &'a mut Index,
// ) -> f64 {
//     let mut build_state = IndexBuildState::new(index);

//     unsafe {
//         pg_sys::IndexBuildHeapScan(
//             heap_relation.as_ptr(),
//             index_relation.as_ptr(),
//             index_info,
//             Some(index_build_callback),
//             &mut build_state,
//         );
//     }

//     build_state.indexed_tuples as f64
// }

// #[cfg(any(
//     feature = "pg13",
//     feature = "pg14",
//     feature = "pg15",
//     feature = "pg16",
//     feature = "pg17"
// ))]
// #[pg_guard]
// unsafe extern "C" fn index_build_callback(
//     index: pg_sys::Relation,
//     tid: pg_sys::ItemPointer,
//     values: *mut pg_sys::Datum,
//     is_null: *mut bool,
//     _tuple_is_alive: bool,
//     state: *mut ::core::ffi::c_void,
// ) {
//     use crate::types::{StructuralFilter, TreeArena};

//     check_for_interrupts!();

//     let state = (state as *mut IndexBuildState).as_mut().unwrap();
//     let mut old_context = state.memcxt.set_as_current();

//     let values = std::slice::from_raw_parts(values, 1);
//     log!("Index build callback values: {values:?}");

//     let pt_data = *tid;
//     // get tuple id
//     let tid = pgrx::itemptr::item_pointer_to_u64(pt_data);

//     let tree = TreeArena::from_datum_in_memory_context(
//         old_context,
//         values[0],
//         *is_null,
//         TreeArena::type_oid(),
//     )
//     .expect("Unable to convert Datum into TreeArena");
//     log!("Converted Datum value into tree: {tree:?}");

//     let sf = StructuralFilter::from(tree);
//     for (k, v) in sf.1.iter() {
//         state
//             .index
//             .entry(k.clone())
//             .and_modify(|postings| {
//                 postings.extend(
//                     v.struct_vec
//                         .iter()
//                         .map(|vecs| (tid, vecs.postorder_id, vecs.mapping_regions)),
//                 );
//             })
//             .or_insert_with(|| {
//                 v.struct_vec
//                     .iter()
//                     .map(|vecs| (tid, vecs.postorder_id, vecs.mapping_regions))
//                     .collect::<Vec<_>>()
//             });
//     }

//     state.memcxt.set_as_current();
//     state.memcxt.reset();
// }

// mod build;
// use pgrx::prelude::*;

// #[pg_extern(sql = "
//     CREATE OR REPLACE FUNCTION iamhandler(internal) RETURNS index_am_handler PARALLEL SAFE IMMUTABLE STRICT COST 0.0001 LANGUAGE c AS 'MODULE_PATHNAME', '@FUNCTION_NAME@';
//     CREATE ACCESS METHOD iamstructuralfilter TYPE INDEX HANDLER iamhandler;
// ")]
// fn amhandler(_fcinfo: pg_sys::FunctionCallInfo) -> PgBox<pg_sys::IndexAmRoutine> {
//     let mut amroutine =
//         unsafe { PgBox::<pg_sys::IndexAmRoutine>::alloc_node(pg_sys::NodeTag::T_IndexAmRoutine) };

//     amroutine.amstrategies = 1;
//     amroutine.amsupport = 0;
//     amroutine.amcanparallel = true;
//     amroutine.amcanunique = false;
//     amroutine.amcanmulticol = false;
//     /* support for postgres 17 and up
//     amroutine.amcanbuildparallel = true;
//     */
//     amroutine.amsearcharray = false;

//     amroutine.amkeytype = pg_sys::InvalidOid;

//     amroutine.amvalidate = Some(amvalidate);
//     amroutine.ambuild = Some(build::ambuild);
//     // amroutine.ambuildempty = Some(build::ambuildempty);
//     // amroutine.aminsert = Some(build::aminsert);
//     // amroutine.ambulkdelete = Some(vacuum::ambulkdelete);
//     // amroutine.amvacuumcleanup = Some(vacuum::amvacuumcleanup);
//     // amroutine.amcostestimate = Some(cost_estimate::amcostestimate);
//     // amroutine.amoptions = Some(options::amoptions);
//     // amroutine.ambeginscan = Some(scan::ambeginscan);
//     // amroutine.amrescan = Some(scan::amrescan);
//     // amroutine.amgettuple = Some(scan::amgettuple);
//     // amroutine.amgetbitmap = Some(scan::ambitmapscan);
//     // amroutine.amendscan = Some(scan::amendscan);

//     amroutine.into_pg_boxed()
// }

// #[pg_guard]
// pub extern "C" fn amvalidate(_opclassoid: pg_sys::Oid) -> bool {
//     true
// }

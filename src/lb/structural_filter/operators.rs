use pg_sys::StrategyNumber;
use pgrx::prelude::*;
use pgrx::Internal;

use crate::types::StructuralFilter;

// TODO: Add type for structural vector to be a postgres type
// TODO: Then define operators for GIN index for that type.
// TODO: define extract operators, pg_sys module will most probably be needed

const STRATEGY_OVERLAP: StrategyNumber = 2;

extension_sql!(
    r#"
    create operator class structuralfilter_ops
        default for type structuralfilter using gin as 
            function    2   structuralfilter_overlaps(structuralfilter, structuralfilter)
"#,
    name = "create_structural_operator_class"
);

// are all labels from right in left aswell?
#[pg_operator(immutable, parallel_safe)]
#[opname(@>)]
fn structuralfilter_contains(left: StructuralFilter, right: StructuralFilter) -> bool {
    let mut t1keys = left.1.keys();
    let mut t2keys = right.1.keys();

    t2keys.all(|k2| t1keys.any(|k1| k1 == k2))
}

// tests if left and right have at least one same label
#[pg_operator(immutable, parallel_safe)]
#[opname(&&)]
fn structuralfilter_overlaps(left: StructuralFilter, right: StructuralFilter) -> bool {
    left.1.keys().any(|k| right.1.contains_key(k))
}

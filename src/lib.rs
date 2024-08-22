use pgrx::prelude::*;

pgrx::pg_module_magic!();

mod lb;
mod parsing;
mod types;

use crate::lb::label_intersection::label_intersection_distance;
use crate::types::tree_arena::TreeArena;
use types::tree_arena;

#[pg_extern]
fn add_node_to_tree_root(mut input_tree: TreeArena, node_value: String) -> TreeArena {
    if let Some(root_id) = input_tree.get_root_id() {
        let new_node = input_tree.new_node(node_value);
        root_id.append(new_node, &mut input_tree);
    } else {
        panic!("Tree has no root!")
    }

    input_tree
}

// TODO: LB filter functions
#[pg_extern(immutable, parallel_safe)]
fn tree_lb_label_intersect(t1: TreeArena, t2: TreeArena) -> i16 {
    let lb = label_intersection_distance(&t1, &t2, t1.count() + t2.count());
    lb as i16
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use super::*;
    use pgrx::prelude::*;
    use std::ffi::CString;

    #[pg_test]
    fn test_label_intersection() {
        let t1 = parsing::parse_tree(&CString::new("{a{b}{c}}").unwrap()).unwrap();
        let t2 = parsing::parse_tree(&CString::new("{a{d}{c}}").unwrap()).unwrap();
        assert_eq!(
            label_intersection_distance(&t1, &t2, t1.count() + t2.count()),
            1
        )
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}

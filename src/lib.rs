use cppffi::tree_ted;

use lb::structural_filter::LabelSetConverter;
use pgrx::prelude::*;

pgrx::pg_module_magic!();

mod lb;
mod parsing;
mod types;

use crate::lb::{
    label_intersection::{bounded_label_intersection_distance, label_intersection_distance},
    sed::{bounded_sed, sed},
    structural_filter::ted as structural_lb,
};
use crate::types::tree_arena::TreeArena;
use types::tree_arena;

#[cxx::bridge]
mod cppffi {

    unsafe extern "C++" {
        include!("tree_similarity_extension/include/apted.h");

        fn tree_ted(a: String, b: String) -> u32;
    }
}

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
fn tree_lb_label_intersect(t1: TreeArena, t2: TreeArena) -> i32 {
    let lb = label_intersection_distance(&t1, &t2);
    lb as i32
}

#[pg_extern(immutable, parallel_safe)]
fn tree_lb_bounded_label_intersect(t1: TreeArena, t2: TreeArena, lb: i32) -> i32 {
    let lb = bounded_label_intersection_distance(&t1, &t2, lb as usize);
    lb as i32
}

// TODO: LB filter functions
#[pg_extern(immutable, parallel_safe)]
fn tree_lb_sed(t1: TreeArena, t2: TreeArena) -> i32 {
    let lb = sed(&t1, &t2);
    lb as i32
}

#[pg_extern(immutable, parallel_safe)]
fn tree_lb_bounded_sed(t1: TreeArena, t2: TreeArena, lb: i32) -> i32 {
    let lb = bounded_sed(&t1, &t2, lb as usize);
    lb as i32
}

#[pg_extern(immutable, parallel_safe)]
fn tree_lb_structural_filter(t1: TreeArena, t2: TreeArena, lb: i32) -> i32 {
    let mut lsc = LabelSetConverter::default();
    let tree_tuples = lsc.create(&vec![t1, t2]);
    match &tree_tuples[..2] {
        [s1, s2] => structural_lb(s1, s2, lb),
        _ => panic!("Trees failed to convert!"),
    }
}

#[pg_extern(immutable, parallel_safe)]
fn tree_ed(t1: TreeArena, t2: TreeArena) -> i32 {
    tree_ted(t1.to_string(), t2.to_string()) as i32
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    // TODO: Add postgres tests
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

use pgrx::prelude::*;

pgrx::pg_module_magic!();

mod types;
mod parsing;

use types::tree_arena;
use crate::types::tree_arena::TreeArena;

#[pg_extern]
fn hello_tree_similarity_extension() -> &'static str {
    "Hello, from my own extension 2!"
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

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_hello_tree_similarity_extension() {
        assert_eq!(
            "Hello, from my own extension 2!",
            crate::hello_tree_similarity_extension()
        );
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

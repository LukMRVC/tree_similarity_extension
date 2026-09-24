use crate::types::TreeArena;
use crate::types::tree_internals::id::NodeId;
use memchr::memchr2_iter;
use std::ffi::CStr;
use std::io;
use thiserror::Error;

/// A node label, stored as its XXH3-64 hash (see [`label_hash`]).
pub type LabelHash = u64;
pub type LabelId = LabelHash;
pub type ParsedTree = TreeArena;

/// Hash a label's raw bytes (exactly the slice the parser extracts, escapes not
/// decoded) into its global id.
///
/// The id depends only on the bytes, so it is consistent across every tree in
/// every table and no per-call label dictionary is needed. XXH3-64 has a frozen
/// specification, so hashes stored on disk stay valid across crate upgrades —
/// do not swap this for FxHash or `DefaultHasher`, whose output is not stable.
#[inline]
pub fn label_hash(label: &[u8]) -> LabelHash {
    twox_hash::XxHash3_64::oneshot(label)
}

#[derive(Error, Debug)]
pub enum TreeParseError {
    #[error("tree string contains non ascii characters")]
    IsNotAscii,
    #[error(transparent)]
    LineReadError(#[from] io::Error),
    #[error("tree string has incorrect bracket notation format: {}", .0)]
    IncorrectFormat(String),
    #[error("Bad tokenizing")]
    TokenizerError,
}

const TOKEN_OPEN_NODE: u8 = b'{';
const TOKEN_CLOSE_NODE: u8 = b'}';
const TOKEN_ESCAPE: u8 = b'\\';

// A brace is escaped iff the byte before it is `\`. Kept byte-for-byte
// equivalent to `is_escaped` in ted-search's `lib/tree-parsing`, which is what
// produced the dataset bracket files: an extra `byte_string[offset - 2]`
// lookback here used to disagree with it on odd backslash runs of three or more
// (`\\\}`), so lines that library accepts failed to parse.
#[inline(always)]
fn is_escaped(byte_string: &[u8], offset: usize) -> bool {
    offset > 0 && byte_string[offset - 1] == TOKEN_ESCAPE
}

/// Receives the node events of a bracket-notation string, in document order.
pub(crate) trait BracketVisitor {
    /// A node opens; `label` is its raw label bytes.
    fn open(&mut self, label: &[u8]);
    /// The most recently opened, still-open node closes.
    fn close(&mut self);
}

/// Tokenize a bracket-notation string and drive `visitor` with its node events.
///
/// This is the single tokenizer behind every parser in the crate, so they all
/// agree on labels and on which inputs are rejected. A node's label is the
/// slice from its `{` to the next unescaped brace. Nodes still open at the end
/// of the input are accepted without a `close` event; visitors that care must
/// close them themselves.
pub(crate) fn walk_bracket(
    tree_bracket_bytes: &[u8],
    visitor: &mut impl BracketVisitor,
) -> Result<(), TreeParseError> {
    use TreeParseError as TPE;

    if tree_bracket_bytes[0] == TOKEN_ESCAPE {
        return Err(TPE::IncorrectFormat(
            "Tree bracket string starts with escape char \\".to_owned(),
        ));
    }

    let token_positions: Vec<usize> =
        memchr2_iter(TOKEN_OPEN_NODE, TOKEN_CLOSE_NODE, tree_bracket_bytes)
            .filter(|char_pos| !is_escaped(tree_bracket_bytes, *char_pos))
            .collect();

    let mut tokens = token_positions.iter().peekable();
    let root_start = *tokens.next().unwrap();
    let root_end = **tokens.peek().expect("Root node had not been closed");

    visitor.open(&tree_bracket_bytes[(root_start + 1)..root_end]);
    // Number of currently open nodes.
    let mut depth = 1usize;

    while let Some(token) = tokens.next() {
        match tree_bracket_bytes[*token] {
            TOKEN_OPEN_NODE => {
                let Some(token_end) = tokens.peek() else {
                    let err_msg = format!("Label has no ending token near col {token}");
                    return Err(TPE::IncorrectFormat(err_msg));
                };
                if depth == 0 {
                    let err_msg = "Reached unexpected end of tree string".to_owned();
                    return Err(TPE::IncorrectFormat(err_msg));
                }
                visitor.open(&tree_bracket_bytes[(*token + 1)..**token_end]);
                depth += 1;
            }
            TOKEN_CLOSE_NODE => {
                if depth == 0 {
                    return Err(TPE::IncorrectFormat("Wrong bracket pairing".to_owned()));
                }
                visitor.close();
                depth -= 1;
            }
            _ => return Err(TPE::TokenizerError),
        }
    }

    Ok(())
}

/// Builds a `TreeArena`, attaching each opened node to the innermost open one.
struct ArenaBuilder {
    tree: TreeArena,
    node_stack: Vec<NodeId>,
}

impl BracketVisitor for ArenaBuilder {
    fn open(&mut self, label: &[u8]) {
        let label = unsafe { String::from_utf8_unchecked(label.to_vec()) };
        let n = self.tree.new_node(label);
        if let Some(parent) = self.node_stack.last() {
            parent.append(n, &mut self.tree);
        }
        self.node_stack.push(n);
    }

    fn close(&mut self) {
        self.node_stack.pop();
    }
}

pub fn parse_tree(tree_bracket_str: &CStr) -> Result<TreeArena, TreeParseError> {
    let tree_bracket_bytes = tree_bracket_str.to_bytes();
    let mut builder = ArenaBuilder {
        tree: TreeArena::with_capacity(tree_bracket_bytes.len() / 3),
        node_stack: Vec::new(),
    };
    walk_bracket(tree_bracket_bytes, &mut builder)?;
    Ok(builder.tree)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reference sanity buffer from xxHash's `XSUM_fillTestBuffer`
    /// (`cli/xsum_sanity_check.c`): byte i = top byte of PRIME32 * PRIME64^i.
    fn xxhash_sanity_buffer(len: usize) -> Vec<u8> {
        const PRIME32: u64 = 2654435761;
        const PRIME64: u64 = 11400714785074694797;
        let mut byte_gen = PRIME32;
        (0..len)
            .map(|_| {
                let b = (byte_gen >> 56) as u8;
                byte_gen = byte_gen.wrapping_mul(PRIME64);
                b
            })
            .collect()
    }

    /// Hashes are stored on disk, so their values must never change. Expected
    /// values are XXH3-64 (seed 0) from the xxHash reference test vectors
    /// (`tests/sanity_test_vectors.h`), independent of the Rust crate.
    #[test]
    fn label_hash_matches_xxh3_reference_vectors() {
        let vectors: &[(usize, u64)] = &[
            (0, 0x2D06800538D394C2),
            (1, 0xC44BDFF4074EECDB),
            (3, 0x54247382A8D6B94D),
            (4, 0xE5DC74BC51848A51),
            (8, 0x24CCC9ACAA9F65E4),
            (9, 0x14D5001C15DD3F2B),
            (16, 0x981B17D36C7498C9),
            (17, 0x796F5ACD3A60F862),
            (128, 0xFCFF24126754D861),
            (129, 0x98F1B0A679A2CA29),
            (240, 0x81C3C2B67F568CCF),
            (241, 0xC5A639ECD2030E5E),
        ];
        let buffer = xxhash_sanity_buffer(256);
        for &(len, expected) in vectors {
            assert_eq!(label_hash(&buffer[..len]), expected, "XXH3-64 mismatch at len {len}");
        }
    }

    /// Records every distinct label hash with an independent fingerprint of the
    /// label bytes, flagging any hash seen with two different fingerprints.
    struct CollisionCheck {
        seen: std::collections::HashMap<LabelHash, u64>,
        collisions: Vec<(LabelHash, String)>,
    }

    impl BracketVisitor for CollisionCheck {
        fn open(&mut self, label: &[u8]) {
            use std::hash::{BuildHasher, BuildHasherDefault, DefaultHasher};
            // SipHash (std), unrelated to XXH3, so a real XXH3 collision is
            // missed only if SipHash collides on the same pair too.
            let fingerprint = BuildHasherDefault::<DefaultHasher>::default().hash_one(label);
            let prev = *self.seen.entry(label_hash(label)).or_insert(fingerprint);
            if prev != fingerprint {
                self.collisions
                    .push((label_hash(label), String::from_utf8_lossy(label).into_owned()));
            }
        }
        fn close(&mut self) {}
    }

    /// `walk_bracket` panics (like `parse_tree`) on an empty line or one with
    /// fewer than two unescaped braces; the raw dataset files contain a few
    /// lines the loader drops, so skip those up front.
    fn walkable(bytes: &[u8]) -> bool {
        !bytes.is_empty()
            && memchr2_iter(TOKEN_OPEN_NODE, TOKEN_CLOSE_NODE, bytes)
                .filter(|&pos| !is_escaped(bytes, pos))
                .nth(1)
                .is_some()
    }

    /// Every label of every tree and query of each dataset must hash to a
    /// distinct value, i.e. XXH3-64 is collision-free on each table's labels.
    /// Checked per dataset (one tree table + its queries). Slow and reads
    /// `datasets/`, hence ignored by default:
    /// `cargo test --release --lib -- --ignored no_label_hash_collisions`
    #[test]
    #[ignore]
    fn no_label_hash_collisions_in_datasets() {
        use std::io::BufRead;

        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("datasets");
        let mut dirs: Vec<_> = std::fs::read_dir(&root)
            .expect("datasets/ directory")
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.join("trees_sorted.bracket").is_file())
            .collect();
        dirs.sort();
        assert!(!dirs.is_empty(), "no datasets found under {}", root.display());

        let mut all_collisions = Vec::new();
        for dir in dirs {
            let mut check = CollisionCheck {
                seen: Default::default(),
                collisions: Vec::new(),
            };
            let mut trees = 0usize;
            for (file, is_query) in [("trees_sorted.bracket", false), ("query.csv", true)] {
                let Ok(f) = std::fs::File::open(dir.join(file)) else {
                    continue;
                };
                for line in std::io::BufReader::new(f).split(b'\n') {
                    let line = line.expect("read line");
                    // query.csv rows are `k;<tree>`; split on the first `;` only.
                    let tree = match (is_query, line.iter().position(|&b| b == b';')) {
                        (true, Some(pos)) => &line[pos + 1..],
                        (true, None) => continue,
                        (false, _) => &line[..],
                    };
                    if walkable(tree) && walk_bracket(tree, &mut check).is_ok() {
                        trees += 1;
                    }
                }
            }
            let name = dir.file_name().unwrap().to_string_lossy().into_owned();
            eprintln!(
                "{name}: {trees} trees, {} distinct label hashes, {} collisions",
                check.seen.len(),
                check.collisions.len()
            );
            all_collisions.extend(check.collisions.into_iter().map(|c| (name.clone(), c)));
        }
        assert!(all_collisions.is_empty(), "label hash collisions: {all_collisions:?}");
    }
}

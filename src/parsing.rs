use std::ffi::CStr;
use std::io;
use super::tree_arena::TreeArena;
use memchr::memchr2_iter;
use thiserror::Error;


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


#[inline(always)]
fn is_escaped(byte_string: &[u8], offset: usize) -> bool {
    offset > 0
        && byte_string[offset - 1] == TOKEN_ESCAPE
        && !(offset > 1 && byte_string[offset - 2] == TOKEN_ESCAPE)
}

pub fn parse_tree(
    tree_bracket_str: &CStr,
) -> Result<TreeArena, TreeParseError> {
    use TreeParseError as TPE;

    let tree_bracket_bytes = tree_bracket_str.to_bytes();
    if tree_bracket_bytes[0] == TOKEN_ESCAPE {
        return Err(TPE::IncorrectFormat("Tree bracket string starts with escape char \\".to_owned()));
    }

    let mut tree = TreeArena::with_capacity(tree_bracket_bytes.len() / 3);

    let token_positions: Vec<usize> = memchr2_iter(TOKEN_OPEN_NODE, TOKEN_CLOSE_NODE, tree_bracket_bytes)
        .filter(|char_pos| !is_escaped(tree_bracket_bytes, *char_pos))
        .collect();

    let mut tokens = token_positions.iter().peekable();
    let root_start = *tokens.next().unwrap();
    let root_end = **tokens.peek().expect("Root node had not been closed");

    let root_label = unsafe {
        String::from_utf8_unchecked(tree_bracket_bytes[(root_start + 1)..root_end].to_vec())
    };

    let root = tree.new_node(root_label);

    let mut node_stack = vec![root];


    while let Some(token) = tokens.next() {
        match tree_bracket_bytes[*token] {
            TOKEN_OPEN_NODE => {
                let Some(token_end) = tokens.peek() else {
                    let err_msg =
                        format!("Label has no ending token near col {token}");
                    return Err(TPE::IncorrectFormat(err_msg));
                };
                let label =
                    unsafe { String::from_utf8_unchecked(tree_bracket_bytes[(*token + 1)..**token_end].to_vec()) };

                let n = tree.new_node(label);
                let Some(last_node) = node_stack.last() else {
                    let err_msg = "Reached unexpected end of tree string".to_owned();
                    return Err(TPE::IncorrectFormat(err_msg));
                };
                last_node.append(n, &mut tree);
                node_stack.push(n);
            }
            TOKEN_CLOSE_NODE => {
                let Some(_) = node_stack.pop() else {
                    return Err(TPE::IncorrectFormat("Wrong bracket pairing".to_owned()));
                };
            }
            _ => return Err(TPE::TokenizerError),
        }
    }

    Ok(tree)
}
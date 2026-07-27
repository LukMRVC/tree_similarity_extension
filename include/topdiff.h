#pragma once

#include <memory>
#include <string>
#include <cstdint>
#include "unit_cost_model.h"
#include "string_label.h"
#include "label_dictionary.h"
#include "tree_indexer.h"
#include "node.h"
#include "touzet_kr_set_tree_index.h"
#include "./bracket_notation_parser.h"
#include "rust/cxx.h"

// Bounded TopDiff (Touzet KR-set) tree edit distance.
// Returns the exact TED when it is <= k; otherwise returns k+1 (over-bound),
// matching the `<= threshold` filtering convention used by the LB functions.
uint32_t tree_topdiff_bounded(rust::String a, rust::String b, int32_t k);

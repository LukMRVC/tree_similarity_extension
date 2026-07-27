#pragma once

#include <memory>
#include <string>
#include "unit_cost_model.h"
#include "string_label.h"
#include "label_dictionary.h"
#include "tree_indexer.h"
#include "node.h"
#include "apted_tree_index.h"
#include "./bracket_notation_parser.h"
#include "rust/cxx.h"

uint32_t tree_ted(rust::String a, rust::String b);

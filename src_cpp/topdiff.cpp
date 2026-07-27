#include "tree_similarity_extension/include/topdiff.h"

#include <limits>

using Label = label::StringLabel;
using CostModelLD = cost_model::UnitCostModelLD<Label>;
using LabelDictionary = label::LabelDictionary<Label>;
using TopDiff = ted::TouzetKRSetTreeIndex<CostModelLD, node::TreeIndexTouzetKRSet>;

uint32_t tree_topdiff_bounded(rust::String a, rust::String b, int32_t k)
{
    std::string t1(a);
    std::string t2(b);
    LabelDictionary ld;
    CostModelLD ucm(ld);
    parser::BracketNotationParser<Label> bnp;

    auto parsed_1 = bnp.parse_single(t1);
    auto parsed_2 = bnp.parse_single(t2);

    node::TreeIndexTouzetKRSet t1_idx;
    node::TreeIndexTouzetKRSet t2_idx;

    node::index_tree(t1_idx, parsed_1, ld, ucm);
    node::index_tree(t2_idx, parsed_2, ld, ucm);

    TopDiff topdiff(ucm);

    // ted_k returns the exact TED if k is a proper upper bound, otherwise an
    // upper bound that may be infinity when no mapping exists within k errors.
    double d = topdiff.ted_k(t1_idx, t2_idx, (int)k);

    // Over-bound convention: anything exceeding k (including infinity) reports
    // k+1 so callers filtering with `<= threshold` exclude the pair.
    if (d > (double)k) {
        return (uint32_t)(k + 1);
    }
    return (uint32_t)d;
}

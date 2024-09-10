#include "tree_similarity_extension/include/apted.h"

using Label = label::StringLabel;
using CostModelLD = cost_model::UnitCostModelLD<Label>;
using LabelDictionary = label::LabelDictionary<Label>;
using APTED = ted::APTEDTreeIndex<CostModelLD>;

uint32_t tree_ted(std::string const &a, std::string const &b)
{
    LabelDictionary ld;
    CostModelLD ucm(ld);
    parser::BracketNotationParser<Label> bnp;

    auto parsed_1 = bnp.parse_single(a);
    auto parsed_2 = bnp.parse_single(b);

    node::TreeIndexAPTED t1_idx;
    node::TreeIndexAPTED t2_idx;

    node::index_tree(t1_idx, parsed_1, ld, ucm);
    node::index_tree(t2_idx, parsed_2, ld, ucm);

    APTED apted(ucm);

    return (uint32_t)apted.ted(t1_idx, t2_idx);
}

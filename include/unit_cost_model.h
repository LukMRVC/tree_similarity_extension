#pragma once

#ifndef UNIT_COST_MODEL
#define UNIT_COST_MODEL

#include <limits>
#include "./node.h"
#include "./label_dictionary.h"
#include "./string_label.h"

namespace cost_model {

  template <typename Label>
  struct UnitCostModelLD {
    /// Label dictionary in case the labels are needed for cost calculations.
    const label::LabelDictionary<Label>& ld_;
    
    /// Constructor. Takes a LabelDictionary.
    UnitCostModelLD(label::LabelDictionary<Label>& ld);
    
    /// Basic rename cost function (unit cost model).
    /**
    * \param label_id_1 Source label id.
    * \param label_id_2 Destination label id.
    * \return Double cost of renaming label_id_1 to label_id_1.
    */
    double ren(const int label_id_1, const int label_id_2) const;
    
    /// Basic delete cost function.
    /**
    * \param label_id Label id to be deleted.
    * \return Double cost of deleting a node with label_id.
    */
    double del(const int label_id) const;
    
    /// Basic insert cost function.
    /**
    * \param label_id Label id to be inserted.
    * \return Double cost of inserting a node with label_id.
    */
    double ins(const int label_id) const;
  };

}
#endif
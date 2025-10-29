// SPDX-License-Identifier: Apache-2.0
// SPDX-FileCopyrightText: Copyright the Vortex contributors

//! Reader implementation for ListLayouts containing lists with reified offsets buffers.

use std::collections::BTreeSet;
use std::ops::Range;
use std::sync::Arc;

use async_trait::async_trait;
use vortex_array::MaskFuture;
use vortex_dtype::{DType, Field, FieldMask};
use vortex_error::VortexResult;
use vortex_expr::ExprRef;
use vortex_mask::Mask;

use crate::layouts::SharedArrayFuture;
use crate::layouts::list::ListLayout;
use crate::{ArrayFuture, LayoutReader, LayoutReaderRef};

/// `LayoutReader` for the list layout holding
/// `List`-typed data.
pub struct ListReader {
    name: Arc<str>,
    layout: ListLayout,
    validity: Option<LayoutReaderRef>,
    offsets: LayoutReaderRef,
    elements: LayoutReaderRef,
}

impl ListReader {
    fn offsets_array(&self) -> VortexResult<SharedArrayFuture> {
        // Read via the layout to access the list offsets
        self.layout
    }
}

#[async_trait]
impl LayoutReader for ListReader {
    fn name(&self) -> &Arc<str> {
        todo!()
    }

    fn dtype(&self) -> &DType {
        &self.layout.dtype
    }

    fn row_count(&self) -> u64 {
        self.layout.row_count
    }

    fn register_splits(
        &self,
        field_mask: &[FieldMask],
        row_range: &Range<u64>,
        splits: &mut BTreeSet<u64>,
    ) -> VortexResult<()> {
        // TODO(aduffy): What am I supposed to do for a FieldMask of vec![Field::ElementType] ?

        if let Some(mask) = field_mask.first() {
            match mask.starting_field()?.unwrap() {
                Field::Name(_) => {
                    // Invalid type
                }
                Field::ElementType => {}
            }
        }

        // We split based on the number of reads of the offsets buffers here
        // Only register for the elements splits.
        self.elements
            .register_splits(field_mask, row_range, splits)?;

        Ok(())
    }

    fn pruning_evaluation(
        &self,
        _row_range: &Range<u64>,
        _expr: &ExprRef,
        mask: Mask,
    ) -> VortexResult<MaskFuture> {
        // TODO(aduffy): if the root is an UNNEST, we can splat the mask out into the elements
        //  space and prune that way
        Ok(MaskFuture::ready(mask))
    }

    fn filter_evaluation(
        &self,
        _row_range: &Range<u64>,
        _expr: &ExprRef,
        _mask: MaskFuture,
    ) -> VortexResult<MaskFuture> {
        todo!()
    }

    fn projection_evaluation(
        &self,
        row_range: &Range<u64>,
        expr: &ExprRef,
        mask: MaskFuture,
    ) -> VortexResult<ArrayFuture> {
        // Expression should be root. We don't know how to handle other types of expressions
        // on the root elements...do we?
        todo!()
    }
}

#[cfg(test)]
mod tests {
    //! List Layouts give us the opportunity to have deeply nested data accesses with the benefits
    //! of full vectorization.
    //!
    //! Say that we have the following schema: `{a:list({b:i32?}?)}`
    //!
    //! This can be written with the following layout tree (some simple nodes elided):
    //!
    //! ```text
    //! StructLayout
    //! |__ a: ListLayout
    //!     |__ offsets: FlatLayout
    //!     |__ elements: StructLayout
    //!         |__ b: ChunkedLayout
    //!             |_ [0]: FlatLayout
    //!             |_ [1]: FlatLayout
    //!             |_ ...
    //!             |_ [N]: FlatLayout
    //! ```
    //!
    //! The ListLayout
}

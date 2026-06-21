pub mod dispatch;

use serde::{Deserialize, Serialize};
use tench_document_core::OfficeProductKind;

pub use dispatch::{
    export_docs_content_bytes, export_kodocs_content_bytes, export_sheets_content_bytes,
    export_slides_content_bytes, serialize_docs_for_target, serialize_kodocs_for_target,
    serialize_sheets_for_target, serialize_slides_for_target, OfficeDispatchProfile, DOCS_DISPATCH,
    KODOCS_DISPATCH, SHEETS_DISPATCH, SLIDES_DISPATCH,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OfficeRuntimeProduct {
    pub product_id: &'static str,
    pub kind: OfficeProductKind,
    pub recent_scope: &'static str,
    pub recovery_scope: &'static str,
}

pub const DOCS_RUNTIME: OfficeRuntimeProduct = OfficeRuntimeProduct {
    product_id: "tench-docs",
    kind: OfficeProductKind::Docs,
    recent_scope: "documents",
    recovery_scope: "documents",
};

pub const KODOCS_RUNTIME: OfficeRuntimeProduct = OfficeRuntimeProduct {
    product_id: "tench-kodocs",
    kind: OfficeProductKind::Docs,
    recent_scope: "hwp-documents",
    recovery_scope: "hwp-documents",
};

pub const SHEETS_RUNTIME: OfficeRuntimeProduct = OfficeRuntimeProduct {
    product_id: "tench-sheets",
    kind: OfficeProductKind::Sheets,
    recent_scope: "workbooks",
    recovery_scope: "workbooks",
};

pub const SLIDES_RUNTIME: OfficeRuntimeProduct = OfficeRuntimeProduct {
    product_id: "tench-slides",
    kind: OfficeProductKind::Slides,
    recent_scope: "presentations",
    recovery_scope: "presentations",
};

pub fn office_runtime_products() -> [OfficeRuntimeProduct; 4] {
    [DOCS_RUNTIME, KODOCS_RUNTIME, SHEETS_RUNTIME, SLIDES_RUNTIME]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declares_all_office_products() {
        let products = office_runtime_products();
        assert_eq!(products.len(), 4);
        assert!(products
            .iter()
            .any(|product| product.product_id == "tench-docs"));
        assert!(products
            .iter()
            .any(|product| product.product_id == "tench-kodocs"));
        assert!(products
            .iter()
            .any(|product| product.product_id == "tench-sheets"));
        assert!(products
            .iter()
            .any(|product| product.product_id == "tench-slides"));
    }
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataBoundary {
    LocalOnly,
    LocalFirstCloudOptional,
    CloudRequired,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StorageClass {
    Config,
    UserContent,
    Cache,
    Index,
    Secret,
    Telemetry,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StorageNamespace {
    pub product_id: String,
    pub class: StorageClass,
    pub name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StoragePolicy {
    pub namespace: StorageNamespace,
    pub boundary: DataBoundary,
    pub encrypted_at_rest: bool,
    pub user_exportable: bool,
    pub retention_days: Option<u16>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfficeStorageArea {
    Config,
    RecentFiles,
    Templates,
    PromptTemplates,
    Assets,
    Cache,
    Temp,
    Recovery,
    Backups,
}

impl OfficeStorageArea {
    pub fn as_str(self) -> &'static str {
        match self {
            OfficeStorageArea::Config => "config",
            OfficeStorageArea::RecentFiles => "recent_files",
            OfficeStorageArea::Templates => "templates",
            OfficeStorageArea::PromptTemplates => "prompt_templates",
            OfficeStorageArea::Assets => "assets",
            OfficeStorageArea::Cache => "cache",
            OfficeStorageArea::Temp => "temp",
            OfficeStorageArea::Recovery => "recovery",
            OfficeStorageArea::Backups => "backups",
        }
    }
}

pub fn local_user_content_policy(
    product_id: impl Into<String>,
    name: impl Into<String>,
) -> StoragePolicy {
    StoragePolicy {
        namespace: StorageNamespace {
            product_id: product_id.into(),
            class: StorageClass::UserContent,
            name: name.into(),
        },
        boundary: DataBoundary::LocalOnly,
        encrypted_at_rest: false,
        user_exportable: true,
        retention_days: None,
    }
}

pub fn office_storage_namespace(
    product_id: impl Into<String>,
    area: OfficeStorageArea,
) -> StorageNamespace {
    StorageNamespace {
        product_id: product_id.into(),
        class: office_storage_class(area),
        name: area.as_str().to_string(),
    }
}

pub fn office_storage_policy(
    product_id: impl Into<String>,
    area: OfficeStorageArea,
) -> StoragePolicy {
    let class = office_storage_class(area);

    StoragePolicy {
        namespace: StorageNamespace {
            product_id: product_id.into(),
            class: class.clone(),
            name: area.as_str().to_string(),
        },
        boundary: DataBoundary::LocalOnly,
        encrypted_at_rest: false,
        user_exportable: matches!(class, StorageClass::UserContent | StorageClass::Config),
        retention_days: match area {
            OfficeStorageArea::Cache | OfficeStorageArea::Temp => Some(30),
            OfficeStorageArea::Recovery => Some(7),
            _ => None,
        },
    }
}

fn office_storage_class(area: OfficeStorageArea) -> StorageClass {
    match area {
        OfficeStorageArea::Config | OfficeStorageArea::RecentFiles => StorageClass::Config,
        OfficeStorageArea::Templates
        | OfficeStorageArea::PromptTemplates
        | OfficeStorageArea::Assets
        | OfficeStorageArea::Recovery
        | OfficeStorageArea::Backups => StorageClass::UserContent,
        OfficeStorageArea::Cache | OfficeStorageArea::Temp => StorageClass::Cache,
    }
}

//! Dynamic evidence attributes for profile-driven source schemas.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Namespaced source field key.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct FieldKey {
    pub namespace: String,
    pub name: String,
}

impl FieldKey {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
        }
    }

    pub fn dotted(&self) -> String {
        format!("{}.{}", self.namespace, self.name)
    }
}

/// Dynamic evidence value carried by source-profile fields.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum EvidenceValue {
    String(String),
    Number(f64),
    Bool(bool),
    Timestamp(DateTime<Utc>),
    StringList(Vec<String>),
    Json(serde_json::Value),
}

impl EvidenceValue {
    pub fn value_type(&self) -> EvidenceFieldType {
        match self {
            Self::String(_) => EvidenceFieldType::String,
            Self::Number(_) => EvidenceFieldType::Number,
            Self::Bool(_) => EvidenceFieldType::Bool,
            Self::Timestamp(_) => EvidenceFieldType::Timestamp,
            Self::StringList(_) => EvidenceFieldType::StringList,
            Self::Json(_) => EvidenceFieldType::Json,
        }
    }
}

/// One dynamic evidence attribute.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceAttribute {
    pub value: EvidenceValue,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub source_path: Option<String>,
}

impl EvidenceAttribute {
    pub fn new(value: EvidenceValue) -> Self {
        Self {
            value,
            confidence: None,
            source_path: None,
        }
    }
}

/// Dynamic attribute map keyed by namespaced source fields.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EvidenceAttributes {
    #[serde(default)]
    pub values: BTreeMap<FieldKey, EvidenceAttribute>,
}

impl EvidenceAttributes {
    pub fn insert(&mut self, key: FieldKey, value: EvidenceValue) {
        self.values.insert(key, EvidenceAttribute::new(value));
    }

    pub fn get(&self, namespace: &str, name: &str) -> Option<&EvidenceAttribute> {
        self.values.get(&FieldKey::new(namespace, name))
    }

    pub fn from_string_map(
        namespace: impl Into<String>,
        values: &BTreeMap<String, String>,
    ) -> Self {
        let namespace = namespace.into();
        Self {
            values: values
                .iter()
                .map(|(name, value)| {
                    (
                        FieldKey::new(namespace.clone(), name.clone()),
                        EvidenceAttribute::new(EvidenceValue::String(value.clone())),
                    )
                })
                .collect(),
        }
    }
}

/// Declared type for a source-profile field.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceFieldType {
    String,
    Number,
    Bool,
    Timestamp,
    StringList,
    Json,
}

/// Canonical slot a dynamic field may project into.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum CanonicalEvidenceSlot {
    Title,
    Identifier,
    PublishedAt,
    UpdatedAt,
    License,
    Jurisdiction,
    EvidenceKind,
    Other(String),
}

impl CanonicalEvidenceSlot {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Title => "title",
            Self::Identifier => "identifier",
            Self::PublishedAt => "published_at",
            Self::UpdatedAt => "updated_at",
            Self::License => "license",
            Self::Jurisdiction => "jurisdiction",
            Self::EvidenceKind => "evidence_kind",
            Self::Other(value) => value.as_str(),
        }
    }

    pub fn wire_label(&self) -> String {
        match self {
            Self::Other(value) if value.starts_with("other:") => value.clone(),
            Self::Other(value) => format!("other:{value}"),
            _ => self.as_str().to_string(),
        }
    }

    pub fn from_label(label: impl AsRef<str>) -> Self {
        let label = label.as_ref().trim();
        match label {
            "title" => Self::Title,
            "identifier" => Self::Identifier,
            "published_at" => Self::PublishedAt,
            "updated_at" => Self::UpdatedAt,
            "license" => Self::License,
            "jurisdiction" => Self::Jurisdiction,
            "evidence_kind" => Self::EvidenceKind,
            other if other.starts_with("other:") => {
                Self::Other(other.strip_prefix("other:").unwrap_or_default().to_string())
            }
            other => Self::Other(other.to_string()),
        }
    }
}

impl Serialize for CanonicalEvidenceSlot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.wire_label())
    }
}

impl<'de> Deserialize<'de> for CanonicalEvidenceSlot {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let label = String::deserialize(deserializer)?;
        Ok(Self::from_label(label))
    }
}

/// Field declaration used by profile-driven evidence schemas.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FieldDescriptor {
    pub key: FieldKey,
    pub value_type: EvidenceFieldType,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub canonical_slot: Option<CanonicalEvidenceSlot>,
}

impl FieldDescriptor {
    pub fn new(key: FieldKey, value_type: EvidenceFieldType) -> Self {
        Self {
            key,
            value_type,
            required: false,
            aliases: Vec::new(),
            canonical_slot: None,
        }
    }
}

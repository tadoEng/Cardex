use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, CardexError>;

#[derive(Debug, thiserror::Error)]
pub enum CardexError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("search index error: {0}")]
    Search(#[from] tantivy::TantivyError),
    #[error("query parse error: {0}")]
    Query(#[from] tantivy::query::QueryParserError),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("missing artifact: {0}")]
    MissingArtifact(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Toc {
    pub entries: Vec<TocEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TocEntry {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<String>,
    pub depth: usize,
    pub ancestors: Vec<String>,
    pub kind: PageKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overload_of: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PageKind {
    Method,
    Interface,
    Enum,
    Property,
    Page,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApiCard {
    pub page_id: String,
    pub title: String,
    pub kind: PageKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overload_of: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_cs: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_vb: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<Parameter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub raw_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOptions {
    pub source_dir: PathBuf,
    pub out_dir: PathBuf,
    pub corpus: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildReport {
    pub corpus: String,
    pub pages: usize,
    pub hhc_entries: usize,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub corpus: String,
    pub schema_version: u32,
    pub pages: usize,
    pub generated_by: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocGraph {
    #[serde(default)]
    pub members: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    pub related: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchHit {
    pub page_id: String,
    pub title: String,
    pub kind: PageKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub score: f32,
}

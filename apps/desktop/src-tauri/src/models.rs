//! Wire types of the command contract. Mirrors `src/lib/ipc-types.ts`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FileKind {
    Project,
    Template,
    Preset,
    Xml,
    Sidecar,
    Unknown,
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum Platform {
    Windows,
    MacIntel,
    MacArm,
    Unknown,
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum Problem {
    NotAeFile,
    Sidecar,
    XmlProject,
    Truncated,
    Damaged,
    TooNew,
    Unreadable,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub build: u32,
    pub platform: Platform,
    pub release: bool,
    pub name: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Format {
    pub major: u32,
    pub minor: u32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub id: String,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub kind: FileKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<Version>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<Format>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem: Option<Problem>,
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetStatus {
    Verified,
    Experimental,
}

#[derive(Serialize)]
pub struct TargetInfo {
    pub major: u32,
    pub name: String,
    pub status: TargetStatus,
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warn,
    Block,
}

#[derive(Serialize)]
pub struct Note {
    pub severity: Severity,
    pub code: String,
    pub params: Map<String, Value>,
}

#[derive(Serialize)]
pub struct Preflight {
    pub target: u32,
    pub blocked: bool,
    pub notes: Vec<Note>,
}

#[derive(Deserialize)]
pub struct Job {
    pub id: String,
    pub path: String,
    pub target: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LocationKeyword {
    NextToSource,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Location {
    Keyword(#[allow(dead_code)] LocationKeyword),
    Folder { folder: String },
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum OnExists {
    Rename,
    Overwrite,
    Skip,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputOptions {
    pub location: Location,
    pub pattern: String,
    pub on_exists: OnExists,
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ProgressState {
    Queued,
    Running,
    Done,
    Failed,
    Skipped,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub id: String,
    pub state: ProgressState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
}

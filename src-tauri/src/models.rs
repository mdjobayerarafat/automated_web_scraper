use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingJob {
    pub id: Option<i64>,
    pub name: String,
    pub url: String,
    pub selector_type: SelectorType,
    pub selector: String,
    pub data_type: DataType,
    pub schedule: String,
    pub user_agent: Option<String>,
    pub proxy_url: Option<String>,
    pub is_active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectorType {
    CSS,
    Regex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Text,
    Attribute(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapingResult {
    pub id: Option<i64>,
    pub job_id: i64,
    pub scraped_data: String,
    pub timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_server: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
    pub sender_email: String,
    pub receiver_email: String,
    pub use_tls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportRequest {
    pub job_id: i64,
    pub format: ExportFormat,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndividualExportRequest {
    pub result_id: i64,
    pub format: ExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    CSV,
    JSON,
    HTML,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStats {
    pub total_jobs: i64,
    pub active_jobs: i64,
    pub total_results: i64,
    pub last_run: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub modified_timestamp: u64,
    pub file_type: String,
}

impl std::fmt::Display for SelectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectorType::CSS => write!(f, "css"),
            SelectorType::Regex => write!(f, "regex"),
        }
    }
}

impl std::str::FromStr for SelectorType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "css" => Ok(SelectorType::CSS),
            "regex" => Ok(SelectorType::Regex),
            _ => Err(anyhow::anyhow!("Invalid selector type: {}", s)),
        }
    }
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Text => write!(f, "text"),
            DataType::Attribute(attr) => write!(f, "attribute:{}", attr),
        }
    }
}

impl std::str::FromStr for DataType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "text" {
            Ok(DataType::Text)
        } else if let Some(attr) = s.strip_prefix("attribute:") {
            Ok(DataType::Attribute(attr.to_string()))
        } else {
            Err(anyhow::anyhow!("Invalid data type: {}", s))
        }
    }
}
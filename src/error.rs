use std::io;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Json(serde_json::Error),
    Network(reqwest::Error),
    Anyhow(anyhow::Error),
    Zip(zip::result::ZipError),
    Walkdir(walkdir::Error),
    Parse(ParseIntError),
    NotFound(String),
    InvalidData(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(e) => write!(f, "IO错误: {}", e),
            CliError::Json(e) => write!(f, "JSON解析错误: {}", e),
            CliError::Network(e) => write!(f, "网络错误: {}", e),
            CliError::Anyhow(e) => write!(f, "{}", e),
            CliError::Zip(e) => write!(f, "压缩错误: {}", e),
            CliError::Walkdir(e) => write!(f, "目录遍历错误: {}", e),
            CliError::Parse(e) => write!(f, "解析错误: {}", e),
            CliError::NotFound(msg) => write!(f, "未找到: {}", msg),
            CliError::InvalidData(msg) => write!(f, "无效数据: {}", msg),
        }
    }
}

impl std::error::Error for CliError {}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> Self {
        CliError::Io(err)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(err: serde_json::Error) -> Self {
        CliError::Json(err)
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        CliError::Network(err)
    }
}

impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        CliError::Anyhow(err)
    }
}

impl From<zip::result::ZipError> for CliError {
    fn from(err: zip::result::ZipError) -> Self {
        CliError::Zip(err)
    }
}

impl From<walkdir::Error> for CliError {
    fn from(err: walkdir::Error) -> Self {
        CliError::Walkdir(err)
    }
}

impl From<ParseIntError> for CliError {
    fn from(err: ParseIntError) -> Self {
        CliError::Parse(err)
    }
}

pub type Result<T> = std::result::Result<T, CliError>;
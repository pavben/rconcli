use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub password: String,
}

#[derive(Debug, Deserialize)]
struct ServerConfig {
    servers: Vec<Server>,
}

#[derive(Debug)]
pub enum ServerLookupError {
    NotFound,
    Ambiguous(Vec<String>),
    FileError(String),
    ParseError(String),
}

impl std::fmt::Display for ServerLookupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerLookupError::NotFound => write!(f, "No server found with the given ID prefix"),
            ServerLookupError::Ambiguous(matches) => {
                write!(f, "Ambiguous ID prefix. Multiple matches found: {}", matches.join(", "))
            }
            ServerLookupError::FileError(msg) => write!(f, "File error: {}", msg),
            ServerLookupError::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for ServerLookupError {}

pub async fn load_servers(file_path: &PathBuf) -> Result<Vec<Server>, ServerLookupError> {
    let content = fs::read_to_string(file_path)
        .await
        .map_err(|e| ServerLookupError::FileError(format!("Failed to read file {}: {}", file_path.display(), e)))?;
    
    let config: ServerConfig = serde_yaml::from_str(&content)
        .map_err(|e| ServerLookupError::ParseError(format!("Failed to parse YAML: {}", e)))?;
    
    Ok(config.servers)
}

pub fn find_server_by_prefix<'a>(servers: &'a[Server], prefix: &str) -> Result<&'a Server, ServerLookupError> {
    let matches: Vec<&Server> = servers
        .iter()
        .filter(|server| server.id.starts_with(prefix))
        .collect();
    
    match matches.len() {
        0 => Err(ServerLookupError::NotFound),
        1 => Ok(matches[0]),
        _ => {
            let ids: Vec<String> = matches.iter().map(|s| s.id.clone()).collect();
            Err(ServerLookupError::Ambiguous(ids))
        }
    }
}

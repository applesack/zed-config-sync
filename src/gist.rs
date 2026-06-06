use std::collections::HashMap;

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use serde::Deserialize;

use crate::config::Config;

const USER_AGENT: &str = concat!("zed-config-sync/", env!("CARGO_PKG_VERSION"));
const GIST_API: &str = "https://api.github.com/gists";

#[derive(Deserialize)]
struct GistResponse {
    files: HashMap<String, GistFileInfo>,
}

#[derive(Deserialize)]
struct GistFileInfo {
    raw_url: String,
}

pub struct GistClient {
    pub token: String,
    pub gist_id: String,
}

impl GistClient {
    pub fn new(token: String, gist_id: String) -> Self {
        Self { token, gist_id }
    }

    fn api_url(&self) -> String {
        format!("{}/{}", GIST_API, self.gist_id)
    }

    fn fetch_gist(&self) -> Result<GistResponse> {
        let resp: GistResponse = ureq::get(&self.api_url())
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("User-Agent", USER_AGENT)
            .call()?
            .into_json()?;
        Ok(resp)
    }

    pub fn list_cfg_files(&self) -> Result<Vec<String>> {
        let gist = self.fetch_gist()?;
        let mut files: Vec<String> = gist
            .files
            .into_keys()
            .filter(|name| name.starts_with("cfg_"))
            .collect();
        files.sort();
        Ok(files)
    }

    pub fn download_file(&self, name: &str) -> Result<Vec<u8>> {
        let gist = self.fetch_gist()?;
        let file = gist
            .files
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("File '{}' not found in gist", name))?;
        // Fetch via raw_url to avoid truncation on large files
        let encoded = ureq::get(&file.raw_url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("User-Agent", USER_AGENT)
            .call()?
            .into_string()?;
        Ok(B64.decode(encoded.trim())?)
    }

    pub fn get_history(&self) -> Result<HashMap<String, String>> {
        let gist = self.fetch_gist()?;
        let Some(file) = gist.files.get("history.json") else {
            return Ok(HashMap::new());
        };
        let content = ureq::get(&file.raw_url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("User-Agent", USER_AGENT)
            .call()?
            .into_string()?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    }

    pub fn patch_files(&self, ops: &HashMap<String, Option<String>>) -> Result<()> {
        let mut files = serde_json::Map::new();
        for (name, content) in ops {
            match content {
                Some(c) => {
                    files.insert(name.clone(), serde_json::json!({ "content": c }));
                }
                None => {
                    files.insert(name.clone(), serde_json::Value::Null);
                }
            }
        }
        let body = serde_json::json!({ "files": files });
        ureq::patch(&self.api_url())
            .set("Authorization", &format!("Bearer {}", self.token))
            .set("User-Agent", USER_AGENT)
            .set("Content-Type", "application/json")
            .send_string(&body.to_string())?;
        Ok(())
    }
}

/// Build a GistClient from saved config, erroring if token or gist_id is unset.
pub fn client_from_config() -> Result<GistClient> {
    let config = Config::load_or_default();
    if config.github_token.is_empty() {
        anyhow::bail!("No GitHub token configured. Run: zed-config set token <TOKEN>");
    }
    if config.gist_id.is_empty() {
        anyhow::bail!("No Gist ID configured. Run: zed-config set gist <GIST_ID>");
    }
    Ok(GistClient::new(config.github_token, config.gist_id))
}

pub fn validate_token(token: &str) -> Result<bool> {
    match ureq::get("https://api.github.com/user")
        .set("Authorization", &format!("Bearer {}", token))
        .set("User-Agent", USER_AGENT)
        .call()
    {
        Ok(_) => Ok(true),
        Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => Ok(false),
        Err(e) => Err(anyhow::anyhow!("Network error: {}", e)),
    }
}

pub fn validate_gist(token: &str, gist_id: &str) -> Result<bool> {
    match ureq::get(&format!("{}/{}", GIST_API, gist_id))
        .set("Authorization", &format!("Bearer {}", token))
        .set("User-Agent", USER_AGENT)
        .call()
    {
        Ok(_) => Ok(true),
        Err(ureq::Error::Status(404, _)) => Ok(false),
        Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => Err(
            anyhow::anyhow!("Token is invalid or lacks permission to access this gist"),
        ),
        Err(e) => Err(anyhow::anyhow!("Network error: {}", e)),
    }
}

use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

pub struct SeafileClient {
    pub url: String,
    pub repo_id: String,
    token: String,
    client: Client,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FolderInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub size: Option<i64>,
}

impl SeafileClient {
    pub fn new(url: String, token: String, repo_id: String) -> Arc<Self> {
        Arc::new(Self {
            url,
            repo_id,
            token,
            client: Client::new(),
        })
    }

    fn auth_header(&self) -> String {
        format!("Token {}", self.token)
    }

    /// List top-level folders in the repo root.
    pub async fn list_folders(&self) -> Result<Vec<FolderInfo>> {
        let url = format!("{}/api2/repos/{}/dir/", self.url, self.repo_id);
        let items: Vec<FolderInfo> = self
            .client
            .get(&url)
            .query(&[("p", "/")])
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(items.into_iter().filter(|i| i.entry_type == "dir").collect())
    }

    /// List files inside a folder (folder = bare name, no leading slash).
    pub async fn list_files(&self, folder: &str) -> Result<Vec<FileInfo>> {
        let url = format!("{}/api2/repos/{}/dir/", self.url, self.repo_id);
        let path = format!("/{}", folder);
        let items: Vec<FileInfo> = self
            .client
            .get(&url)
            .query(&[("p", path.as_str())])
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(items.into_iter().filter(|i| i.entry_type == "file").collect())
    }

    /// Get a 1-hour temporary download URL for a file.
    /// `path` is relative to repo root without a leading slash (e.g. "Folder/video.mp4").
    pub async fn get_download_url(&self, path: &str) -> Result<String> {
        let url = format!("{}/api2/repos/{}/file/", self.url, self.repo_id);
        let api_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        let download_url: String = self
            .client
            .get(&url)
            .query(&[("p", api_path.as_str()), ("reuse", "1")])
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(download_url)
    }
}

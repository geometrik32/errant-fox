use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;

pub struct SeafileClient {
    pub url: String,
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

#[derive(Deserialize)]
struct DirResponse {
    dirent_list: Vec<serde_json::Value>,
}

impl SeafileClient {
    pub fn new(url: String, token: String) -> Arc<Self> {
        Arc::new(Self {
            url,
            token,
            client: Client::new(),
        })
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    /// List top-level folders in the repo root.
    pub async fn list_folders(&self) -> Result<Vec<FolderInfo>> {
        let url = format!("{}/api/v2.1/via-repo-token/dir/", self.url);
        let resp: DirResponse = self
            .client
            .get(&url)
            .query(&[("path", "/")])
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let folders = resp
            .dirent_list
            .into_iter()
            .filter_map(|v| {
                let entry_type = v["type"].as_str()?.to_string();
                let name = v["name"].as_str()?.to_string();
                if entry_type == "dir" {
                    Some(FolderInfo { name, entry_type })
                } else {
                    None
                }
            })
            .collect();
        Ok(folders)
    }

    /// List files inside a folder (folder = bare name, no leading slash).
    pub async fn list_files(&self, folder: &str) -> Result<Vec<FileInfo>> {
        let url = format!("{}/api/v2.1/via-repo-token/dir/", self.url);
        let path = format!("/{}", folder);
        let resp: DirResponse = self
            .client
            .get(&url)
            .query(&[("path", path.as_str())])
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        let files = resp
            .dirent_list
            .into_iter()
            .filter_map(|v| {
                let entry_type = v["type"].as_str()?.to_string();
                let name = v["name"].as_str()?.to_string();
                if entry_type == "file" {
                    let size = v["size"].as_i64();
                    Some(FileInfo { name, entry_type, size })
                } else {
                    None
                }
            })
            .collect();
        Ok(files)
    }

    /// Fetch a byte range from a file, proxying through a fresh download URL.
    /// Returns the raw reqwest Response so the caller can stream it.
    pub async fn fetch_range(
        &self,
        path: &str,
        range: Option<&str>,
    ) -> Result<reqwest::Response> {
        let download_url = self.get_download_url(path).await?;
        let mut req = self.client.get(&download_url);
        if let Some(r) = range {
            req = req.header("Range", r);
        }
        let resp = req.send().await?.error_for_status()?;
        Ok(resp)
    }

    /// Get a temporary download URL for a file.
    /// `path` is relative to repo root without a leading slash (e.g. "Folder/video.mp4").
    pub async fn get_download_url(&self, path: &str) -> Result<String> {
        let url = format!("{}/api/v2.1/via-repo-token/download-link/", self.url);
        let api_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        let download_url: String = self
            .client
            .get(&url)
            .query(&[("path", api_path.as_str())])
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(download_url)
    }

    /// Get a temporary upload URL for a directory.
    pub async fn get_upload_link(&self, parent_dir: &str) -> Result<String> {
        let url = format!("{}/api/v2.1/via-repo-token/upload-link/", self.url);
        let upload_url: String = self
            .client
            .get(&url)
            .query(&[("path", parent_dir)])
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(upload_url)
    }

    /// Upload a file, replacing it if it exists.
    /// `path` is the full relative path (e.g. "Folder/video.mp4").
    pub async fn upload_file(&self, path: &str, file_bytes: Vec<u8>) -> Result<()> {
        let path = if path.starts_with('/') { path.to_string() } else { format!("/{}", path) };
        let path_obj = std::path::Path::new(&path);
        let parent_dir = path_obj.parent().unwrap_or(std::path::Path::new("/")).to_string_lossy().to_string();
        let file_name = path_obj.file_name().unwrap_or_default().to_string_lossy().to_string();

        let upload_url = self.get_upload_link(&parent_dir).await?;

        let part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_name.clone());

        let form = reqwest::multipart::Form::new()
            .text("parent_dir", parent_dir.clone())
            .text("replace", "1")
            .part("file", part);

        self.client
            .post(&upload_url)
            .header("Authorization", self.auth_header())
            .multipart(form)
            .send()
            .await?
            .error_for_status()?;
            
        Ok(())
    }
}

use std::sync::Arc;

use anyhow::Result;
use reqwest::{Client, Url};
use serde::Deserialize;

pub struct SeafileClient {
    pub url: String,
    pub internal_url: Option<String>,
    public_host: Option<String>,
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
    pub fn new(url: String, token: String, internal_url: Option<String>) -> Arc<Self> {
        let public_host = Url::parse(&url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()));

        let internal_url = internal_url.map(|u| u.trim_end_matches('/').to_string());

        Arc::new(Self {
            url: url.trim_end_matches('/').to_string(),
            internal_url,
            public_host,
            token,
            client: Client::builder()
                .connect_timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_else(|_| Client::new()),
        })
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }

    /// Rewrites a public Seafile URL (e.g. from FILE_SERVER_ROOT) into an internal network URL
    /// if `internal_url` is configured.
    pub fn to_internal_url(&self, original_url: &str) -> String {
        if let Some(internal) = &self.internal_url {
            if let Ok(parsed) = Url::parse(original_url) {
                let path_and_query = match parsed.query() {
                    Some(q) => format!("{}?{}", parsed.path(), q),
                    None => parsed.path().to_string(),
                };
                return format!("{}{}", internal, path_and_query);
            }
        }
        original_url.to_string()
    }

    /// Internal helper to execute API requests with fallback to public URL if internal URL fails.
    async fn send_api_get(&self, path: &str, query: &[(&str, &str)]) -> Result<reqwest::Response> {
        let clean_path = path.trim_start_matches('/');
        if let Some(internal) = &self.internal_url {
            let internal_api = format!("{}/api/v2.1/via-repo-token/{}", internal, clean_path);
            let mut req = self
                .client
                .get(&internal_api)
                .query(query)
                .header("Authorization", self.auth_header());
            if let Some(host) = &self.public_host {
                req = req.header("Host", host);
            }
            match req.send().await {
                Ok(resp) if resp.status().is_success() => return Ok(resp),
                Ok(resp) => {
                    tracing::warn!(
                        "Internal Seafile API GET {} returned {}, falling back to public URL",
                        clean_path,
                        resp.status()
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        "Internal Seafile API GET {} failed ({}), falling back to public URL",
                        clean_path,
                        e
                    );
                }
            }
        }

        // Public URL / Default
        let public_api = format!("{}/api/v2.1/via-repo-token/{}", self.url, clean_path);
        let resp = self
            .client
            .get(&public_api)
            .query(query)
            .header("Authorization", self.auth_header())
            .send()
            .await?
            .error_for_status()?;
        Ok(resp)
    }

    /// List top-level folders in the repo root.
    pub async fn list_folders(&self) -> Result<Vec<FolderInfo>> {
        let resp: DirResponse = self
            .send_api_get("dir/", &[("path", "/")])
            .await?
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
        let path = format!("/{}", folder);
        let resp: DirResponse = self
            .send_api_get("dir/", &[("path", path.as_str())])
            .await?
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

    /// Fetch raw download URL directly from Seafile API without internal rewrite.
    pub async fn get_raw_download_url(&self, path: &str) -> Result<String> {
        let api_path = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        let download_url: String = self
            .send_api_get("download-link/", &[("path", api_path.as_str())])
            .await?
            .json()
            .await?;
        Ok(download_url)
    }

    /// Get a download URL for a file (rewritten for internal container access if configured).
    /// `path` is relative to repo root without a leading slash (e.g. "Folder/video.mp4").
    pub async fn get_download_url(&self, path: &str) -> Result<String> {
        let raw_url = self.get_raw_download_url(path).await?;
        Ok(self.to_internal_url(&raw_url))
    }

    /// Fetch a byte range from a file, proxying through a fresh download URL.
    /// Returns the raw reqwest Response so the caller can stream it.
    pub async fn fetch_range(
        &self,
        path: &str,
        range: Option<&str>,
    ) -> Result<reqwest::Response> {
        let raw_url = self.get_raw_download_url(path).await?;
        let internal_url = self.to_internal_url(&raw_url);

        let mut req = self.client.get(&internal_url);
        if let Some(r) = range {
            req = req.header("Range", r);
        }
        if self.internal_url.is_some() {
            if let Some(host) = &self.public_host {
                req = req.header("Host", host);
            }
        }

        match req.send().await {
            Ok(resp) if resp.status().is_success() || resp.status().as_u16() == 206 => Ok(resp),
            Ok(resp) if internal_url != raw_url => {
                tracing::warn!(
                    "Internal Seafile fetch returned {}, falling back to raw download URL",
                    resp.status()
                );
                let mut fallback_req = self.client.get(&raw_url);
                if let Some(r) = range {
                    fallback_req = fallback_req.header("Range", r);
                }
                let fallback_resp = fallback_req.send().await?.error_for_status()?;
                Ok(fallback_resp)
            }
            Ok(resp) => Err(resp.error_for_status().unwrap_err().into()),
            Err(e) if internal_url != raw_url => {
                tracing::warn!(
                    "Internal Seafile fetch failed ({}), falling back to raw download URL",
                    e
                );
                let mut fallback_req = self.client.get(&raw_url);
                if let Some(r) = range {
                    fallback_req = fallback_req.header("Range", r);
                }
                let fallback_resp = fallback_req.send().await?.error_for_status()?;
                Ok(fallback_resp)
            }
            Err(e) => Err(e.into()),
        }
    }

    /// Get a temporary upload URL for a directory.
    pub async fn get_upload_link(&self, parent_dir: &str) -> Result<String> {
        let upload_url: String = self
            .send_api_get("upload-link/", &[("path", parent_dir)])
            .await?
            .json()
            .await?;
        Ok(self.to_internal_url(&upload_url))
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

        let mut req = self.client
            .post(&upload_url)
            .header("Authorization", self.auth_header());

        if self.internal_url.is_some() {
            if let Some(host) = &self.public_host {
                req = req.header("Host", host);
            }
        }

        req.multipart(form)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_internal_url_rewriting() {
        let client = SeafileClient::new(
            "https://seafile.aat-terra.ru".to_string(),
            "dummy_token".to_string(),
            Some("http://seafile".to_string()),
        );

        let public_download = "https://seafile.aat-terra.ru/seafhttp/files/2c5d427a-e4dc-40fc-8c17-079d5d0454b0/GX014584.mp4";
        let internal = client.to_internal_url(public_download);
        assert_eq!(
            internal,
            "http://seafile/seafhttp/files/2c5d427a-e4dc-40fc-8c17-079d5d0454b0/GX014584.mp4"
        );

        let upload_url = "https://seafile.aat-terra.ru/seafhttp/upload-api/01234567";
        let internal_upload = client.to_internal_url(upload_url);
        assert_eq!(
            internal_upload,
            "http://seafile/seafhttp/upload-api/01234567"
        );
    }

    #[test]
    fn test_to_internal_url_without_internal() {
        let client = SeafileClient::new(
            "https://seafile.aat-terra.ru".to_string(),
            "dummy_token".to_string(),
            None,
        );

        let public_download = "https://seafile.aat-terra.ru/seafhttp/files/xxx/test.mp4";
        let result = client.to_internal_url(public_download);
        assert_eq!(result, public_download);
    }

    #[test]
    fn test_to_internal_url_trailing_slashes() {
        let client = SeafileClient::new(
            "https://seafile.aat-terra.ru/".to_string(),
            "dummy_token".to_string(),
            Some("http://seafile:8080/".to_string()),
        );

        let public_download = "https://seafile.aat-terra.ru/seafhttp/files/xxx/test.mp4?token=123";
        let result = client.to_internal_url(public_download);
        assert_eq!(
            result,
            "http://seafile:8080/seafhttp/files/xxx/test.mp4?token=123"
        );
    }
}

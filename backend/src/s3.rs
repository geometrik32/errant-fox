use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{BehaviorVersion, Region},
    presigning::PresigningConfig,
    Client,
};

pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub fn new(
        endpoint: String,
        bucket: String,
        access_key: String,
        secret_key: String,
        region: String,
    ) -> Arc<Self> {
        let creds = Credentials::new(&access_key, &secret_key, None, None, "ef_static");
        let config = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .endpoint_url(endpoint)
            .region(Region::new(region))
            .credentials_provider(creds)
            .force_path_style(true)
            .build();
        Arc::new(Self {
            client: Client::from_conf(config),
            bucket,
        })
    }

    /// List all object keys in the bucket.
    pub async fn list_objects(&self) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        let mut continuation: Option<String> = None;
        loop {
            let mut req = self.client.list_objects_v2().bucket(&self.bucket);
            if let Some(tok) = continuation.take() {
                req = req.continuation_token(tok);
            }
            let page = req.send().await?;
            for obj in page.contents() {
                if let Some(key) = obj.key() {
                    keys.push(key.to_string());
                }
            }
            if page.is_truncated().unwrap_or(false) {
                continuation = page.next_continuation_token().map(ToOwned::to_owned);
            } else {
                break;
            }
        }
        Ok(keys)
    }

    /// Generate a presigned GET URL for an object.
    pub async fn get_presigned_url(&self, key: &str, expires_secs: u64) -> Result<String> {
        let ps_config = PresigningConfig::expires_in(Duration::from_secs(expires_secs))?;
        let presigned = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(ps_config)
            .await?;
        Ok(presigned.uri().to_string())
    }
}

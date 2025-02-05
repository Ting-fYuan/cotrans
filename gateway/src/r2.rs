use std::{collections::HashMap, ops::Deref};

use axum::body::Bytes;
use reqwest::Result;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug)]
pub struct R2Inner {
  pub private: R2Bucket,
  pub public: R2BucketPublic,
}

impl R2Inner {
  pub fn new(private: R2Bucket, public: R2BucketPublic) -> Self {
    Self { private, public }
  }
}

#[derive(Debug)]
pub struct R2Bucket {
  client: reqwest::Client,
  base: String,
  secret: String,
}

impl R2Bucket {
  pub fn new(client: reqwest::Client, base: String, secret: String) -> Self {
    Self {
      client,
      base,
      secret,
    }
  }

  pub async fn get(&self, key: &str) -> Result<Bytes> {
    debug!(key = %key, "get object from r2");
    self
      .client
      .get(&format!("{}/{}", self.base, key))
      .header("x-secret", &self.secret)
      .send()
      .await?
      .bytes()
      .await
  }

  pub async fn put(&self, key: &str, value: &Bytes) -> Result<()> {
    debug!(key = %key, file_len = %bytefmt::format(value.len() as u64), "put object to r2");
    self
      .client
      .put(&format!("{}/{}", self.base, key))
      .header("x-secret", &self.secret)
      .body(value.clone())
      .send()
      .await?
      .error_for_status()?;
    Ok(())
  }

  pub async fn delete(&self, key: &str) -> Result<()> {
    debug!(key = %key, "delete object from r2");
    self
      .client
      .delete(&format!("{}/{}", self.base, key))
      .header("x-secret", &self.secret)
      .send()
      .await?
      .error_for_status()?;
    Ok(())
  }
}

#[derive(Debug)]
pub struct R2BucketPublic {
  bucket: R2Bucket,
  public_base: String,
}

impl Deref for R2BucketPublic {
  type Target = R2Bucket;

  fn deref(&self) -> &Self::Target {
    &self.bucket
  }
}

impl R2BucketPublic {
  pub fn new(client: reqwest::Client, base: String, secret: String, public_base: String) -> Self {
    Self {
      bucket: R2Bucket::new(client, base, secret),
      public_base,
    }
  }

  pub fn public_url(&self, key: &str) -> String {
    format!("{}/{}", self.public_base, key)
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Object {
  key: String,
  version: String,
  size: u32,
  etag: String,
  #[serde(rename = "httpEtag")]
  http_etag: String,
  uploaded: String,
  #[serde(rename = "httpMetadata")]
  http_metadata: R2HttpMetadata,
  #[serde(rename = "customMetadata")]
  custom_metadata: HashMap<String, String>,
  range: R2Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2HttpMetadata {
  #[serde(rename = "contentType")]
  content_type: Option<String>,
  #[serde(rename = "contentLanguage")]
  content_language: Option<String>,
  #[serde(rename = "contentDisposition")]
  content_disposition: Option<String>,
  #[serde(rename = "contentEncoding")]
  content_encoding: Option<String>,
  #[serde(rename = "cacheControl")]
  cache_control: Option<String>,
  #[serde(rename = "cacheExpiry")]
  cache_expiry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct R2Range {
  pub offset: Option<u32>,
  pub length: Option<u32>,
  pub suffix: Option<u32>,
}

pub fn upload_image_key(sha: &str) -> String {
  format!("upload/{sha}.png")
}

pub fn translation_mask_key(task_id: &str) -> String {
  format!("mask/{task_id}.png")
}

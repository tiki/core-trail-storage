/*
 * Copyright (c) TIKI Inc.
 * MIT license. See LICENSE file in root directory.
 */

use super::super::utils::S3Client;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignerModel {
    key: String,
    #[serde(default = "Utc::now")]
    created: DateTime<Utc>,
}

#[allow(unused)]
impl SignerModel {
    pub async fn write(client: &S3Client, path: &str, key: &str) -> Result<Self, Box<dyn Error>> {
        let model = Self {
            key: key.to_string(),
            created: Utc::now(),
        };
        let body = serde_json::to_string(&model)?.as_bytes().to_vec();
        client.write(&path, &body).await?;
        Ok(model)
    }

    pub async fn read(client: &S3Client, path: &str) -> Result<Self, Box<dyn Error>> {
        let body = client.read(&path).await?;
        let res: Self = serde_json::from_str(&String::from_utf8(body)?)?;
        Ok(res)
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn created(&self) -> DateTime<Utc> {
        self.created
    }
}

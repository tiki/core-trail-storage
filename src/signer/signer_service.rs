/*
 * Copyright (c) TIKI Inc.
 * MIT license. See LICENSE file in root directory.
 */

use super::{
    super::{
        utils::{byte_helpers, S3Client},
        Owner,
    },
    SignerModel,
};
use chrono::{DateTime, Utc};
use ring::rsa::KeyPair;
use ring::signature;
use std::error::Error;

#[derive(Debug)]
pub struct SignerService {
    key_pair: KeyPair,
    created: DateTime<Utc>,
    uri: String,
}

#[allow(unused)]
impl SignerService {
    pub async fn create(
        client: &S3Client,
        owner: &Owner,
        key: &str,
    ) -> Result<Self, Box<dyn Error>> {
        let path = Self::path(owner);
        let model = SignerModel::write(client, &path, key).await?;
        Ok(Self::from_model(&path, &model)?)
    }

    pub async fn get(client: &S3Client, owner: &Owner) -> Result<Self, Box<dyn Error>> {
        Self::get_from_path(client, &Self::path(owner)).await
    }

    pub async fn get_from_path(client: &S3Client, path: &str) -> Result<Self, Box<dyn Error>> {
        let model = SignerModel::read(client, path).await?;
        Ok(Self::from_model(path, &model)?)
    }

    pub fn sign(&self, message: &Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut signature = vec![0; self.key_pair.public().modulus_len()];
        match self.key_pair.sign(
            &signature::RSA_PKCS1_SHA256,
            &ring::rand::SystemRandom::new(),
            message.as_slice(),
            &mut signature,
        ) {
            Ok(_) => Ok(signature),
            Err(e) => Err(e.to_string())?,
        }
    }

    pub fn verify(&self, message: &Vec<u8>, signature: &Vec<u8>) -> bool {
        let pub_key = signature::UnparsedPublicKey::new(
            &signature::RSA_PKCS1_2048_8192_SHA256,
            self.key_pair.public(),
        );
        match pub_key.verify(message.as_slice(), signature.as_slice()) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn created(&self) -> DateTime<Utc> {
        self.created
    }
    pub fn uri(&self) -> &str {
        &self.uri
    }
    pub fn key_pair(&self) -> &KeyPair {
        &self.key_pair
    }

    fn from_model(path: &str, model: &SignerModel) -> Result<Self, Box<dyn Error>> {
        let key = byte_helpers::base64_decode(model.key())?;
        match KeyPair::from_der(key.as_slice()) {
            Ok(key_pair) => Ok(Self {
                key_pair,
                created: model.created(),
                uri: path.to_string(),
            }),
            Err(e) => Err(e.to_string())?,
        }
    }

    fn path(owner: &Owner) -> String {
        match owner.provider() {
            Some(provider) => format!("providers/{}/sign.json", provider),
            None => "providers/sign.json".to_string(),
        }
    }
}

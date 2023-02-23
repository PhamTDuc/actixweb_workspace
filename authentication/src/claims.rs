use actix_web::{Error, error::ErrorUnauthorized};
use argonautica::{ Hasher, Verifier};
use chrono::{Utc, Duration};
use jsonwebtoken::{EncodingKey, Header, DecodingKey, Validation};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub username: String,
    pub permissions: Vec<String>,
    pub exp: i64,
}

#[derive(Clone)]
pub struct AuthProvider{
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
    pub header: Header,
    pub validation: Validation,
}

impl AuthProvider {
    pub fn new(jwt_secret: &str)->Self{
        let encoding_key = EncodingKey::from_secret(&jwt_secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(&jwt_secret.as_bytes());
        let header = Header::default();
        let validation = Validation::default();

        Self { encoding_key, decoding_key, header, validation}
    }

    pub fn create_jwt(&self, claims: &Claims)->Result<String, Error>{
        jsonwebtoken::encode(&self.header, claims, &self.encoding_key)
        .map_err(|e| ErrorUnauthorized(e.to_string()))
    }

    pub fn decode_jwt(&self, token: &str) -> Result<Claims, Error>{
        jsonwebtoken::decode::<Claims>(token, &self.decoding_key, &self.validation)
        .map(|data| data.claims)
        .map_err(|e| ErrorUnauthorized(e.to_string()))
    }
}

impl Claims {
    pub fn new(username: String, permissions: Vec<String>, expiration_sec : i64) -> Self {
        Self {
            username,
            permissions,
            exp: (Utc::now() + Duration::seconds(expiration_sec)).timestamp(),
        }
    }

    pub fn hashing_pasword(secret_key: &str, password: &str)->Result<String, argonautica::Error>{
        let mut hasher = Hasher::default();
        hasher.with_password(password);
        return hasher
            .with_password(password)
            .with_secret_key(secret_key)
            .hash();
    }

    pub fn verify_password(secret_key: &str, password: &str, hash: &str) ->Result<bool, argonautica::Error>{
        let mut verifier = Verifier::default();
        return verifier
            .with_hash(hash)
            .with_password(password)
            .with_secret_key(secret_key)
            .verify();
    }

}

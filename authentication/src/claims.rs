use actix_web::{Error, error::ErrorUnauthorized, dev::ServiceRequest};
use actix_web_grants::permissions::AttachPermissions;
use actix_web_httpauth::extractors::bearer::BearerAuth;
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

impl Claims {
    pub fn new(username: String, permissions: Vec<String>, expiration_hour : i64) -> Self {
        Self {
            username,
            permissions,
            exp: (Utc::now() + Duration::hours(expiration_hour)).timestamp(),
        }
    }

    pub fn create_jwt(jwt_secret: &str, claims: Claims) -> Result<String, Error> {
        let encoding_key = EncodingKey::from_secret(&jwt_secret.as_bytes());
        jsonwebtoken::encode(&Header::default(), &claims, &encoding_key)
            .map_err(|e| ErrorUnauthorized(e.to_string()))
    }

    /// Decode a json web token (JWT)
    pub fn decode_jwt(jwt_secret: &str,token: &str) -> Result<Claims, Error> {
        let decoding_key = DecodingKey::from_secret(&jwt_secret.as_bytes());
        jsonwebtoken::decode::<Claims>(token, &decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| ErrorUnauthorized(e.to_string()))
    }

    pub async fn validator(jwt_secret: &str, req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, (Error, ServiceRequest)> {
        // We just get permissions from JWT
        let result = Claims::decode_jwt(jwt_secret, credentials.token());
        match result {
            Ok(claims) => {
                req.attach(claims.permissions);
                Ok(req)
            }
            // required by `actix-web-httpauth` validator signature
            Err(e) => Err((e, req))
        }
    }

    pub fn hashing_pasword(secret_key: &str, password: &str)->Result<String, argonautica::Error>{
        let mut hasher = Hasher::default();
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

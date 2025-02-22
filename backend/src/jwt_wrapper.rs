use std::env;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub iat: usize, // Optional. Issued at (as UTC timestamp)
    pub ip: String,
}

pub enum JwtError {
    SecretError,
    EncodeError,
    DecodeError,
}

pub fn jwt_encode(claims: Claims) -> Result<String, JwtError> {
    let secret: String = env::var("BACKEND_SECRET").map_err(|_| JwtError::SecretError)?;

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .map_err(|_| JwtError::EncodeError)
}

pub fn jwt_decode(jwt: &str) -> Result<TokenData<Claims>, JwtError> {
    let secret: String = env::var("BACKEND_SECRET").map_err(|_| JwtError::SecretError)?;

    decode::<Claims>(
        jwt,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|_| JwtError::DecodeError)
}

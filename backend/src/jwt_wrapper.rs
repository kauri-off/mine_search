use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::BACKEND_SECRET;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize, // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub iat: usize, // Optional. Issued at (as UTC timestamp)
    pub ip: String,
}

pub enum JwtError {
    EncodeError,
    DecodeError,
}

pub async fn jwt_encode(claims: Claims) -> Result<String, JwtError> {
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret((*BACKEND_SECRET.lock().await).as_ref()),
    )
    .map_err(|_| JwtError::EncodeError)
}

pub async fn jwt_decode(jwt: &str) -> Result<TokenData<Claims>, JwtError> {
    decode::<Claims>(
        jwt,
        &DecodingKey::from_secret((*BACKEND_SECRET.lock().await).as_ref()),
        &Validation::default(),
    )
    .map_err(|_| JwtError::DecodeError)
}

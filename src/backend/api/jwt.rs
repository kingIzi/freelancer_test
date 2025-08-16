

#[cfg(feature = "server")]
pub mod Jwt {
    use chrono::{Duration, Utc};
    use dioxus::prelude::Context;
    use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};
    use dotenvy::dotenv;

    #[derive(Serialize, Deserialize)]
    struct  User {
        password: String,
    }


    #[derive(Serialize, Deserialize)]
    struct Claims {
        password: String,
        exp: i64,
    }

    pub fn get_jwt(password: String) -> Result<String, String> {
        dotenv().ok();
        let secret = std::env::var("JWT_PASSCODE")
            .context("JWT_PASSCODE is not found.")
            .map_err(|e| e.to_string())?;
        let token = encode(
            &Header::default(),
            &Claims {
                password,
                exp: (Utc::now() + Duration::minutes(1)).timestamp(),
            },
            &EncodingKey::from_secret(secret.as_str().as_bytes()),
        )
        .map_err(|e| e.to_string());
        token
    }

    pub fn decode_jwt(token: &str) -> Result<User, String> {
        let secret = std::env::var("JWT_PASSCODE")
            .context("JWT_PASSCODE is not found.")
            .map_err(|e| e.to_string())?;
        let token_data = decode::<User>(
            token,
            &DecodingKey::from_secret(secret.as_str().as_bytes()),
            &Validation::default(),
        );
        match token_data {
            Ok(token_data) => Ok(token_data.claims),
            Err(e) => Err(e.to_string()),
        }
    }

}
const CURRENT_USER_KEY:&str = "current_user";
const JWT_TOKEN:&str = "jwt_token";



#[cfg(feature = "server")]
pub mod Api {
    use validator::Validate;
    use axum::{http::StatusCode, response::IntoResponse};
    

    use crate::backend::{api::{api::{CURRENT_USER_KEY, JWT_TOKEN}, jwt}, forms::{Forms::AuthUserForm, Token}, mongo_crud::MongoRepoError, mongo_models::{self, Docs::BaseUser}, users};

    pub async fn register_user(session: tower_sessions::Session, axum::extract::Json(payload): axum::extract::Json<AuthUserForm>) -> Result<axum::response::Response, StatusCode> {
        match payload.validate() {
            Ok(_) => {
                let optional = users::Users::register_user(payload).await;
                match optional {
                    Ok(user) => Ok(axum::Json(user).into_response()),
                    Err(e) => match e {
                        MongoRepoError::PasswordExistsError(_) => Err(StatusCode::ALREADY_REPORTED),
                        _ => Err(StatusCode::INTERNAL_SERVER_ERROR), 
                    }
                }
            },
            Err(err) => Err(StatusCode::BAD_REQUEST) 
        }
    }

    pub async fn login_user(session: tower_sessions::Session, axum::extract::Json(payload): axum::extract::Json<AuthUserForm>) -> Result<axum::Json<BaseUser>, StatusCode> {
        match payload.validate() {
            Ok(_) => {
                let user = users::Users::login_user(payload).await;
                match user {
                    Ok(user) => {
                        let token = jwt::Jwt::get_jwt(user.password.clone())
                        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                        session.insert(CURRENT_USER_KEY, user.clone()).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                        session.insert(JWT_TOKEN, token).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
                        Ok(axum::Json(user))
                    },
                    Err(e) => match e {
                        MongoRepoError::NotFoundError(_) => Err(StatusCode::NOT_FOUND), 
                        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
                    }
                }
            },
            Err(err) => Err(StatusCode::BAD_REQUEST)
        }
    }

    pub async fn is_authenticated(session: tower_sessions::Session) -> Result<axum::Json<Token>, StatusCode> {
        let token = session.get::<String>(JWT_TOKEN).await.unwrap();
        match token {
             Some(value) => Ok(axum::Json(Token::new(value))),
            _ => Err(StatusCode::UNAUTHORIZED)
        }
    }
}
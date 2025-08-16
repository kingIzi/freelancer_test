
#[cfg(feature="server")]
pub mod Users {
    use bson::{doc, oid::ObjectId, to_bson, Document};
    use tower_sessions_mongodb_store::mongodb::{Collection, error::{ErrorKind,WriteError,WriteFailure}};
    use  crate::backend::{forms::Forms::{AuthUserForm, UserRole}, mongo_crud::{MongoRepo, MongoRepoError}, mongo_models::Docs::BaseUser, utils::server_utils::{decrypt, encrypt}, BASE_USERS};
    use chrono::{Utc};


    fn create_user_body(user: &AuthUserForm) -> BaseUser {
        let now = Utc::now();
        BaseUser {
            id: None,
            password: encrypt(&user.get_password()),
            role: user.get_role().unwrap_or(UserRole::Buyer),
            created: now,
            modified: now
        }
    }

    pub async fn get_users_repo() -> MongoRepo<BaseUser> {
        let repo  = MongoRepo::<BaseUser>::new("sample_mflix", "base_users").await.unwrap();
        repo.create_unique_index(doc! {
            "password": 1
        }).await.unwrap();
        repo
    }


    pub async fn register_user(user: AuthUserForm) -> Result<BaseUser,MongoRepoError> {
        let base_user = create_user_body(&user);
        let col = get_users_repo().await;
        match col.create(base_user).await {
            Ok(id) => {
                let inserted = col.get_by_id(id.to_string().as_str()).await.map_err(|e| {
                    MongoRepoError::UnexpectedError(e.to_string())
                })?;
                let inserted = inserted.ok_or(MongoRepoError::NotFoundError("User not found".to_string()))?;
                Ok(inserted)
            },
            Err(e) => match e.kind.as_ref() {
                ErrorKind::Write(WriteFailure::WriteError(write_err)) => {
                    if write_err.code == 11000 {
                        Err(MongoRepoError::PasswordExistsError(user.get_password().into()))
                    }
                    else {
                        Err(MongoRepoError::WriteError(write_err.message.clone()))
                    }
                },
                _ => {
                    Err(MongoRepoError::UnexpectedError(e.to_string()))
                }
            }
        }
    }

    pub async fn login_user(user: AuthUserForm) -> Result<BaseUser, MongoRepoError> {
        let col = get_users_repo().await;
        let filter = doc! { "password": encrypt(&user.get_password()) };
        match col.find_one(filter).await {
            Ok(value) => {
                let user = value.ok_or(MongoRepoError::NotFoundError("User not found".to_string()))?;
                Ok(user)
            },
            Err(e) => Err(MongoRepoError::UnexpectedError(e.to_string())),
        }
    }
}



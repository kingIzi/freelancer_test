

#[cfg(feature = "server")]
pub mod AuthSession {
    use std::{default};

    use dotenvy::dotenv;
    use thiserror::Error;
    use dioxus::prelude::Context;
    use tokio::task::JoinHandle;
    use tower_sessions_mongodb_store::{mongodb::Client, mongodb::options::ClientOptions ,MongoDBStore};
    use tower_sessions::{cookie::time::Duration, Expiry, MemoryStore, Session, SessionManagerLayer};
    use tower_sessions_core::ExpiredDeletion;


    #[derive(Debug, Error)]
    pub enum MongoCreateClientError {
        #[error("Failed to connect to MongoDB: {0}")]
        DatabaseUrlError(String),
    }

    pub struct SessionData{
        pub layer: SessionManagerLayer<MongoDBStore>,
        pub deletion_task: JoinHandle<Result<(),tower_sessions_core::session_store::Error>>
    }

    pub async fn create_mongodb_client() -> Client{
        dotenv().ok();
        let conn_str = std::env::var("DATABASE_URL").context("DATABASE_URL is not found.").map_err(|e| MongoCreateClientError::DatabaseUrlError(e.to_string())).unwrap();
        let options = ClientOptions::parse(conn_str.as_str()).await.unwrap();
        Client::with_options(options).unwrap()
    }

    pub async fn create_app_session() -> SessionData {
        let client = create_mongodb_client().await;
        let session_store = MongoDBStore::new(client, "tower-sessions".to_string());
        let deletion_task = tokio::task::spawn(
            session_store
                .clone()
                .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
        );
        let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)));
        SessionData {
            layer: session_layer.to_owned(),
            deletion_task: deletion_task
        }
    }
}
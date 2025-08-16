use dioxus::prelude::Context;
// mongo_repo.rs
// #[cfg(feature="server")]
// use mongodb::{
//     error::Result,
//     options::{ClientOptions,IndexOptions},
//     Client, Collection,IndexModel
// };

#[cfg(feature = "server")]
use tower_sessions_mongodb_store::mongodb::{Client,error::Result,Collection,IndexModel,options::{ClientOptions,IndexOptions}};

#[cfg(feature="server")]
use bson::{doc, oid::ObjectId, to_bson, Document};

#[cfg(feature="server")]
use dotenvy::dotenv;

use serde::{de::DeserializeOwned, Serialize};

use std::env;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MongoRepoError {
    #[error("Password already exists: {0}")]
    PasswordExistsError(String),

    #[error("Failed to connect to MongoDB: {0}")]
    DatabaseUrlError(String),

    #[error("Failed to perform write operation: {0}")]
    WriteError(String),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error("NOT FOUND")]
    NotFoundError(String),
}

#[cfg(feature="server")]
pub struct MongoRepo<T>
where
    T: Send + Sync + Unpin + Serialize + DeserializeOwned,
{
    col: Collection<T>,
}

#[cfg(feature="server")]
impl<T> MongoRepo<T>
where
    T: Serialize + DeserializeOwned + Unpin + Send + Sync,
{

    // pub async fn create_client() {
    //     dotenv().ok();
    //     let conn_str = env::var("DATABASE_URL").context("DATABASE_URL is not found.").map_err(|e| MongoRepoError::DatabaseUrlError(e.to_string())).unwrap();
    //     let options = ClientOptions::parse(conn_str.as_str()).await.unwrap();
    //     Client::with_options(options).unwrap();
    // }

    pub async fn new(db_name: &str, col_name: &str) -> Result<Self> {
        use crate::backend::auths;
        let client = auths::auth_session::AuthSession::create_mongodb_client().await;
        let col = client.database(db_name).collection::<T>(col_name);
        Ok(Self { col })
    }

    // CREATE
    pub async fn create(&self, item: T) -> Result<ObjectId> {
        let insert_result = self.col.insert_one(item, None).await?;
        Ok(insert_result
            .inserted_id
            .as_object_id()
            .expect("Inserted ID should be ObjectId"))
    }

    // READ by ID
    pub async fn get_by_id(&self, id: &str) -> Result<Option<T>> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        self.col.find_one(doc! { "_id": obj_id }, None).await
    }

    pub async fn find_one(&self, filter: Document) -> Result<Option<T>> {
        self.col.find_one(filter, None).await
    }

    // UPDATE by ID
    pub async fn update_by_id(&self, id: &str, update_doc: Document) -> Result<bool> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let result = self
            .col
            .update_one(doc! { "_id": obj_id }, doc! { "$set": update_doc }, None)
            .await?;
        Ok(result.modified_count > 0)
    }

    // DELETE by ID
    pub async fn delete_by_id(&self, id: &str) -> Result<bool> {
        let obj_id = ObjectId::parse_str(id).unwrap();
        let result = self.col.delete_one(doc! { "_id": obj_id }, None).await?;
        Ok(result.deleted_count > 0)
    }

    pub async fn create_unique_index(&self,indexes: Document) -> Result<()>{
        let index_model = IndexModel::builder().keys(indexes).options(IndexOptions::builder().unique(true).build()).build();
        let result = self.col.create_index(index_model, None).await.unwrap();
        Ok(())
    }
}

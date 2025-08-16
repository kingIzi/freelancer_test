

#[cfg(feature="server")]
pub mod Docs {
    use serde::Serialize;
    use serde::Deserialize;
    use chrono::{DateTime, Utc};
    use bson::oid::ObjectId;
    use crate::backend::forms::Forms::UserRole;

    #[derive(Serialize,Deserialize,Debug,Clone)]
    pub struct BaseUser {
        #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
        pub id: Option<ObjectId>,
        pub password: String,
        pub role: UserRole,
        pub created: DateTime<Utc>,
        pub modified: DateTime<Utc>,
    }
}
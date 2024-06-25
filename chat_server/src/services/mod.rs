mod chat;
mod user;
mod workspace;

pub use chat::*;
use serde::{Deserialize, Serialize};
pub use user::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, sqlx::Type)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

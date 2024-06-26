mod chat;
mod message;
mod user;
mod workspace;
use serde::{Deserialize, Serialize};

pub use chat::*;
pub use message::*;
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

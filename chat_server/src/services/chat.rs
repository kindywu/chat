use std::collections::HashSet;

use crate::{AppError, AppState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use super::ChatType;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: HashSet<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Chat {
    pub id: i64,
    pub ws_id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub members: Vec<i64>,
    pub created_at: DateTime<Utc>,
}

impl AppState {
    pub async fn create_chat(&self, input: &CreateChat, ws_id: i64) -> Result<i64, AppError> {
        let members: Vec<i64> = input.members.iter().cloned().collect();
        let len = members.len();

        if len < 2 {
            return Err(AppError::CreateChatError(
                "the chat members must >=2".to_string(),
            ));
        }

        if len >= 8 && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "the chat members >=8, then the chat name must not empty".to_string(),
            ));
        }

        let userids: Vec<(i64,)> = sqlx::query_as(
            r#"SELECT id
                    FROM users
                    WHERE id = ANY($1)
                    "#,
        )
        .bind(&members)
        .fetch_all(&self.pool)
        .await?;

        let userids: HashSet<i64> = userids.into_iter().map(|f| f.0).collect();

        let diff: HashSet<_> = input.members.difference(&userids).cloned().collect();
        let diff: Vec<i64> = diff.into_iter().collect();

        if !diff.is_empty() {
            return Err(AppError::CreateChatError(format!(
                "the chat members ({diff:?}) is not exist"
            )));
        }

        let chat_type = match (&input.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };

        let (chat_id,): (i64,) = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id"#,
        )
        .bind(ws_id)
        .bind(&input.name)
        .bind(chat_type)
        .bind(&members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat_id)
    }

    pub async fn fetch_chats(&self, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }

    pub async fn is_chat_member(&self, chat_id: i64, user_id: i64) -> Result<bool, AppError> {
        let is_member = sqlx::query(
            r#"
            SELECT 1
            FROM chats
            WHERE id = $1 AND $2 = ANY(members)
            "#,
        )
        .bind(chat_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(is_member.is_some())
    }
}

#[cfg(test)]
impl CreateChat {
    pub(crate) fn new(name: Option<String>, members: HashSet<i64>, public: bool) -> Self {
        Self {
            name,
            members,
            public,
        }
    }
}
#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use anyhow::Result;

    use crate::{services::CreateChat, AppError, AppState};

    #[tokio::test]
    async fn create_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let create_chat = CreateChat::new(None, HashSet::from([1, 2, 3]), true);
        let chat_id = state.create_chat(&create_chat, 1).await?;
        assert!(chat_id.is_positive());
        Ok(())
    }

    #[tokio::test]
    async fn create_chat_should_fail_because_99() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let create_chat = CreateChat::new(None, HashSet::from([1, 2, 99]), true);
        let result = state.create_chat(&create_chat, 1).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, AppError::CreateChatError(_)));
        assert_eq!(
            err.to_string(),
            "create chat error: the chat members ([99]) is not exist".to_string()
        );
        Ok(())
    }
}

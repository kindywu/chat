use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{AppError, AppState};

impl AppState {
    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
        SELECT id, name, owner_id, created_at
        FROM workspaces
        WHERE name = $1
        "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(ws)
    }

    pub async fn create_workspace(&self, name: &str, user_id: i64) -> Result<Workspace, AppError> {
        let sql = r#"
        INSERT INTO workspaces (name, owner_id)
        VALUES ($1, $2)
        RETURNING id, name, owner_id, created_at
        "#;
        let ws = sqlx::query_as(sql)
            .bind(name)
            .bind(user_id)
            .fetch_one(&self.pool)
            .await?;

        Ok(ws)
    }

    pub async fn update_workspace_owner(&self, id: i64, owner_id: i64) -> Result<(), AppError> {
        // update owner_id in two cases 1) owner_id = 0 2) owner's ws_id = id
        let sql = r#"
        UPDATE workspaces
        SET owner_id = $1
        WHERE id = $2 and (SELECT ws_id FROM users WHERE id = $1) = $2
        RETURNING id, name, owner_id, created_at
        "#;
        sqlx::query_as(sql)
            .bind(owner_id)
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, Default)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}

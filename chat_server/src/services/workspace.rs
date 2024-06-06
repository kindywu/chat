use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{AppError, AppState};

impl AppState {
    pub async fn find_workspace_by_name(&self, _name: &str) -> Result<Option<Workspace>, AppError> {
        todo!()
    }

    pub async fn create_workspace(
        &self,
        _name: &str,
        _user_id: i64,
    ) -> Result<Workspace, AppError> {
        todo!()
    }

    pub async fn update_workspace_owner(&self, _id: i64, _user_id: i64) -> Result<(), AppError> {
        todo!()
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, Default)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}

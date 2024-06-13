use serde::{Deserialize, Serialize};

use crate::AppError;
use crate::AppState;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};

use chrono::{DateTime, Utc};
use sqlx::prelude::FromRow;

impl AppState {
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    pub async fn create_user(&self, input: &CreateUser) -> Result<User, AppError> {
        // check if email exists
        let user = self.find_user_by_email(&input.email).await?;
        if user.is_some() {
            return Err(AppError::EmailAlreadyExists(input.email.clone()));
        }

        // check if workspace exists, if not create one
        let ws = match self.find_workspace_by_name(&input.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&input.workspace, 0).await?,
        };

        let password_hash = hash_password(&input.password)?;
        let user: User = sqlx::query_as(
            r#"
            INSERT INTO users (ws_id, email, fullname, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, fullname, email, created_at
            "#,
        )
        .bind(ws.id)
        .bind(&input.email)
        .bind(&input.fullname)
        .bind(password_hash)
        .fetch_one(&self.pool)
        .await?;

        if ws.owner_id == 0 {
            self.update_workspace_owner(ws.id as _, user.id as _)
                .await?;
        }

        Ok(user)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();

    Ok(password_hash)
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct User {
    pub id: i64,
    pub ws_id: i64,
    pub fullname: String,
    pub email: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            ws_id: 0,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: Utc::now(),
        }
    }
}

/// create a user with email and password
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    /// Full name of the user
    pub fullname: String,
    /// Email of the user
    pub email: String,
    /// Workspace name - if not exists, create one
    pub workspace: String,
    /// Password of the user
    pub password: String,
}

#[cfg(test)]
impl CreateUser {
    pub fn new(fullname: &str, email: &str, workspace: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            email: email.to_string(),
            workspace: workspace.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::AppState;
    use anyhow::Result;

    #[tokio::test]
    async fn find_user_by_noexist_email_should_work() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let result = state.find_user_by_email("abc@qq.com").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_email_should_work() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let result = state.find_user_by_email("bob@acme.org").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
        Ok(())
    }
}

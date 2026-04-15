use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::utils::errors::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

impl CreateProjectRequest {
    pub fn validate(self) -> AppResult<Self> {
        let name = self.name.trim().to_owned();
        let description = self.description.map(|value| value.trim().to_owned());

        if name.len() < 3 {
            return Err(AppError::BadRequest(
                "Project name must have at least 3 characters".to_owned(),
            ));
        }

        if name.len() > 80 {
            return Err(AppError::BadRequest(
                "Project name must have at most 80 characters".to_owned(),
            ));
        }

        if let Some(content) = &description {
            if content.len() > 4_000 {
                return Err(AppError::BadRequest(
                    "Project description must have at most 4000 characters".to_owned(),
                ));
            }
        }

        Ok(Self { name, description })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Project {
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
}

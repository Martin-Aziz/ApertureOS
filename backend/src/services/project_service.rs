use std::sync::Arc;

use chrono::Utc;
use serde_json::json;
use uuid::Uuid;

use crate::models::audit::{AuditAction, AuditLogEntry};
use crate::models::auth::AuthenticatedUser;
use crate::models::project::{CreateProjectRequest, Project};
use crate::repositories::project_repository::ProjectRepository;
use crate::utils::errors::{AppError, AppResult};

#[derive(Clone)]
pub struct ProjectService {
    repository: Arc<dyn ProjectRepository>,
}

impl ProjectService {
    pub fn new(repository: Arc<dyn ProjectRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_project(
        &self,
        user: &AuthenticatedUser,
        request: CreateProjectRequest,
    ) -> AppResult<Project> {
        let payload = request.validate()?;
        let now = Utc::now();

        let project = Project {
            id: Uuid::new_v4(),
            owner_id: user.user_id,
            name: payload.name,
            description: payload.description,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        };

        let saved = self.repository.create_project(project).await;

        self.repository
            .append_audit_log(AuditLogEntry {
                id: Uuid::new_v4(),
                actor_id: user.user_id,
                action: AuditAction::ProjectCreated,
                resource_type: "project".to_owned(),
                resource_id: Some(saved.id),
                metadata: json!({ "name": saved.name }),
                created_at: now,
            })
            .await;

        Ok(saved)
    }

    pub async fn list_projects(&self, user: &AuthenticatedUser) -> Vec<Project> {
        self.repository.list_projects_by_owner(user.user_id).await
    }

    pub async fn soft_delete_project(
        &self,
        user: &AuthenticatedUser,
        project_id: Uuid,
    ) -> AppResult<()> {
        let mut project = self
            .repository
            .find_project(project_id)
            .await
            .ok_or_else(|| AppError::NotFound("Project not found".to_owned()))?;

        if project.owner_id != user.user_id {
            return Err(AppError::Forbidden(
                "You are not allowed to delete this project".to_owned(),
            ));
        }

        if project.is_deleted() {
            return Err(AppError::NotFound("Project not found".to_owned()));
        }

        let now = Utc::now();
        project.deleted_at = Some(now);
        project.updated_at = now;

        self.repository.update_project(project).await;

        self.repository
            .append_audit_log(AuditLogEntry {
                id: Uuid::new_v4(),
                actor_id: user.user_id,
                action: AuditAction::ProjectDeleted,
                resource_type: "project".to_owned(),
                resource_id: Some(project_id),
                metadata: json!({ "reason": "user_requested" }),
                created_at: now,
            })
            .await;

        Ok(())
    }

    pub async fn list_audit_logs(&self) -> Vec<AuditLogEntry> {
        self.repository.list_audit_logs().await
    }
}

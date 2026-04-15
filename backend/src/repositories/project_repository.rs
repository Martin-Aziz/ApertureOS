use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::audit::AuditLogEntry;
use crate::models::project::Project;

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn create_project(&self, project: Project) -> Project;
    async fn list_projects_by_owner(&self, owner_id: Uuid) -> Vec<Project>;
    async fn find_project(&self, project_id: Uuid) -> Option<Project>;
    async fn update_project(&self, project: Project);
    async fn append_audit_log(&self, entry: AuditLogEntry);
    async fn list_audit_logs(&self) -> Vec<AuditLogEntry>;
}

#[derive(Default)]
pub struct InMemoryProjectRepository {
    projects: RwLock<HashMap<Uuid, Project>>,
    audit_logs: RwLock<Vec<AuditLogEntry>>,
}

#[async_trait]
impl ProjectRepository for InMemoryProjectRepository {
    async fn create_project(&self, project: Project) -> Project {
        self.projects.write().await.insert(project.id, project.clone());
        project
    }

    async fn list_projects_by_owner(&self, owner_id: Uuid) -> Vec<Project> {
        let projects = self.projects.read().await;
        let mut result = projects
            .values()
            .filter(|project| project.owner_id == owner_id && !project.is_deleted())
            .cloned()
            .collect::<Vec<_>>();

        result.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
        result
    }

    async fn find_project(&self, project_id: Uuid) -> Option<Project> {
        self.projects.read().await.get(&project_id).cloned()
    }

    async fn update_project(&self, project: Project) {
        self.projects.write().await.insert(project.id, project);
    }

    async fn append_audit_log(&self, entry: AuditLogEntry) {
        self.audit_logs.write().await.push(entry);
    }

    async fn list_audit_logs(&self) -> Vec<AuditLogEntry> {
        let mut entries = self.audit_logs.read().await.clone();
        entries.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        entries
    }
}

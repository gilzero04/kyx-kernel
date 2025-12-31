use std::sync::Arc;
use crate::modules::system::domain::user::{UserRepository, UserFilter, PaginatedUsers};
use crate::core::AppError;
use anyhow::Result;
use uuid::Uuid;

// Import separate action modules
mod delete_action;

pub struct UserAdminService {
    repo: Arc<dyn UserRepository>,
}

impl UserAdminService {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn list_users(&self, mut filter: UserFilter, actor_tenant_id: Option<Uuid>) -> Result<PaginatedUsers, AppError> {
        filter.actor_tenant_id = actor_tenant_id;
        self.repo.list(filter).await.map_err(|e| AppError { code: 500, message: e.to_string() })
    }

    pub async fn update_user(&self, id: Uuid, full_name: Option<String>, is_active: Option<bool>, role_slug: Option<String>, mut tenant_id: Option<Uuid>, actor_tenant_id: Option<Uuid>) -> Result<(), AppError> {
        // Enforce isolation: Non-system admins cannot move users to other tenants
        if let Some(tid) = actor_tenant_id {
            tenant_id = Some(tid);
        }
        
        let updated = self.repo.update(id, full_name, is_active, role_slug, tenant_id, actor_tenant_id).await
            .map_err(|e| AppError { code: 500, message: e.to_string() })?;
        
        if !updated {
            return Err(AppError { code: 404, message: "User not found".into() });
        }
        Ok(())
    }

    pub async fn delete_user(&self, target_id: Uuid, current_user_id: Option<Uuid>, actor_tenant_id: Option<Uuid>) -> Result<(), AppError> {
        // Delegate to separate file
        delete_action::delete_user(&self.repo, target_id, current_user_id, actor_tenant_id).await
    }

    pub async fn reset_password(&self, id: Uuid, new_password: String, actor_tenant_id: Option<Uuid>) -> Result<(), AppError> {
        let hashed = crate::core::utils::password::hash_password(&new_password)?;
        let updated = self.repo.update_password(id, hashed, actor_tenant_id).await
            .map_err(|e| AppError { code: 500, message: e.to_string() })?;
        
        if !updated {
            return Err(AppError { code: 404, message: "User not found".into() });
        }
        Ok(())
    }
}

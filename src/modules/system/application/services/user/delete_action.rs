use std::sync::Arc;
use crate::modules::system::domain::user::UserRepository;
use crate::core::AppError;
use uuid::Uuid;

pub async fn delete_user(
    repo: &Arc<dyn UserRepository>,
    target_id: Uuid,
    current_user_id: Option<Uuid>,
    actor_tenant_id: Option<Uuid>
) -> Result<(), AppError> {
    // 1. Check self-deletion
    if let Some(current) = current_user_id {
        if current == target_id {
            return Err(AppError { code: 400, message: "Cannot delete your own account".into() });
        }
    }

    // 2. Check if SuperAdmin
    let is_super = repo.is_superadmin(target_id).await.unwrap_or(false);
    if is_super {
        let count = repo.count_superadmins().await.unwrap_or(0);
        if count <= 1 {
            return Err(AppError { code: 400, message: "Cannot delete the last Super Administrator".into() });
        }
    }

    // 3. Check if last user in any tenant
    let tenants = repo.get_user_tenants(target_id).await.unwrap_or_default();
    for t in tenants {
        if let Some(count) = t.member_count {
            if count <= 1 {
                    return Err(AppError {
                    code: 400,
                    message: format!("Cannot delete the last user of tenant '{}'", t.name).into() 
                });
            }
        }
    }

    // 4. Perform Delete
    let deleted = repo.soft_delete(target_id, actor_tenant_id).await
            .map_err(|e| AppError { code: 500, message: e.to_string() })?;
    
    if !deleted {
            return Err(AppError { code: 404, message: "User not found or already deleted".into() });
    }

    Ok(())
}

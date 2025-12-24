use std::sync::Arc;
use crate::core::infrastructure::redis::Redis;
use crate::modules::auth::domain::login::UserCredentials;
use crate::core::utils::jwt::{JwtService, TokenType};
use crate::core::AppError;
use anyhow::Result;

use crate::core::infrastructure::audit::AuditService;
use crate::core::infrastructure::config_service::ConfigService;
use crate::core::infrastructure::database::Database;
use crate::core::utils::password::verify_password;
use uuid::Uuid;
use sqlx::Row;

use crate::modules::auth::interface::http::dto::auth::{UserInfo, AuthResponse, SetupRequest, CreateUserRequest};

pub struct AuthService {
    db: Arc<Database>,
    _redis: Arc<Redis>,
    jwt: Arc<JwtService>,
    audit: Arc<AuditService>,
    config: Arc<ConfigService>,
}

impl AuthService {
    pub fn new(
        db: Arc<Database>,
        redis: Arc<Redis>, 
        jwt: Arc<JwtService>, 
        audit: Arc<AuditService>, 
        config: Arc<ConfigService>
    ) -> Self {
        Self { db, _redis: redis, jwt, audit, config }
    }

    pub async fn login(&self, creds: UserCredentials) -> Result<AuthResponse, AppError> {
        // 1. Find User by Email
        let user_row = sqlx::query(
            "SELECT id, email, full_name, hashed_password FROM auth_users WHERE email = $1"
        )
        .bind(&creds.username)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Database error: {}", e),
        })?;

        let (user_id, email, full_name, hashed_password) = match user_row {
            Some(row) => (
                row.get::<Uuid, _>("id"),
                row.get::<String, _>("email"),
                row.get::<Option<String>, _>("full_name"),
                row.get::<String, _>("hashed_password")
            ),
            None => {
                let _ = self.audit.log(&creds.username, "LOGIN_FAILURE", None, "FAILURE", None).await;
                return Err(AppError { code: 401, message: "Invalid credentials".to_string() });
            }
        };

        // 2. Verify Password
        if !verify_password(&creds.password, &hashed_password)? {
            let _ = self.audit.log(&creds.username, "LOGIN_FAILURE", None, "FAILURE", None).await;
            return Err(AppError { code: 401, message: "Invalid credentials".to_string() });
        }

        // 3. Find Membership and Role Slug
        let membership_row = sqlx::query(
            r#"
            SELECT m.tenant_id, r.slug as role_slug, r.id as role_id 
            FROM auth_memberships m
            JOIN sys_roles r ON m.role_id = r.id
            WHERE m.user_id = $1 AND m.is_active = TRUE AND r.is_active = TRUE
            LIMIT 1
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Database error while fetching membership: {}", e),
        })?;
 
        let (tenant_id, role, role_id) = match membership_row {
            Some(row) => (
                row.get::<Uuid, _>("tenant_id").to_string(),
                row.get::<String, _>("role_slug"),
                row.get::<Uuid, _>("role_id")
            ),
            None => {
                return Err(AppError { code: 403, message: "No active tenant membership found".to_string() });
            }
        };

        // 4. Fetch Permissions for this Role
        let permission_rows = sqlx::query(
            r#"
            SELECT p.slug 
            FROM sys_permissions p
            JOIN sys_role_permissions rp ON p.id = rp.permission_id
            WHERE rp.role_id = $1 AND p.is_active = TRUE AND p.deleted_at IS NULL
            "#
        )
        .bind(role_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Database error while fetching permissions: {}", e),
        })?;

        let permissions: Vec<String> = permission_rows.iter().map(|r| r.get("slug")).collect();

        let user_id_str = user_id.to_string();

        // 5. Generate Tokens
        let access_expiry = self.config.get_int("access_token_expire_minutes", 30).await;
        let refresh_expiry = self.config.get_int("refresh_token_expire_minutes", 1440).await;
        
        let access_token = self.jwt.generate_token(&user_id_str, &role, &tenant_id, permissions.clone(), TokenType::Access, chrono::Duration::minutes(access_expiry))?;
        let refresh_token = self.jwt.generate_token(&user_id_str, &role, &tenant_id, permissions, TokenType::Refresh, chrono::Duration::minutes(refresh_expiry))?;
        
        // 5. Log and Cache
        self.audit.log(&user_id_str, "LOGIN_SUCCESS", Some(&tenant_id), "SUCCESS", None).await?;
        
        let key = format!("auth:refresh:{}", user_id_str);
        let mut conn = self._redis.get_connection();
        
        let _: () = redis::cmd("SET")
            .arg(&key)
            .arg(&refresh_token)
            .arg("EX")
            .arg(refresh_expiry * 60)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to store refresh token: {}", e),
            })?;

        Ok(AuthResponse { 
            access_token, 
            refresh_token,
            user: UserInfo {
                id: user_id,
                email,
                full_name,
                role: role,
            }
        })
    }

    pub async fn refresh_session(&self, refresh_token: &str) -> Result<AuthResponse, AppError> {
        let claims = self.jwt.verify_token(refresh_token)?;
        
        if claims.token_type != TokenType::Refresh {
            return Err(AppError {
                code: 401,
                message: "Invalid token type".to_string(),
            });
        }
        
        let user_id = &claims.sub;
        let key = format!("auth:refresh:{}", user_id);
        
        let mut conn = self._redis.get_connection();
        
        let stored_token: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to get refresh token: {}", e),
            })?;
            
        if let Some(st) = stored_token {
            if st == refresh_token {
                // Get dynamic expiry
                let access_expiry = self.config.get_int("access_token_expire_minutes", 30).await;
                let refresh_expiry = self.config.get_int("refresh_token_expire_minutes", 1440).await;

                let access_token = self.jwt.generate_token(user_id, &claims.role, &claims.tenant_id, claims.permissions.clone(), TokenType::Access, chrono::Duration::minutes(access_expiry))?;
                let new_refresh_token = self.jwt.generate_token(user_id, &claims.role, &claims.tenant_id, claims.permissions.clone(), TokenType::Refresh, chrono::Duration::minutes(refresh_expiry))?;
                
                // Fetch User Details for Response
                let user_uuid = Uuid::parse_str(user_id).unwrap_or_default();
                let user_row = sqlx::query(
                    "SELECT email, full_name FROM auth_users WHERE id = $1"
                )
                .bind(user_uuid)
                .fetch_optional(&self.db.pool)
                .await
                .map_err(|e| AppError {
                    code: 500,
                    message: format!("Database error: {}", e),
                })?;

                let (email, full_name) = match user_row {
                    Some(row) => (
                        row.get::<String, _>("email"),
                        row.get::<Option<String>, _>("full_name")
                    ),
                    None => ("unknown".to_string(), None)
                };

                // Update Redis
                let _: () = redis::cmd("SET")
                    .arg(&key)
                    .arg(&new_refresh_token)
                    .arg("EX")
                    .arg(refresh_expiry * 60)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| AppError {
                        code: 500,
                        message: format!("Failed to update refresh token: {}", e),
                    })?;
                    
                // Log refresh
                self.audit.log(user_id, "SESSION_REFRESH", Some(&claims.tenant_id), "SUCCESS", None).await?;

                return Ok(AuthResponse { 
                    access_token, 
                    refresh_token: new_refresh_token,
                    user: UserInfo {
                        id: user_uuid,
                        email,
                        full_name,
                        role: claims.role,
                    }
                });
            }
        }
        
        Err(AppError {
            code: 401,
            message: "Refresh token revoked or expired".to_string(),
        })
    }

    pub async fn logout(&self, access_token: &str) -> Result<(), AppError> {
        let claims = self.jwt.verify_token(access_token)?;
        let key = format!("auth:refresh:{}", claims.sub);
        
        let mut conn = self._redis.get_connection();
        
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to delete refresh token: {}", e),
            })?;
            
        // Log logout
        self.audit.log(&claims.sub, "LOGOUT", None, "SUCCESS", None).await?;
            
        Ok(())
    }

    pub async fn is_setup_done(&self) -> Result<bool, AppError> {
        let count: Option<i64> = sqlx::query_scalar("SELECT COUNT(*) FROM auth_users")
            .fetch_one(&self.db.pool)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Database error while checking setup status: {}", e),
            })?;
            
        Ok(count.unwrap_or(0) > 0)
    }

    pub async fn initialize_system(&self, req: SetupRequest) -> Result<AuthResponse, AppError> {
        // 1. Double check if setup is already done
        if self.is_setup_done().await? {
            return Err(AppError {
                code: 403,
                message: "System is already initialized".to_string(),
            });
        }

        use crate::core::utils::password::hash_password;
        use crate::core::utils::password_policy::validate_password;

        // 2. Validate password policy
        validate_password(&req.password)?;

        // 3. Transact: Create Tenant -> Create User -> Create Membership
        let mut tx = self.db.pool.begin().await.map_err(|e| AppError {
            code: 500,
            message: format!("Transaction error: {}", e),
        })?;

        // 4. Create Tenant
        let tenant_slug = format!("org-{}", &req.org_name.to_lowercase().replace(' ', "-"));
        let tenant_row = sqlx::query(
            "INSERT INTO auth_tenants (name, slug) VALUES ($1, $2) RETURNING id"
        )
        .bind(&req.org_name)
        .bind(&tenant_slug)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create tenant: {}", e),
        })?;

        let tenant_id = tenant_row.get::<Uuid, _>("id");

        // Check Role Limit for SuperAdmin before creating user (though it's the first one)
        // We do this check inside transaction or before committed, but check_role_limit uses &self.db which is separate connection
        // Since we are in init, concurrency is low risk. But let's check it strictly.
        // Actually, check_role_limit uses self.db.pool, not tx. 
        // For init, we can skip or just call it. It will return 0 count.
        // But for create_user it is important.

        // 5. Create User
        let hashed_pw = hash_password(&req.password).map_err(|e| e)?;
        let user_row = sqlx::query(
            "INSERT INTO auth_users (email, hashed_password, full_name) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(&req.email)
        .bind(hashed_pw)
        .bind(&req.full_name)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create admin user: {}", e),
        })?;

        let user_id = user_row.get::<Uuid, _>("id");

        // 5. Get Role ID for SuperAdmin
        let role_row = sqlx::query("SELECT id FROM sys_roles WHERE slug = 'superadmin'")
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to find superadmin role: {}", e),
            })?;
        
        let role_id = role_row.get::<Uuid, _>("id");

        // 6. Create Membership
        sqlx::query(
            "INSERT INTO auth_memberships (user_id, tenant_id, role, role_id) VALUES ($1, $2, $3, $4)"
        )
        .bind(user_id)
        .bind(tenant_id)
        .bind("SuperAdmin")
        .bind(role_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create membership: {}", e),
        })?;

        tx.commit().await.map_err(|e| AppError {
            code: 500,
            message: format!("Failed to commit transaction: {}", e),
        })?;

        // 6. Generate Tokens
        let user_id_str = user_id.to_string();
        let tenant_id_str = tenant_id.to_string();
        let access_expiry = self.config.get_int("access_token_expire_minutes", 30).await;
        let refresh_expiry = self.config.get_int("refresh_token_expire_minutes", 1440).await;
        
        let access_token = self.jwt.generate_token(&user_id_str, "superadmin", &tenant_id_str, vec!["system:manage".to_string()], TokenType::Access, chrono::Duration::minutes(access_expiry))?;
        let refresh_token = self.jwt.generate_token(&user_id_str, "superadmin", &tenant_id_str, vec!["system:manage".to_string()], TokenType::Refresh, chrono::Duration::minutes(refresh_expiry))?;

        // 7. Persist Global Branding (PCAL)
        // owner_name is now stored in auth_tenants.name - no need for sys_configs
        if let Some(app_name) = req.app_name {
            if !app_name.is_empty() {
                let _ = self.config.set("branding_app_name", serde_json::Value::String(app_name)).await;
            }
        }

        // 8. Audit & Cache
        self.audit.log(&user_id_str, "SYSTEM_INITIALIZED", Some(&tenant_id_str), "SUCCESS", None).await?;

        Ok(AuthResponse { 
            access_token, 
            refresh_token,
            user: UserInfo {
                id: user_id,
                email: req.email,
                full_name: Some(req.full_name),
                role: "superadmin".to_string(),
            }
        })
    }


    pub async fn create_user(&self, req: CreateUserRequest) -> Result<Uuid, AppError> {
        // 1. Check Role Limit
        self.check_role_limit(req.tenant_id, &req.role_slug).await?;

        use crate::core::utils::password::hash_password;
        use crate::core::utils::password_policy::validate_password;

        validate_password(&req.password)?;

        let mut tx = self.db.pool.begin().await.map_err(|e| AppError {
            code: 500,
            message: format!("Transaction error: {}", e),
        })?;

        let hashed_pw = hash_password(&req.password).map_err(|e| e)?;
        
        // 2. Create User
        let user_row = sqlx::query(
            "INSERT INTO auth_users (email, hashed_password, full_name) VALUES ($1, $2, $3) RETURNING id"
        )
        .bind(&req.email)
        .bind(hashed_pw)
        .bind(&req.full_name)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create user: {}", e),
        })?;

        let user_id = user_row.get::<Uuid, _>("id");

        // 3. Get Role ID
        let role_row = sqlx::query("SELECT id FROM sys_roles WHERE slug = $1")
            .bind(&req.role_slug)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Database error fetching role: {}", e),
            })?
            .ok_or(AppError {
                code: 404,
                message: format!("Role '{}' not found", req.role_slug),
            })?;
        
        let role_id = role_row.get::<Uuid, _>("id");

        // 4. Create Membership
        sqlx::query(
            "INSERT INTO auth_memberships (user_id, tenant_id, role, role_id) VALUES ($1, $2, $3, $4)"
        )
        .bind(user_id)
        .bind(req.tenant_id)
        .bind(&req.role_slug) // Store slug as role string for legacy support
        .bind(role_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to assign role: {}", e),
        })?;

        tx.commit().await.map_err(|e| AppError {
            code: 500,
            message: format!("Commit failed: {}", e),
        })?;

        self.audit.log(&user_id.to_string(), "USER_CREATED", Some(&req.tenant_id.to_string()), "SUCCESS", None).await?;

        Ok(user_id)
    }

    async fn check_role_limit(&self, tenant_id: Uuid, role_slug: &str) -> Result<(), AppError> {
        let role_row = sqlx::query("SELECT id, max_members FROM sys_roles WHERE slug = $1")
            .bind(role_slug)
            .fetch_optional(&self.db.pool)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Database error while fetching role: {}", e),
            })?;

        let (role_id, global_max) = match role_row {
            Some(row) => (
                row.get::<Uuid, _>("id"),
                row.get::<Option<i32>, _>("max_members"),
            ),
            None => return Err(AppError {
                code: 404,
                message: format!("Role '{}' not found", role_slug),
            }),
        };

        let override_row = sqlx::query(
            "SELECT max_members FROM sys_tenant_role_limits WHERE tenant_id = $1 AND role_id = $2"
        )
        .bind(tenant_id)
        .bind(role_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Database error while checking tenant role limit: {}", e),
        })?;

        let override_max: Option<i32> = override_row.map(|r| r.get("max_members"));
        let limit = override_max.or(global_max);

        if let Some(max) = limit {
            let count: i64 = sqlx::query_scalar(
                r#"SELECT COUNT(*) FROM auth_memberships m
                 JOIN auth_users u ON m.user_id = u.id
                 WHERE m.tenant_id = $1 AND m.role_id = $2 AND u.deleted_at IS NULL"#
            )
            .bind(tenant_id)
            .bind(role_id)
            .fetch_one(&self.db.pool)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Database error while counting active members: {}", e),
            })?;

            if count >= max as i64 {
                return Err(AppError {
                    code: 400,
                    message: format!("Role limit reached for '{}'. Max allowed: {}", role_slug, max),
                });
            }
        }

        Ok(())
    }
}

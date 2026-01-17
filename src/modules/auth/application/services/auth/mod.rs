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

use crate::modules::auth::interface::http::dto::auth::{UserInfo, AuthResponse, SetupRequest, CreateUserRequest, SignupRequest, SessionInfo, AdminSessionInfo};
// ApiKeyService removed as it is no longer used in AuthService

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
        config: Arc<ConfigService>,
    ) -> Self {
        Self { db, _redis: redis, jwt, audit, config }
    }

    pub async fn login(&self, creds: UserCredentials, ip: String, ua: String) -> Result<AuthResponse, AppError> {
        // 1. Find User by Email
        let user_row = sqlx::query(
            "SELECT id, email, full_name, avatar_url, cover_url, hashed_password FROM auth_users WHERE email = $1"
        )
        .bind(&creds.username)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Database error: {}", e),
        })?;

        let (user_id, email, full_name, avatar_url, cover_url, hashed_password) = match user_row {
            Some(row) => (
                row.get::<Uuid, _>("id"),
                row.get::<String, _>("email"),
                row.get::<Option<String>, _>("full_name"),
                row.get::<Option<String>, _>("avatar_url"),
                row.get::<Option<String>, _>("cover_url"),
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
            WHERE m.user_id = $1 AND m.is_active = TRUE AND r.is_active = TRUE AND m.deleted_at IS NULL
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
 
        let (tenant_id_uuid, role, role_id) = match membership_row {
            Some(row) => (
                row.get::<Uuid, _>("tenant_id"),
                row.get::<String, _>("role_slug"),
                row.get::<Uuid, _>("role_id")
            ),
            None => {
                return Err(AppError { code: 403, message: "No active tenant membership found".to_string() });
            }
        };

        // Check if this tenant is the platform owner and get its type
        let tenant_row = sqlx::query(
            r#"
            SELECT t.id, tt.slug as type_slug 
            FROM auth_tenants t
            LEFT JOIN sys_tenant_types tt ON t.tenant_type_id = tt.id
            WHERE t.id = $1
            "#
        )
        .bind(tenant_id_uuid)
        .fetch_one(&self.db.pool)
        .await
        .map_err(|e| AppError { code: 500, message: format!("Failed to fetch tenant details: {}", e) })?;
        
        let owner_row = sqlx::query("SELECT id FROM auth_tenants ORDER BY created_at ASC LIMIT 1")
            .fetch_one(&self.db.pool)
            .await
            .map_err(|e| AppError { code: 500, message: format!("Failed to fetch owner: {}", e) })?;
        
        let owner_id = owner_row.get::<Uuid, _>("id");
        let tenant_type = tenant_row.get::<Option<String>, _>("type_slug");
        let is_system_owner = tenant_id_uuid == owner_id;
        let tenant_id = tenant_id_uuid;

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
        let refresh_expiry = self.config.get_int("refresh_token_expire_minutes", 1440).await;

        // 5. Generate Session ID and Tokens
        let sid = Uuid::new_v4().to_string();
        let access_token = self.jwt.generate_access_token(&user_id_str, &role, tenant_id, permissions.clone(), is_system_owner, Some(sid.clone()))?;
        let refresh_token = self.jwt.generate_refresh_token(&user_id_str, &role, tenant_id, permissions.clone(), is_system_owner, Some(sid.clone()))?;
        
        // 5. Log and Cache Session
        self.audit.log(&user_id_str, "LOGIN_SUCCESS", Some(&tenant_id.to_string()), "SUCCESS", None).await?;
        
        let session_key = format!("auth:session:{}:{}", user_id_str, sid);
        let mut conn = self._redis.get_connection();
        
        let session_info = SessionInfo {
            sid: sid.clone(),
            ip,
            user_agent: ua,
            created_at: chrono::Utc::now().timestamp(),
            expires_at: (chrono::Utc::now() + chrono::Duration::minutes(refresh_expiry)).timestamp(),
        };

        let _: () = redis::cmd("SET")
            .arg(&session_key)
            .arg(serde_json::to_string(&session_info).unwrap_or_default())
            .arg("EX")
            .arg(refresh_expiry * 60)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to store session: {}", e),
            })?;

        Ok(AuthResponse { 
            access_token, 
            refresh_token,
            user: UserInfo {
                id: user_id,
                email,
                full_name,
                role: role,
                permissions: permissions.clone(),
                tenant_type,
                avatar_url,
                cover_url,
                images: vec![],
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
        let sid = claims.sid.clone().unwrap_or_default();
        let session_key = format!("auth:session:{}:{}", user_id, sid);
        
        let mut conn = self._redis.get_connection();
        
        let session_json: Option<String> = redis::cmd("GET")
            .arg(&session_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to get session: {}", e),
            })?;
            
        if let Some(json) = session_json {
            // Get dynamic expiry
            let access_expiry = self.config.get_int("access_token_expire_minutes", 30).await;
            let refresh_expiry = self.config.get_int("refresh_token_expire_minutes", 1440).await;

            // Fetch User Details and Tenant Type for Response
            let user_uuid = Uuid::parse_str(user_id).unwrap_or_default();
            let user_row = sqlx::query(
                r#"
                SELECT u.email, u.full_name, u.avatar_url, u.cover_url, tt.slug as tenant_type
                FROM auth_users u
                JOIN auth_memberships m ON u.id = m.user_id
                JOIN auth_tenants t ON m.tenant_id = t.id
                LEFT JOIN sys_tenant_types tt ON t.tenant_type_id = tt.id
                WHERE u.id = $1
                LIMIT 1
                "#
            )
            .bind(user_uuid)
            .fetch_optional(&self.db.pool)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Database error: {}", e),
            })?;

            let (email, full_name, avatar_url, cover_url, tenant_type) = match user_row {
                Some(row) => (
                    row.get::<String, _>("email"),
                    row.get::<Option<String>, _>("full_name"),
                    row.get::<Option<String>, _>("avatar_url"),
                    row.get::<Option<String>, _>("cover_url"),
                    row.get::<Option<String>, _>("tenant_type")
                ),
                None => ("unknown".to_string(), None, None, None, None)
            };

            // Update Session in Redis
            if let Ok(mut session) = serde_json::from_str::<SessionInfo>(&json) {
                session.expires_at = (chrono::Utc::now() + chrono::Duration::minutes(refresh_expiry)).timestamp();
                let _: () = redis::cmd("SET")
                    .arg(&session_key)
                    .arg(serde_json::to_string(&session).unwrap_or_default())
                    .arg("EX")
                    .arg(refresh_expiry * 60)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| AppError {
                        code: 500,
                        message: format!("Failed to update session: {}", e),
                    })?;
            }

            let is_system_owner = claims.is_system_owner.unwrap_or(false);
            let access_token = self.jwt.generate_access_token_dynamic(user_id, &claims.role, claims.tenant_id, claims.permissions.clone(), is_system_owner, access_expiry, claims.sid.clone())?;
            let new_refresh_token = self.jwt.generate_refresh_token_dynamic(user_id, &claims.role, claims.tenant_id, claims.permissions.clone(), is_system_owner, refresh_expiry, claims.sid.clone())?;
            
            // Log refresh
            self.audit.log(user_id, "SESSION_REFRESH", Some(&claims.tenant_id.to_string()), "SUCCESS", None).await?;

            return Ok(AuthResponse { 
                access_token, 
                refresh_token: new_refresh_token,
                user: UserInfo {
                    id: user_uuid,
                    email,
                    full_name,
                    role: claims.role,
                    permissions: claims.permissions,
                    tenant_type,
                    avatar_url,
                    cover_url,
                    images: vec![],
                }
            });
        }
        
        Err(AppError {
            code: 401,
            message: "Session revoked or expired".to_string(),
        })
    }

    pub async fn logout(&self, access_token: &str) -> Result<(), AppError> {
        let claims = self.jwt.verify_token(access_token)?;
        let sid = claims.sid.unwrap_or_default();
        let key = format!("auth:session:{}:{}", claims.sub, sid);
        
        let mut conn = self._redis.get_connection();
        
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to delete session: {}", e),
            })?;
            
        // Log logout
        self.audit.log(&claims.sub, "LOGOUT", Some(&claims.tenant_id.to_string()), "SUCCESS", None).await?;
            
        Ok(())
    }

    pub async fn list_sessions(&self, user_id: &Uuid) -> Result<Vec<SessionInfo>, AppError> {
        let pattern = format!("auth:session:{}:*", user_id);
        let mut conn = self._redis.get_connection();
        
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(&pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to list sessions: {}", e),
            })?;
            
        let mut sessions = Vec::new();
        for key in keys {
            let json: Option<String> = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .unwrap_or(None);
                
            if let Some(j) = json {
                if let Ok(s) = serde_json::from_str::<SessionInfo>(&j) {
                    sessions.push(s);
                }
            }
        }
        
        Ok(sessions)
    }

    pub async fn revoke_session(&self, user_id: &Uuid, sid: &str) -> Result<(), AppError> {
        let key = format!("auth:session:{}:{}", user_id, sid);
        let mut conn = self._redis.get_connection();
        
        let _: () = redis::cmd("DEL")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to revoke session: {}", e),
            })?;
            
        Ok(())
    }

    pub async fn list_all_sessions(&self, current_sid: Option<String>) -> Result<Vec<AdminSessionInfo>, AppError> {
        let pattern = "auth:session:*";
        let mut conn = self._redis.get_connection();
        
        // 1. Get all session keys
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to list all sessions from Redis: {}", e),
            })?;
            
        let mut admin_sessions = Vec::new();

        // 2. Extract User IDs and SIDs from keys
        // Pattern: auth:session:{user_id}:{sid}
        for key in keys {
            let parts: Vec<&str> = key.split(':').collect();
            if parts.len() < 4 { continue; }
            
            let user_id_str = parts[2];
            let user_id = Uuid::parse_str(user_id_str).unwrap_or_default();
            
            // 3. Get session info from Redis
            let json: Option<String> = redis::cmd("GET")
                .arg(&key)
                .query_async(&mut conn)
                .await
                .unwrap_or(None);
                
            if let Some(j) = json {
                if let Ok(s) = serde_json::from_str::<SessionInfo>(&j) {
                    // 4. Enrich with user data from Postgres
                    let user_row = sqlx::query("SELECT email, full_name FROM auth_users WHERE id = $1")
                        .bind(user_id)
                        .fetch_optional(&self.db.pool)
                        .await
                        .unwrap_or(None);
                        
                    let (email, full_name) = match user_row {
                        Some(row) => (row.get::<String, _>("email"), row.get::<Option<String>, _>("full_name")),
                        None => ("unknown".to_string(), None),
                    };
                    
                    let is_current = current_sid.as_ref().map(|id| id == &s.sid).unwrap_or(false);

                    admin_sessions.push(AdminSessionInfo {
                        sid: s.sid,
                        user_id,
                        email,
                        full_name,
                        ip: s.ip,
                        user_agent: s.user_agent,
                        created_at: s.created_at,
                        expires_at: s.expires_at,
                        is_current,
                    });
                }
            }
        }
        
        Ok(admin_sessions)
    }

    pub async fn admin_revoke_session(&self, user_id: Uuid, sid: String) -> Result<(), AppError> {
        self.revoke_session(&user_id, &sid).await
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

    /// Check if a slug is available for a new tenant
    pub async fn check_slug_availability(&self, slug: &str) -> Result<bool, AppError> {
        let exists: Option<bool> = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM auth_tenants WHERE slug = $1)"
        )
        .bind(slug)
        .fetch_one(&self.db.pool)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Database error checking slug: {}", e),
        })?;

        // Returns true if slug is available (doesn't exist)
        Ok(!exists.unwrap_or(false))
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

        // 4. Create Tenant - use provided slug or generate from org_name
        let tenant_slug = req.org_slug
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("org-{}", &req.org_name.to_lowercase().replace(' ', "-")));

        let type_id: Uuid = sqlx::query_scalar("SELECT id FROM sys_tenant_types WHERE slug = 'owner'")
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| AppError { 
                code: 500, 
                message: format!("Default tenant type 'owner' not found: {}", e) 
            })?;

        let tenant_id = Uuid::new_v4();
        
        // STEP 1: Create tenant FIRST (without branding_id, it will be updated later)
        sqlx::query(
            "INSERT INTO auth_tenants (id, parent_id, name, slug, tenant_type_id) 
             VALUES ($1, $1, $2, $3, $4)"
        )
        .bind(tenant_id)
        .bind(&req.org_name)
        .bind(&tenant_slug)
        .bind(type_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create tenant: {}", e),
        })?;

        // STEP 2: Create branding records (tenant now exists, FK will pass)
        // Create TWO branding records for parent: console + workspace context
        let branding_id_console = Uuid::new_v4();
        let branding_id_workspace = Uuid::new_v4();
        let default_primary = "#0ea5e9".to_string(); // Sky blue
        let default_secondary = "#6366f1".to_string(); // Indigo
        let default_accent = "#f43f5e".to_string(); // Rose
        
        // Create Console branding (primary - will be linked to tenant)
        sqlx::query(
            "INSERT INTO sys_brandings (id, tenant_id, context, name, app_name, primary_color, secondary_color, accent_color, created_at, updated_at)
             VALUES ($1, $2, 'console', $3, $4, $5, $6, $7, NOW(), NOW())"
        )
        .bind(branding_id_console)
        .bind(tenant_id)
        .bind(&req.org_name)  // No " Branding" suffix!
        .bind(&req.app_name)
        .bind(req.primary_color.as_ref().unwrap_or(&default_primary))
        .bind(req.secondary_color.as_ref().unwrap_or(&default_secondary))
        .bind(req.accent_color.as_ref().unwrap_or(&default_accent))
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create console branding: {}", e),
        })?;
        
        // Create Workspace branding (inherits from console by sharing same values)
        sqlx::query(
            "INSERT INTO sys_brandings (id, tenant_id, context, name, app_name, primary_color, secondary_color, accent_color, created_at, updated_at)
             VALUES ($1, $2, 'workspace', $3, $4, $5, $6, $7, NOW(), NOW())"
        )
        .bind(branding_id_workspace)
        .bind(tenant_id)
        .bind(&req.org_name)  // Same name, no suffix
        .bind(&req.app_name)
        .bind(req.primary_color.as_ref().unwrap_or(&default_primary))
        .bind(req.secondary_color.as_ref().unwrap_or(&default_secondary))
        .bind(req.accent_color.as_ref().unwrap_or(&default_accent))
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create workspace branding: {}", e),
        })?;

        // STEP 3: Update tenant with branding_id reference
        sqlx::query("UPDATE auth_tenants SET branding_id = $1 WHERE id = $2")
            .bind(branding_id_console)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Failed to update tenant branding: {}", e),
            })?;


        // STEP 4: Create base RBAC roles for this tenant (BEFORE querying)
        sqlx::query(
            "INSERT INTO sys_roles (tenant_id, slug, name, description, sort_order) VALUES 
                ($1, 'superadmin', 'Super Administrator', 'Full access within the organization', 100),
                ($1, 'admin', 'Administrator', 'Administrative access', 80),
                ($1, 'operator', 'Operator', 'Operation access', 60),
                ($1, 'viewer', 'Viewer', 'Read-only access', 40)
             ON CONFLICT (tenant_id, slug) DO NOTHING"
        )
        .bind(tenant_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create roles: {}", e),
        })?;

        // Assign ALL permissions to superadmin role
        sqlx::query(
            "INSERT INTO sys_role_permissions (role_id, permission_id)
             SELECT r.id, p.id FROM sys_roles r, sys_permissions p
             WHERE r.slug = 'superadmin' AND r.tenant_id = $1
             ON CONFLICT DO NOTHING"
        )
        .bind(tenant_id)
        .execute(&mut *tx)
        .await
        .ok(); // Non-critical, ignore errors

        // 5. Create User
        use crate::core::utils::avatar::generate_default_avatar;
        let default_avatar = generate_default_avatar(&req.full_name);
        let hashed_pw = hash_password(&req.password).map_err(|e| e)?;
        let user_row = sqlx::query(
            "INSERT INTO auth_users (email, hashed_password, full_name, avatar_url) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(&req.email)
        .bind(hashed_pw)
        .bind(&req.full_name)
        .bind(default_avatar)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create admin user: {}", e),
        })?;

        let user_id = user_row.get::<Uuid, _>("id");

        // 6. Get Role ID for SuperAdmin (now exists!)
        let role_row = sqlx::query("SELECT id FROM sys_roles WHERE slug = 'superadmin' AND tenant_id = $1")
            .bind(tenant_id)
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

        // 7. Manual API Key Management (Self-Service)
        // DELETED: Automatic API Key generation removed. Users will create keys via dashboard.

        // Seed Default Homepage
        use crate::core::utils::seeding::get_default_home_page_content;
        let home_page_content = get_default_home_page_content(&req.org_name);
        
        sqlx::query(
            "INSERT INTO sys_pages (tenant_id, slug, title, content, is_published) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(tenant_id)
        .bind("home")
        .bind("Home")
        .bind(&home_page_content)
        .bind(true)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create default homepage: {}", e),
        })?;

        // 7. Handover orphaned records (Themes, Pages, Configs seeded during first boot)
        // These are records created with NULL tenant_id before setup was completed.
        
        // Update Pages
        sqlx::query("UPDATE sys_pages SET tenant_id = $1 WHERE tenant_id IS NULL")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError { code: 500, message: format!("Failed to handover pages: {}", e) })?;

        // Update Themes
        sqlx::query("UPDATE sys_themes SET tenant_id = $1 WHERE tenant_id IS NULL")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError { code: 500, message: format!("Failed to handover themes: {}", e) })?;

        // Update Roles - Assign global roles (tenant_id = NULL) to the owner
        // ONLY assign roles that the owner doesn't already have (skip duplicates)
        sqlx::query(r#"
            UPDATE sys_roles SET tenant_id = $1 
            WHERE tenant_id IS NULL 
            AND slug NOT IN (SELECT slug FROM sys_roles WHERE tenant_id = $1)
        "#)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError { code: 500, message: format!("Failed to handover roles: {}", e) })?;

        // Update API Keys
        sqlx::query("UPDATE sys_api_keys SET tenant_id = $1 WHERE tenant_id IS NULL")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError { code: 500, message: format!("Failed to handover api keys: {}", e) })?;

        // Update Configs (Assign to platform scope by default during handover)
        sqlx::query("UPDATE sys_configs SET tenant_id = $1, scope = COALESCE(scope, 'platform') WHERE tenant_id IS NULL")
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError { code: 500, message: format!("Failed to handover configs: {}", e) })?;

        tx.commit().await.map_err(|e| AppError {
            code: 500,
            message: format!("Failed to commit transaction: {}", e),
        })?;

        // 6. Generate Tokens
        let user_id_str = user_id.to_string();
        let tenant_id = tenant_id; // Uuid
        let _access_expiry = self.config.get_int("access_token_expire_minutes", 30).await;
        let _refresh_expiry = self.config.get_int("refresh_token_expire_minutes", 1440).await;
        
        let access_token = self.jwt.generate_access_token(&user_id_str, "superadmin", tenant_id, vec!["system:manage".to_string()], true, None)?;
        let refresh_token = self.jwt.generate_refresh_token(&user_id_str, "superadmin", tenant_id, vec!["system:manage".to_string()], true, None)?;

        // 7. Branding is now stored in sys_brandings (created above)
        // No need to persist in sys_configs anymore

        // 8. Persist Platform Mode
        let _ = self.config.set("platform_type", serde_json::Value::String(req.platform_type)).await;

        // 9. Set Default Themes (Kyx Light / Kyx Dark)
        // We look for themes named "light" and "dark" (seeded by system) or contain "Kyx"
        let light_theme_row = sqlx::query("SELECT id FROM sys_themes WHERE name = 'Kyx Light' OR (name ILIKE '%light%' AND is_shared = TRUE) ORDER BY (name = 'Kyx Light') DESC, created_at ASC LIMIT 1")
            .fetch_optional(&self.db.pool)
            .await
            .map_err(|e| AppError { code: 500, message: format!("Failed to find default light theme: {}", e) })?;

        let dark_theme_row = sqlx::query("SELECT id FROM sys_themes WHERE name = 'Kyx Dark' OR (name ILIKE '%dark%' AND is_shared = TRUE) ORDER BY (name = 'Kyx Dark') DESC, created_at ASC LIMIT 1")
            .fetch_optional(&self.db.pool)
            .await
            .map_err(|e| AppError { code: 500, message: format!("Failed to find default dark theme: {}", e) })?;

        // Update theme columns correctly per context:
        // Console: uses theme_light_id, theme_dark_id ONLY
        // Workspace: uses theme_workspace_*, theme_app_* ONLY (inherits from console if NULL)
        
        if let Some(row) = &light_theme_row {
            let id: Uuid = row.get("id");
            // Console: set theme_light_id only
            sqlx::query("UPDATE sys_brandings SET theme_light_id = $1, updated_at = NOW() WHERE tenant_id = $2 AND context = 'console'")
                .bind(id)
                .bind(tenant_id)
                .execute(&self.db.pool)
                .await
                .ok();
            // Workspace: set theme_workspace_light_id and theme_app_light_id only
            sqlx::query("UPDATE sys_brandings SET theme_workspace_light_id = $1, theme_app_light_id = $1, updated_at = NOW() WHERE tenant_id = $2 AND context = 'workspace'")
                .bind(id)
                .bind(tenant_id)
                .execute(&self.db.pool)
                .await
                .ok();
        }

        if let Some(row) = &dark_theme_row {
            let id: Uuid = row.get("id");
            // Console: set theme_dark_id only
            sqlx::query("UPDATE sys_brandings SET theme_dark_id = $1, updated_at = NOW() WHERE tenant_id = $2 AND context = 'console'")
                .bind(id)
                .bind(tenant_id)
                .execute(&self.db.pool)
                .await
                .ok();
            // Workspace: set theme_workspace_dark_id and theme_app_dark_id only
            sqlx::query("UPDATE sys_brandings SET theme_workspace_dark_id = $1, theme_app_dark_id = $1, updated_at = NOW() WHERE tenant_id = $2 AND context = 'workspace'")
                .bind(id)
                .bind(tenant_id)
                .execute(&self.db.pool)
                .await
                .ok();
        }


        // 10. Audit & Cache
        self.audit.log(&user_id_str, "SYSTEM_INITIALIZED", Some(&tenant_id.to_string()), "SUCCESS", None).await?;

        Ok(AuthResponse { 
            access_token, 
            refresh_token,
            user: UserInfo {
                id: user_id,
                email: req.email,
                full_name: Some(req.full_name),
                role: "superadmin".to_string(),
                permissions: vec!["system:manage".to_string()],
                tenant_type: None,
                avatar_url: None,
                cover_url: None,
                images: vec![],
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

        use crate::core::utils::avatar::generate_default_avatar;
        let default_avatar = generate_default_avatar(&req.full_name);
        let hashed_pw = hash_password(&req.password).map_err(|e| e)?;
        
        // 2. Create User
        let user_row = sqlx::query(
            "INSERT INTO auth_users (email, hashed_password, full_name, avatar_url) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(&req.email)
        .bind(hashed_pw)
        .bind(&req.full_name)
        .bind(default_avatar)
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

    pub async fn signup(&self, req: SignupRequest) -> Result<AuthResponse, AppError> {
        use crate::core::utils::password::hash_password;
        use crate::core::utils::password_policy::validate_password;

        // 1. Validate password policy
        validate_password(&req.password)?;

        // 2. Check if slug is available
        if !self.check_slug_availability(&req.org_slug).await? {
            return Err(AppError {
                code: 400,
                message: format!("Slug '{}' is already taken", req.org_slug),
            });
        }

        // 3. Transact: Create Tenant -> Create User -> Create Membership
        let mut tx = self.db.pool.begin().await.map_err(|e| AppError {
            code: 500,
            message: format!("Transaction error: {}", e),
        })?;

        // 4. Get Tenant Type ID
        let type_row = sqlx::query("SELECT id FROM sys_tenant_types WHERE slug = $1")
            .bind(&req.plan_type)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AppError {
                code: 500,
                message: format!("Database error fetching tenant type: {}", e),
            })?
            .ok_or(AppError {
                code: 400,
                message: format!("Tenant type '{}' not found", req.plan_type),
            })?;
        
        let tenant_type_id = type_row.get::<Uuid, _>("id");

        // 5. Create Tenant
        let tenant_row = sqlx::query(
            "INSERT INTO auth_tenants (name, slug, parent_id, tenant_type_id) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(&req.org_name)
        .bind(&req.org_slug)
        .bind(req.parent_id)
        .bind(tenant_type_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create tenant: {}", e),
        })?;

        let tenant_id = tenant_row.get::<Uuid, _>("id");

        // 6. Create User
        use crate::core::utils::avatar::generate_default_avatar;
        let default_avatar = generate_default_avatar(&req.full_name);
        let hashed_pw = hash_password(&req.password).map_err(|e| e)?;
        let user_row = sqlx::query(
            "INSERT INTO auth_users (email, hashed_password, full_name, avatar_url) VALUES ($1, $2, $3, $4) RETURNING id"
        )
        .bind(&req.email)
        .bind(hashed_pw)
        .bind(&req.full_name)
        .bind(default_avatar)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create admin user: {}", e),
        })?;

        let user_id = user_row.get::<Uuid, _>("id");

        // 7. Find 'superadmin' role (all tenants have one admin)
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

        // Seed Default Homepage
        use crate::core::utils::seeding::get_default_home_page_content;
        let home_page_content = get_default_home_page_content(&req.org_name);

        sqlx::query(
            "INSERT INTO sys_pages (tenant_id, slug, title, content, is_published) VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(tenant_id)
        .bind("home")
        .bind("Home")
        .bind(&home_page_content)
        .bind(true)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError {
            code: 500,
            message: format!("Failed to create default homepage: {}", e),
        })?;

        tx.commit().await.map_err(|e| AppError {
            code: 500,
            message: format!("Failed to commit transaction: {}", e),
        })?;

        // 9. Generate Tokens
        let user_id_str = user_id.to_string();
        
        // Fetch permissions for the role
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

        // For internal use or platform setup, we can use None for sid
        let access_token = self.jwt.generate_access_token(&user_id_str, "superadmin", tenant_id, permissions.clone(), false, None)?;
        let refresh_token = self.jwt.generate_refresh_token(&user_id_str, "superadmin", tenant_id, permissions.clone(), false, None)?;

        // 10. Audit & Cache
        self.audit.log(&user_id_str, "USER_SIGNUP", Some(&tenant_id.to_string()), "SUCCESS", None).await?;

        Ok(AuthResponse { 
            access_token, 
            refresh_token,
            user: UserInfo {
                id: user_id,
                email: req.email,
                full_name: Some(req.full_name),
                role: "superadmin".to_string(),
                permissions,
                tenant_type: Some(req.plan_type),
                avatar_url: None,
                cover_url: None,
                images: vec![],
            }
        })
    }

    // ════════════════════════════════════════════════════════════════════════════
    // PROFILE METHODS
    // ════════════════════════════════════════════════════════════════════════════

    /// Get current user profile
    pub async fn get_profile(&self, user_id: &Uuid) -> Result<crate::modules::auth::interface::http::dto::auth::ProfileResponse, AppError> {
        use crate::modules::auth::interface::http::dto::auth::{ProfileResponse, UserImageInfo};

        // Get user
        let row = sqlx::query(
            "SELECT id, email, full_name, avatar_url, cover_url, created_at FROM auth_users WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(user_id)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(|e| AppError { code: 500, message: format!("Database error: {}", e) })?
        .ok_or(AppError { code: 404, message: "User not found".to_string() })?;

        // Get images
        let image_rows = sqlx::query(
            "SELECT id, url, image_type, is_primary, created_at FROM auth_user_images WHERE user_id = $1 ORDER BY created_at DESC"
        )
        .bind(user_id)
        .fetch_all(&self.db.pool)
        .await
        .map_err(|e| AppError { code: 500, message: format!("Database error: {}", e) })?;

        let images: Vec<UserImageInfo> = image_rows.iter().map(|r| {
            let created: chrono::DateTime<chrono::Utc> = r.get("created_at");
            UserImageInfo {
                id: r.get("id"),
                url: r.get("url"),
                image_type: r.get("image_type"),
                is_primary: r.get("is_primary"),
                created_at: created.to_rfc3339(),
            }
        }).collect();

        let created_at: chrono::DateTime<chrono::Utc> = row.get("created_at");

        Ok(ProfileResponse {
            id: row.get("id"),
            email: row.get("email"),
            full_name: row.get("full_name"),
            avatar_url: row.get("avatar_url"),
            cover_url: row.get("cover_url"),
            images,
            created_at: created_at.to_rfc3339(),
        })
    }

    /// Update user profile (name, cover)
    pub async fn update_profile(&self, user_id: &Uuid, full_name: Option<String>, cover_url: Option<String>) -> Result<(), AppError> {
        let mut updates = Vec::new();
        let mut bind_idx = 1;

        if full_name.is_some() {
            updates.push(format!("full_name = ${}", bind_idx));
            bind_idx += 1;
        }
        if cover_url.is_some() {
            updates.push(format!("cover_url = ${}", bind_idx));
            bind_idx += 1;
        }

        if updates.is_empty() {
            return Ok(()); // Nothing to update
        }

        let query = format!(
            "UPDATE auth_users SET {}, updated_at = NOW() WHERE id = ${} AND deleted_at IS NULL",
            updates.join(", "),
            bind_idx
        );

        let mut q = sqlx::query(&query);

        if let Some(ref name) = full_name {
            q = q.bind(name);
        }
        if let Some(ref url) = cover_url {
            q = q.bind(url);
        }
        q = q.bind(user_id);

        q.execute(&self.db.pool)
            .await
            .map_err(|e| AppError { code: 500, message: format!("Failed to update profile: {}", e) })?;

        self.audit.log(&user_id.to_string(), "PROFILE_UPDATED", None, "SUCCESS", None).await?;

        Ok(())
    }

    /// Update user avatar
    pub async fn update_avatar(&self, user_id: &Uuid, avatar_url: &str) -> Result<String, AppError> {
        // Update avatar_url in auth_users
        sqlx::query("UPDATE auth_users SET avatar_url = $1, updated_at = NOW() WHERE id = $2 AND deleted_at IS NULL")
            .bind(avatar_url)
            .bind(user_id)
            .execute(&self.db.pool)
            .await
            .map_err(|e| AppError { code: 500, message: format!("Failed to update avatar: {}", e) })?;

        // Also add to auth_user_images for history
        let image_id: Uuid = sqlx::query_scalar(
            "INSERT INTO auth_user_images (user_id, url, image_type, is_primary) VALUES ($1, $2, 'avatar', TRUE) RETURNING id"
        )
        .bind(user_id)
        .bind(avatar_url)
        .fetch_one(&self.db.pool)
        .await
        .map_err(|e| AppError { code: 500, message: format!("Failed to save image record: {}", e) })?;

        // Set all other avatars as non-primary
        sqlx::query("UPDATE auth_user_images SET is_primary = FALSE WHERE user_id = $1 AND image_type = 'avatar' AND id != $2")
            .bind(user_id)
            .bind(image_id)
            .execute(&self.db.pool)
            .await
            .ok();

        self.audit.log(&user_id.to_string(), "AVATAR_UPDATED", None, "SUCCESS", None).await?;

        Ok(avatar_url.to_string())
    }
}

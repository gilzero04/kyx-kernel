use std::sync::Arc;
use crate::core::infrastructure::database::Database;
use crate::modules::system::domain::cms::entity::PageEntry;
use anyhow::Result;
use uuid::Uuid;

pub struct PostgresCmsRepository {
    db: Arc<Database>,
}

impl PostgresCmsRepository {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn get_page_by_slug(&self, tenant_id: Uuid, slug: &str) -> Result<Option<PageEntry>> {
        // Normalize slug - try both with and without leading slash
        let normalized_slug = slug.trim_start_matches('/').trim_end_matches('/');
        let with_slash = format!("/{}", normalized_slug);
        
        let row = sqlx::query_as!(
            PageEntry,
            r#"
            SELECT id, tenant_id, slug, title, content, is_published as "is_published!", created_at as "created_at!", updated_at as "updated_at!"
            FROM sys_pages
            WHERE tenant_id = $1 AND (slug = $2 OR slug = $3) AND deleted_at IS NULL AND is_published = true
            "#,
            tenant_id,
            normalized_slug,
            with_slash
        )
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(row)
    }

    /// Get a published page by slug only (for public viewing, ignores tenant_id)
    #[allow(dead_code)]
    pub async fn get_published_page_by_slug(&self, slug: &str) -> Result<Option<PageEntry>> {
        // Normalize slug - try both with and without leading slash
        let normalized_slug = slug.trim_start_matches('/').trim_end_matches('/');
        let with_slash = format!("/{}", normalized_slug);
        
        let row = sqlx::query_as!(
            PageEntry,
            r#"
            SELECT id, tenant_id, slug, title, content, is_published as "is_published!", created_at as "created_at!", updated_at as "updated_at!"
            FROM sys_pages
            WHERE (slug = $1 OR slug = $2) AND deleted_at IS NULL AND is_published = true
            LIMIT 1
            "#,
            normalized_slug,
            with_slash
        )
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(row)
    }

    pub async fn get_page_by_id(&self, tenant_id: Uuid, id: Uuid) -> Result<Option<PageEntry>> {
        let row = sqlx::query_as!(
            PageEntry,
            r#"
            SELECT id, tenant_id, slug, title, content, is_published as "is_published!", created_at as "created_at!", updated_at as "updated_at!"
            FROM sys_pages
            WHERE tenant_id = $1 AND id = $2 AND deleted_at IS NULL
            "#,
            tenant_id,
            id
        )
        .fetch_optional(&self.db.pool)
        .await?;

        Ok(row)
    }

    pub async fn list_pages(&self, tenant_id: Uuid) -> Result<Vec<PageEntry>> {
        let rows = sqlx::query_as!(
            PageEntry,
            r#"
            SELECT id, tenant_id, slug, title, content, is_published as "is_published!", created_at as "created_at!", updated_at as "updated_at!"
            FROM sys_pages
            WHERE tenant_id = $1 AND deleted_at IS NULL
            ORDER BY created_at DESC
            "#,
            tenant_id
        )
        .fetch_all(&self.db.pool)
        .await?;

        Ok(rows)
    }

    pub async fn delete_page(&self, tenant_id: Uuid, id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE sys_pages
            SET deleted_at = NOW()
            WHERE tenant_id = $1 AND id = $2
            "#,
            tenant_id,
            id
        )
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }

    pub async fn save_page(&self, page: PageEntry) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO sys_pages (id, tenant_id, slug, title, content, is_published)
            VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (id) DO UPDATE
            SET slug = EXCLUDED.slug, title = EXCLUDED.title, content = EXCLUDED.content, is_published = EXCLUDED.is_published, updated_at = NOW()
            "#,
            page.id,
            page.tenant_id,
            page.slug,
            page.title,
            page.content,
            page.is_published
        )
        .execute(&self.db.pool)
        .await?;

        Ok(())
    }
}

use std::sync::Arc;
use crate::core::infrastructure::cors::CorsManager;
use crate::core::AppError;
use crate::modules::system::domain::cors::{CorsOrigin, CorsRepository};

pub struct CORSService {
    repo: Arc<dyn CorsRepository>,
    manager: Arc<CorsManager>,
}

impl CORSService {
    pub fn new(repo: Arc<dyn CorsRepository>, manager: Arc<CorsManager>) -> Self {
        Self { repo, manager }
    }

    pub async fn list_origins(&self) -> Result<Vec<CorsOrigin>, AppError> {
        self.repo.list().await.map_err(|e| AppError { 
            code: 500, 
            message: e.to_string() 
        })
    }

    pub async fn add_origin(&self, origin: &str, description: Option<String>) -> Result<CorsOrigin, AppError> {
        let result = self.repo.add(origin, description).await
            .map_err(|e| AppError { code: 500, message: e.to_string() })?;
        
        self.manager.refresh().await?;
        Ok(result)
    }

    pub async fn update_origin(&self, id: i32, is_active: Option<bool>, description: Option<String>) -> Result<CorsOrigin, AppError> {
        let result = self.repo.update(id, is_active, description).await
            .map_err(|e| AppError { code: 500, message: e.to_string() })?;
        
        self.manager.refresh().await?;
        Ok(result)
    }

    pub async fn delete_origin(&self, id: i32) -> Result<(), AppError> {
        self.repo.delete(id).await
            .map_err(|e| AppError { code: 500, message: e.to_string() })?;

        self.manager.refresh().await?;
        Ok(())
    }
}

use std::sync::Arc;

use evobase_core::{AuthService, DatabaseManager, MessagingService, StorageAdapter};

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<dyn AuthService>,
    pub messaging_service: Arc<dyn MessagingService>,
    pub storage: Arc<dyn StorageAdapter>,
    pub database_manager: Arc<DatabaseManager>,
    pub admin_token: String,
}

impl AppState {
    pub fn new(
        auth_service: Arc<dyn AuthService>,
        messaging_service: Arc<dyn MessagingService>,
        storage: Arc<dyn StorageAdapter>,
        database_manager: Arc<DatabaseManager>,
        admin_token: String,
    ) -> Self {
        Self {
            auth_service,
            messaging_service,
            storage,
            database_manager,
            admin_token,
        }
    }
}

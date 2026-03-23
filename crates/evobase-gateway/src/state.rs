use std::sync::Arc;

use evobase_core::{AuthService, MessagingService, StorageAdapter};

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<dyn AuthService>,
    pub messaging_service: Arc<dyn MessagingService>,
    pub storage: Arc<dyn StorageAdapter>,
    pub admin_token: String,
}

impl AppState {
    pub fn new(
        auth_service: Arc<dyn AuthService>,
        messaging_service: Arc<dyn MessagingService>,
        storage: Arc<dyn StorageAdapter>,
        admin_token: String,
    ) -> Self {
        Self {
            auth_service,
            messaging_service,
            storage,
            admin_token,
        }
    }
}

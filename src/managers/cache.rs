use uuid::Uuid;

#[derive(Clone)]
pub struct CacheManager {
    pub current_preset_id: Option<Uuid>,
}

impl CacheManager {
    pub fn new() -> Self {
        Self {
            current_preset_id: None,
        }
    }
}

use crate::managers::database::constants::{DATABASE_NAME, PRESETS_COLLECTION_NAME};
use crate::managers::database::error::Error;
use crate::managers::database::models::Preset;
use crate::settings::Settings;
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::{Client, Collection, Database};
use uuid::Uuid;

mod constants;
mod error;
pub mod models;

#[derive(Clone)]
pub struct DatabaseManager {
    database: Database,
}

impl DatabaseManager {
    pub async fn new(settings: &Settings) -> Result<Self, Error> {
        let client = Client::with_uri_str(settings.database_connection_string.clone()).await?;
        tracing::info!("MongoDB client initialised");
        Ok(Self {
            database: client.database(DATABASE_NAME),
        })
    }

    pub async fn get_default_preset_id(&self) -> Option<Uuid> {
        let presets: Collection<Preset> = self.database.collection(PRESETS_COLLECTION_NAME);
        presets
            .find_one(doc! { "isDefault": true }, None)
            .await
            .unwrap_or_default()
            .map(|preset| preset.id)
    }

    pub async fn set_default_preset_id(&self, preset_id: Uuid) -> Result<(), Error> {
        let presets: Collection<Preset> = self.database.collection(PRESETS_COLLECTION_NAME);
        presets
            .update_many(doc! {}, doc! { "$set": {"isDefault": false} }, None)
            .await?;
        presets
            .update_one(
                doc! {"id": preset_id.to_string()},
                doc! {"$set": {"isDefault": true}},
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn get_presets_list(&self) -> Vec<Preset> {
        let presets: Collection<Preset> = self.database.collection(PRESETS_COLLECTION_NAME);
        let cursor = match presets.find(doc! {}, None).await {
            Ok(cursor) => cursor,
            Err(_) => return vec![],
        };
        cursor.try_collect().await.unwrap_or_default()
    }

    pub async fn get_preset(&self, uuid: Uuid) -> Option<Preset> {
        let presets: Collection<Preset> = self.database.collection(PRESETS_COLLECTION_NAME);
        presets
            .find_one(doc! { "id": uuid.to_string() }, None)
            .await
            .unwrap_or_default()
    }

    pub async fn delete_preset(&self, uuid: Uuid) {
        let presets: Collection<Preset> = self.database.collection(PRESETS_COLLECTION_NAME);
        let _ = presets
            .delete_many(doc! { "id": uuid.to_string() }, None)
            .await;
    }

    pub async fn save_preset(&self, preset: Preset) {
        let presets: Collection<Preset> = self.database.collection(PRESETS_COLLECTION_NAME);
        let _ = presets.insert_one(preset, None).await;
    }
}

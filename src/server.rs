use crate::settings::Settings;
use axum::{serve, Router};
use std::io;
use tokio::net::TcpListener;

pub struct Server {
    connection_string: String,
}

impl Server {
    pub fn new(settings: &Settings) -> Self {
        Self {
            connection_string: settings.connection_string(),
        }
    }

    pub async fn start(&self, app: &Router) -> Result<(), io::Error> {
        let listener = TcpListener::bind(self.connection_string.clone()).await?;
        tracing::info!("server listening on {}", self.connection_string);
        serve(listener, app.clone()).await
    }
}

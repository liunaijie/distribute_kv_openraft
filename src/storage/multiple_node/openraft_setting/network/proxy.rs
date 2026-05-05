use reqwest::Client;
use std::{sync::OnceLock, time::Duration};

use crate::storage::entity::DataEntity;

pub struct RequestProxy {
    http_client: Client,
}

static STORAGE: OnceLock<RequestProxy> = OnceLock::new();

impl RequestProxy {
    pub fn get_proxy() -> &'static RequestProxy {
        STORAGE.get_or_init(|| {
            let client = Client::builder()
                .timeout(Duration::from_secs(5))
                .pool_idle_timeout(Duration::from_secs(90))
                .build()
                .expect("Failed to build reqwest client");
            RequestProxy {
                http_client: client,
            }
        })
    }

    pub async fn forward_to_remote_leader(
        &self,
        leader_addr: &String,
        leader_port: &u32,
        entity: DataEntity,
    ) -> Result<(), anyhow::Error> {
        let url = format!(
            "http://{}:{}/api/v1/set?key={}&value={}",
            leader_addr, leader_port, entity.key, entity.value
        );

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Forwarding network error: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Leader at {} returned error", leader_addr))
        }
    }
}

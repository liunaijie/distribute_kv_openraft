use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

// the stored data structure
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataEntity {
    pub key: String,
    pub value: String,
    pub update_time: Option<u128>,
}

impl DataEntity {
    pub fn new(key: String, value: String) -> DataEntity {
        DataEntity {
            key,
            value,
            update_time: None,
        }
    }

    pub fn from_entity(entity: DataEntity) -> DataEntity {
        DataEntity {
            update_time: Some(Self::now_millis_second()),
            ..entity
        }
    }

    fn now_millis_second() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, anyhow::Error> {
        let entity = serde_json::from_slice(bytes)?;
        Ok(entity)
    }

    pub fn as_bytes(&self) -> Result<Vec<u8>, anyhow::Error> {
        let bytes = serde_json::to_vec(self)?;
        Ok(bytes)
    }
}

impl Display for DataEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let time = self.update_time.unwrap_or(0);
        write!(
            f,
            "key={},value={},update_time={}",
            self.key, self.value, time
        )
    }
}

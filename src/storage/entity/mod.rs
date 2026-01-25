use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

// the stored data structure
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataEntity {
    pub value: String,
    pub create_time: u128,
}

impl DataEntity {
    pub fn new(value: String) -> DataEntity {
        DataEntity {
            value,
            create_time: Self::now_millis_second(),
        }
    }

    fn now_millis_second() -> u128 {
        return SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
    }
}

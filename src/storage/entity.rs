use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

// the stored data structure
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct DataEntity {
    pub key: String,
    pub value: String,
    pub update_time: u128,
}

impl DataEntity {
    pub fn new(key: String, value: String) -> DataEntity {
        DataEntity {
            key,
            value,
            update_time: Self::now_millis_second(),
        }
    }

    fn now_millis_second() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }
}

impl Display for DataEntity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "key={},value={},update_time={}",
            self.key, self.value, self.update_time
        )
    }
}

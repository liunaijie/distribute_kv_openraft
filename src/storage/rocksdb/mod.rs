use anyhow::Error;
use async_trait::async_trait;
use rocksdb::{DB, DBCompactionStyle, Options, SliceTransform};

use crate::storage::{StorageService, entity::DataEntity};

#[derive(Debug)]
pub(crate) struct RocksdbStorage {
    db: DB,
}

impl RocksdbStorage {
    pub fn new(path: String) -> Result<RocksdbStorage, anyhow::Error> {
        let opts = Self::build_rocksdb_options();
        let db = DB::open(&opts, path)
            .map_err(|e| anyhow::Error::msg(format!("Failed to open DB: {}", e)))?;
        Ok(RocksdbStorage { db })
    }

    fn build_rocksdb_options() -> Options {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(1000);
        opts.set_use_fsync(false);
        opts.set_bytes_per_sync(8388608);
        opts.optimize_for_point_lookup(1024);
        opts.set_table_cache_num_shard_bits(6);
        opts.set_max_write_buffer_number(32);
        opts.set_write_buffer_size(536870912);
        opts.set_target_file_size_base(1073741824);
        opts.set_min_write_buffer_number_to_merge(4);
        opts.set_level_zero_stop_writes_trigger(2000);
        opts.set_level_zero_slowdown_writes_trigger(0);
        opts.set_compaction_style(DBCompactionStyle::Universal);
        opts.set_disable_auto_compactions(true);
        let transform = SliceTransform::create_fixed_prefix(10);
        opts.set_prefix_extractor(transform);
        opts.set_memtable_prefix_bloom_ratio(0.2);
        opts
    }
}

#[async_trait]
impl StorageService for RocksdbStorage {
    async fn set(&self, entity: DataEntity) -> Result<(), Error> {
        let bytes = serde_json::to_vec(&entity)
            .map_err(|e| anyhow::Error::msg(format!("Failed to serialize entity: {}", e)))?;

        self.db
            .put(&entity.key, bytes)
            .map_err(|e| anyhow::Error::msg(format!("Failed to set: {}", e)))?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<DataEntity>, anyhow::Error> {
        let value = self
            .db
            .get(key)
            .map_err(|e| anyhow::Error::msg(format!("Failed to get: {}", e)))?;
        let entity = match value {
            Some(bytes) => {
                let entity = serde_json::from_slice::<DataEntity>(&bytes).map_err(|e| {
                    anyhow::Error::msg(format!("Failed to deserialize entity: {}", e))
                })?;
                Some(entity)
            }
            None => None,
        };
        Ok(entity)
    }

    async fn delete(&self, key: &str) -> Result<(), anyhow::Error> {
        self.db
            .delete(key)
            .map_err(|e| anyhow::Error::msg(format!("Failed to delete: {}", e)))?;
        Ok(())
    }
}

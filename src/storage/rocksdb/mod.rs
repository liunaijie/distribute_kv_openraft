use rocksdb::{DB, DBCompactionStyle, Options, SliceTransform};

use crate::storage::StorageService;

#[derive(Debug)]
pub struct RocksdbStorage {
    db: DB,
}

impl RocksdbStorage {
    pub fn new(path: &str) -> Result<RocksdbStorage, anyhow::Error> {
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
        return opts;
    }
}

impl StorageService<String> for RocksdbStorage {
    fn set(&self, key: &str, value: &str) -> Result<(), anyhow::Error> {
        self.db
            .put(key, value)
            .map_err(|e| anyhow::Error::msg(format!("Failed to set: {}", e)))?;
        Ok(())
    }

    fn get(&self, key: &str) -> Result<String, anyhow::Error> {
        let value = self
            .db
            .get(key)
            .map_err(|e| anyhow::Error::msg(format!("Failed to get: {}", e)))?;
        match value {
            Some(value) => Ok(String::from_utf8(value)
                .map_err(|e| anyhow::Error::msg(format!("Failed to get: {}", e)))?),
            None => Ok("".to_string()),
        }
    }

    fn delete(&self, key: &str) -> Result<(), anyhow::Error> {
        self.db
            .delete(key)
            .map_err(|e| anyhow::Error::msg(format!("Failed to delete: {}", e)))?;
        Ok(())
    }
}

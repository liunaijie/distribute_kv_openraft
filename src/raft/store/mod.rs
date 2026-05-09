use std::path::PathBuf;
use std::sync::Arc;

use crate::raft::store::log_store::LogStore;
use crate::raft::store::rocksdb::RocksDBEngine;
use crate::raft::store::state_machine::StateMachineStore;

mod log_store;
pub(super) mod rocksdb;
pub(super) mod state_machine;

pub(crate) async fn new_storage(
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    snapshot_dir: PathBuf,
) -> (LogStore, StateMachineStore) {
    let log_store = LogStore::new(rocksdb_engine_handler.db.clone());
    let sm_store = StateMachineStore::new(rocksdb_engine_handler.db.clone(), snapshot_dir)
        .await
        .unwrap();
    (log_store, sm_store)
}

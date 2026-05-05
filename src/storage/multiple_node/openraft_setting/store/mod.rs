use std::path::PathBuf;
use std::sync::Arc;

use crate::storage::multiple_node::openraft_setting::store::log_store::LogStore;
use crate::storage::multiple_node::openraft_setting::store::state_machine::StateMachineStore;
use rocksdb::DB;

mod log_store;
pub mod state_machine;

pub(crate) async fn new_storage(
    log_store_db: Arc<DB>,
    state_machine_db: Arc<DB>,
    snapshot_dir: PathBuf,
) -> (LogStore, StateMachineStore) {
    let log_store = LogStore::new(log_store_db);
    let sm_store = StateMachineStore::new(state_machine_db, snapshot_dir)
        .await
        .unwrap();
    (log_store, sm_store)
}

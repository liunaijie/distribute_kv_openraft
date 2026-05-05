use distribute_kv_openraft::storage::entity::DataEntity;
use distribute_kv_openraft::storage::{MemoryStorage, StorageService};

#[tokio::test]
async fn storage_get_delete_accept_str() {
    let storage = MemoryStorage::new().unwrap();

    storage
        .set(DataEntity::new("k1".to_string(), "v1".to_string()))
        .await
        .unwrap();

    let got = storage.get("k1").await.unwrap();
    assert!(got.is_some());

    storage.delete("k1").await.unwrap();
    let got2 = storage.get("k1").await.unwrap();
    assert!(got2.is_none());
}

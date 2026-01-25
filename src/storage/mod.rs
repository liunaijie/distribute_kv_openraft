use anyhow::Error;

mod entity;
pub mod rocksdb;

pub trait StorageService<T> {
    fn set(&self, key: &str, value: &str) -> Result<(), Error>;
    fn get(&self, key: &str) -> Result<T, Error>;
    fn delete(&self, key: &str) -> Result<(), Error>;
}

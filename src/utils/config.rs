use dotenv::dotenv;
use serde::Serialize;
use std::env;

#[derive(Debug, Serialize)]
pub struct SystemConfig {
    pub id: u16,
    pub port: u32,
    pub storage_path: String,
}

pub fn load_config() -> SystemConfig {
    // 加载.env文件
    if let Err(e) = dotenv() {
        tracing::error!("无法加载.env文件: {}", e);
        panic!("无法加载.env文件");
    }

    // 从环境变量加载配置
    let config = SystemConfig {
        id: env::var("id")
            .unwrap_or_else(|_| "1".to_string())
            .parse()
            .expect("id必须为唯一的数字"),
        port: env::var("port")
            .unwrap_or_else(|_| "9901".to_string())
            .parse()
            .expect("PORT必须是有效的数字"),
        storage_path: env::var("storage_path")
            .unwrap_or_else(|_| "rocksdb".to_string())
            .parse()
            .expect("必须设置文件存储路径"),
    };
    tracing::info!("startup with config : {:?}", config);
    config
}

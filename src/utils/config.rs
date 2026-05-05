use std::fmt::Display;

use clap::{Parser, command};

pub struct AppConfig {
    pub storage_type: StorageType,
    pub port: u32,
}

impl AppConfig {
    pub fn is_multiple_node(&self) -> bool {
        matches!(&self.storage_type, StorageType::MultipleNode(..))
    }
}

#[derive(Debug, Clone)]
pub enum StorageType {
    Memory,
    RocksDB(String),
    MultipleNode(u16, String, Vec<String>),
}

impl Display for StorageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageType::Memory => write!(f, "Memory"),

            StorageType::RocksDB(path) => {
                write!(f, "RocksDB(path: {})", path)
            }

            StorageType::MultipleNode(id, path, members) => {
                let members_str = members.join(", ");
                write!(
                    f,
                    "MultipleNode(id: {}, storage_path: {}, members: [{}])",
                    id, path, members_str
                )
            }
        }
    }
}

// ======================
// 命令行参数定义（核心）
// ======================
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// 存储类型: memory | rocksdb | multiple_node
    #[arg(short, long, required = false, default_value = "memory")]
    pub r#type: String,

    /// 服务端口 (必填)
    #[arg(short, long, required = false, default_value = "9901")]
    pub port: u32,

    /// 节点ID (仅 multiple_node 需要)
    #[arg(long)]
    pub node_id: Option<u16>,

    /// 存储路径 (rocksdb / multiple_node 需要)
    #[arg(long)]
    pub storage_path: Option<String>,

    /// 集群成员列表，逗号分隔 (仅 multiple_node 需要，例如：127.0.0.1:8080,127.0.0.1:8081)
    #[arg(long)]
    pub member_list: Option<String>,
}

impl CliArgs {
    /// 验证参数并构建 AppConfig
    pub fn validate(self) -> anyhow::Result<AppConfig> {
        let storage_type = match self.r#type.as_str() {
            "memory" => StorageType::Memory,

            "rocksdb" => {
                let storage_path = self
                    .storage_path
                    .ok_or_else(|| anyhow::anyhow!("rocksdb 模式必须传入 --storage-path"))?;
                StorageType::RocksDB(storage_path)
            }

            "multiple_node" => {
                let node_id = self
                    .node_id
                    .ok_or_else(|| anyhow::anyhow!("multiple_node 模式必须传入 --id"))?;
                let storage_path = self
                    .storage_path
                    .ok_or_else(|| anyhow::anyhow!("multiple_node 模式必须传入 --storage-path"))?;
                let member_list_str = self
                    .member_list
                    .ok_or_else(|| anyhow::anyhow!("multiple_node 模式必须传入 --member-list"))?;

                // ======================
                // 严格校验格式：节点ID@IP:端口
                // 示例：1@127.0.0.1:8080,2@127.0.0.1:8081
                // ======================
                let member_list = member_list_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .map(|member| {
                        // 1. 必须包含 @ 符号
                        let (id_part, addr_part) = member.split_once('@').ok_or_else(|| {
                            anyhow::anyhow!(
                                "集群成员格式错误：{}，必须是 节点ID@IP:端口 格式",
                                member
                            )
                        })?;

                        // 2. 节点ID必须是纯数字（u16）
                        let _node_id: u16 = id_part.parse().map_err(|_| {
                            anyhow::anyhow!("节点ID格式错误：{}，必须是数字（0-65535）", id_part)
                        })?;

                        // 3. 地址部分必须包含 : （IP:端口）
                        if !addr_part.contains(':') {
                            return Err(anyhow::anyhow!(
                                "地址格式错误：{}，必须是 IP:端口 格式",
                                addr_part
                            ));
                        }

                        Ok(member)
                    })
                    .collect::<anyhow::Result<Vec<String>>>()?;

                StorageType::MultipleNode(node_id, storage_path, member_list)
            }

            other => anyhow::bail!("不支持的存储类型: {}", other),
        };

        Ok(AppConfig {
            storage_type,
            port: self.port,
        })
    }
}

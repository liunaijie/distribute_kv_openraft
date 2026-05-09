use std::fmt::Display;

use clap::{Parser, command};

#[derive(Debug)]
pub struct AppConfig {
    pub node_id: u16,
    pub api_port: u32,
    pub storage_path: String,
    pub member_list: Vec<NodeRpcInfo>,
}

impl Display for AppConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AppConfig {{ ")?;
        write!(f, "node_id: {}, ", self.node_id)?;
        write!(f, "api_port: {}, ", self.api_port)?;
        write!(f, "storage_path: {}, ", self.storage_path)?;

        write!(f, "member_list: [")?;
        for (i, node) in self.member_list.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", node)?;
        }
        write!(f, "]")?;
        write!(f, " }}")
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq, Default)]
pub struct NodeRpcInfo {
    pub node_id: u16,
    pub rpc_addr: String,
    pub rpc_port: u32,
}

impl Display for NodeRpcInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Node {{ node_id: {}, rpc_addr: {}, rpc_port: {} }}",
            self.node_id, self.rpc_addr, self.rpc_port
        )
    }
}

// ======================
// 命令行参数定义（核心）
// ======================
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// rest服务端口
    #[arg(long, required = true)]
    pub port: u32,

    #[arg(long, required = true)]
    pub node_id: u16,

    #[arg(long, required = true)]
    pub storage_path: String,

    /// 集群成员列表，逗号分隔 (例如：1@127.0.0.1:8080,2@127.0.0.1:8081)
    #[arg(long)]
    pub member_list: String,
}

impl CliArgs {
    /// 验证参数并构建 AppConfig
    pub fn validate(self) -> anyhow::Result<AppConfig> {
        // ======================
        // 严格校验格式：节点ID@IP:端口
        // 示例：1@127.0.0.1:8080,2@127.0.0.1:8081
        // ======================
        let member_list_vec = self
            .member_list
            .split(',')
            .map(|s| s.trim().to_string())
            .map(|member| {
                // 1. 必须包含 @ 符号
                let (id_part, addr_part) = member.split_once('@').ok_or_else(|| {
                    anyhow::anyhow!("集群成员格式错误：{}，必须是 节点ID@IP:端口 格式", member)
                })?;

                // 2. 节点ID必须是纯数字（u16）
                let node_id: u16 = id_part.parse().map_err(|_| {
                    anyhow::anyhow!("节点ID格式错误：{}，必须是数字（0-65535）", id_part)
                })?;

                // 3. 地址部分必须包含 : （IP:端口）
                if !addr_part.contains(':') {
                    return Err(anyhow::anyhow!(
                        "地址格式错误：{}，必须是 IP:端口 格式",
                        addr_part
                    ));
                }
                let (rpc_address, rpc_port) = member.split_once(':').ok_or_else(|| {
                    anyhow::anyhow!("集群成员格式错误：{}，必须是 节点ID@IP:端口 格式", member)
                })?;

                Ok(NodeRpcInfo {
                    node_id,
                    rpc_addr: rpc_address.to_string(),
                    rpc_port: rpc_port.parse().unwrap(),
                })
            })
            .collect::<anyhow::Result<Vec<NodeRpcInfo>>>()?;

        Ok(AppConfig {
            node_id: self.node_id,
            api_port: self.port,
            storage_path: self.storage_path,
            member_list: member_list_vec,
        })
    }
}

# 简介
一个为了学习Raft算法而编写的KV存储系统。  
使用Rust编写，最开始先实现了单机版本的KV存储，支持内存和RocksDB两种存储后端。
之后实现了分布式版本，使用OpenRaft库实现Raft算法，使用RocksDB作为持久化存储后端。  


## 启动命令

``bash

# 多节点（OpenRaft）后端，需要指定节点ID、端口、存储路径和成员列表
# member list format: <node_id>@<ip>:<port>,<node_id>@<ip>:<port>
# Example with 3 nodes:
#  - node 1: 1@127.0.0.1:9901
#  - node 2: 2@127.0.0.1:9902
#  - node 3: 3@127.0.0.1:9903
cargo run -- \
--node-id 1 \
--port 9901 \
--storage-path /tmp/kv-node-1 \
--member-list "1@127.0.0.1:9901,2@127.0.0.1:9902,3@127.0.0.1:9903"


cargo run -- \
--node-id 2 \
--port 9902 \
--storage-path /tmp/kv-node-2 \
--member-list "1@127.0.0.1:9901,2@127.0.0.1:9902,3@127.0.0.1:9903"



cargo run -- \
--node-id 3 \
--port 9903 \
--storage-path /tmp/kv-node-3 \
--member-list "1@127.0.0.1:9901,2@127.0.0.1:9902,3@127.0.0.1:9903"
```



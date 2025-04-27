# OpenAI-RS 开发实践指南

## 目录
- [开发环境配置](#开发环境配置)
- [常见开发场景](#常见开发场景)
- [最佳实践](#最佳实践)
- [调试技巧](#调试技巧)
- [性能优化](#性能优化)

## 开发环境配置

### 必要环境
1. Rust 工具链
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

2. 开发工具
- VS Code + rust-analyzer 插件
- LLDB 调试器
- cargo-watch（可选，用于开发热重载）

### 推荐配置

1. VS Code settings.json
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.cargo.features": "all",
    "editor.formatOnSave": true
}
```

2. 常用开发工具
```bash
cargo install cargo-watch   # 热重载
cargo install cargo-edit    # 依赖管理
cargo install cargo-expand  # 宏展开
```

## 常见开发场景

### 1. 添加新的处理器

以添加敏感词过滤处理器为例：

1. 在 domain/src/processor/ 下创建新文件
```rust
// domain/src/processor/sensitive.rs

use crate::event::InternalStreamEvent;
use async_trait::async_trait;
use std::collections::VecDeque;

pub struct SensitiveWordProcessor {
    words: Vec<String>
}

impl SensitiveWordProcessor {
    pub fn new(words: Vec<String>) -> Self {
        Self { words }
    }
}

#[async_trait]
impl Processor for SensitiveWordProcessor {
    async fn process(
        &self,
        event: &mut InternalStreamEvent,
        output_queue: &mut VecDeque<InternalStreamEvent>
    ) -> anyhow::Result<()> {
        if let Some(content) = &mut event.content {
            for word in &self.words {
                *content = content.replace(word, "***");
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sensitive_word_filtering() {
        let processor = SensitiveWordProcessor::new(vec!["敏感词".to_string()]);
        let mut event = InternalStreamEvent {
            role: Some("user".to_string()),
            content: Some("这是一个敏感词测试".to_string())
        };
        let mut queue = VecDeque::new();
        
        processor.process(&mut event, &mut queue).await.unwrap();
        
        assert_eq!(event.content, Some("这是一个***测试".to_string()));
    }
}
```

2. 注册处理器
```rust
// app/src/conversation.rs

let processors: Vec<Box<dyn Processor>> = vec![
    Box::new(SensitiveWordProcessor::new(vec![
        "敏感词1".to_string(),
        "敏感词2".to_string()
    ]))
];

let conversation = ConversationStream::new(processors);
```

### 2. 实现新的 Token Provider

以实现缓存 Token Provider 为例：

```rust
// infra/src/token_vault/cached.rs

use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use domain::token::TokenProvider;

pub struct CachedTokenProvider {
    inner: Box<dyn TokenProvider>,
    cache: Arc<RwLock<Option<String>>>,
}

impl CachedTokenProvider {
    pub fn new(provider: Box<dyn TokenProvider>) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(None)),
        }
    }
}

#[async_trait]
impl TokenProvider for CachedTokenProvider {
    async fn get_token(&self) -> anyhow::Result<String> {
        if let Some(token) = self.cache.read().await.clone() {
            return Ok(token);
        }
        
        let token = self.inner.get_token().await?;
        *self.cache.write().await = Some(token.clone());
        Ok(token)
    }
}
```

### 3. 添加新的 API 后端支持

1. 定义 API 类型
```rust
// domain/src/api/mod.rs

#[derive(Debug, Clone)]
pub enum ApiBackend {
    OpenAI,
    Claude,
    Custom(String),
}
```

2. 实现请求构建
```rust
// infra/src/dispatcher/builder.rs

impl RequestBuilder {
    pub fn build(&self, backend: ApiBackend) -> reqwest::RequestBuilder {
        match backend {
            ApiBackend::OpenAI => self.build_openai_request(),
            ApiBackend::Claude => self.build_claude_request(),
            ApiBackend::Custom(url) => self.build_custom_request(url),
        }
    }
}
```

## 最佳实践

### 1. 错误处理

使用 anyhow 和 thiserror 组合处理错误：

```rust
// domain/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Token not found")]
    TokenNotFound,
    
    #[error("Invalid event: {0}")]
    InvalidEvent(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
```

### 2. 配置管理

使用 config crate 管理配置：

```rust
// app/src/config.rs

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub processors: ProcessorConfig,
    pub api: ApiConfig,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

// 加载配置
pub fn load_config() -> anyhow::Result<AppConfig> {
    let config = config::Config::builder()
        .add_source(config::File::with_name("config/default"))
        .add_source(config::Environment::with_prefix("APP"))
        .build()?;
        
    config.try_deserialize()
}
```

### 3. 日志处理

使用 tracing 进行日志记录：

```rust
// infra/src/logging.rs

use tracing::{info, warn, error};

pub fn setup_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

// 使用示例
info!("Processing event: {:?}", event);
warn!("Token cache miss");
error!("API request failed: {}", err);
```

## 调试技巧

### 1. 流式数据调试

使用 tokio-console 调试异步流：

```bash
# 安装
cargo install tokio-console

# 运行时启用
TOKIO_CONSOLE=1 cargo run
```

### 2. 性能分析

使用 cargo flamegraph 生成性能火焰图：

```bash
# 安装
cargo install flamegraph

# 生成火焰图
cargo flamegraph --bin server

# 查看结果
open flamegraph.svg
```

## 性能优化

### 1. 内存优化

1. 使用对象池
```rust
use deadpool::managed::{Manager, Object, Pool};

pub struct ProcessorPool {
    pool: Pool<ProcessorManager>,
}

impl ProcessorPool {
    pub async fn get_processor(&self) -> impl Processor {
        self.pool.get().await?
    }
}
```

2. 避免不必要的克隆
```rust
// 不好的实践
let content = event.content.clone();

// 好的实践
let content = &event.content;
```

### 2. 并发优化

1. 使用 Stream 并行处理
```rust
use futures::stream::{self, StreamExt};

async fn process_events(events: Vec<Event>) {
    stream::iter(events)
        .map(|evt| process_single_event(evt))
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await;
}
```

2. 合理使用缓存
```rust
use moka::future::Cache;

let cache: Cache<String, String> = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(300))
    .build();
```

## 发布流程

### 1. 版本管理

遵循 Semantic Versioning：
- MAJOR.MINOR.PATCH
- MAJOR: 不兼容的 API 变更
- MINOR: 向后兼容的功能性新增
- PATCH: 向后兼容的问题修复

### 2. 发布检查清单

1. 更新版本号
2. 更新 CHANGELOG.md
3. 运行完整测试套件
4. 检查文档更新
5. 创建 git tag
6. 发布到 crates.io（如果需要）

## 常见问题解决

### 1. 编译错误

1. 特征约束相关
```rust
// 错误
fn process<T>(item: T) {}

// 正确
fn process<T: Send + Sync>(item: T) {}
```

2. 生命周期相关
```rust
// 错误
struct Holder { data: &str }

// 正确
struct Holder<'a> { data: &'a str }
```

### 2. 运行时错误

1. Token 获取失败
- 检查环境变量配置
- 验证 Token Provider 实现
- 查看网络连接状态

2. 流处理中断
- 检查错误处理逻辑
- 验证异步函数实现
- 检查资源释放时机

use actix_web::{web, App, HttpServer};
use domain::stream::StreamConfig;
use infra::{StreamDispatcher, TokenVault, StaticTokenProvider};
use interface::http_handler::{chat_handler, HandlerConfig};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init();

    // Initialize token vault with a default static provider
    let mut token_vault = TokenVault::new();
    token_vault.add_provider(
        "default".into(),
        Arc::new(StaticTokenProvider::new("default-token".into())),
    );
    let token_vault = Arc::new(token_vault);

    // Initialize stream dispatcher
    let stream_dispatcher = Arc::new(StreamDispatcher::new());

    // Create handler configuration
    let handler_config = web::Data::new(HandlerConfig {
        token_vault,
        stream_dispatcher,
        stream_config: StreamConfig::default(),
    });

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(handler_config.clone())
            .route("/chat", web::post().to(chat_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

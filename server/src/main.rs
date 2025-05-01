use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use core::{
    StaticUrlProvider,
    context::{StreamingProxyContext, StreamingProxyContextBuilder},
    token_provider::StaticTokenProvider,
};
use std::sync::Arc;
use tracing::{error, info};
use tracing_actix_web::TracingLogger;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{
    EnvFilter, Registry,
    fmt::{self, format::FmtSpan},
};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from Rust Intelligent Streaming Proxy Server!")
}

#[post("/v1/chat/completions")]
async fn chat_handler(
    req_body: web::Bytes,
    context: web::Data<StreamingProxyContext>,
) -> impl Responder {
    match context.process_request(req_body.into()).await {
        Ok(stream) => HttpResponse::Ok()
            .content_type("text/event-stream")
            .insert_header(("Cache-Control", "no-cache"))
            .streaming(tokio_stream::wrappers::ReceiverStream::new(stream)),
        Err(e) => {
            error!(error = %e, "Error processing request");
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing with a more detailed configuration
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,server=debug,core=debug"));

    let formatting_layer = fmt::layer()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .pretty();

    let subscriber = Registry::default().with(env_filter).with(formatting_layer);

    // Set the subscriber as the default
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    info!("Starting server on http://127.0.0.1:8080");
    info!("Reading OPENAI_API_KEY and OPENAI_MODEL from environment variables.");

    let context = StreamingProxyContextBuilder::new()
        .with_url_provider(Arc::new(StaticUrlProvider::new(
            "https://api.siliconflow.cn/v1/chat/completions".to_string(),
        )))
        .with_token_provider(Arc::new(StaticTokenProvider::new(
            "sk-wezkvfciyxaadlxzygdfulbayklquysmyzrefpncaugnkhbf".to_string(),
        )))
        .build();

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(web::Data::new(context.clone()))
            .service(hello)
            .service(chat_handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

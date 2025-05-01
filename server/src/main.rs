use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use core::{
    StaticUrlProvider,
    context::{StreamingProxyContext, StreamingProxyContextBuilder},
    token_provider::StaticTokenProvider,
};
use std::{env, sync::Arc}; // For reading environment variables

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
            .streaming(stream),
        Err(e) => {
            eprintln!("Error processing request: {}", e);
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    println!("Starting server on http://127.0.0.1:8080");
    println!("Reading OPENAI_API_KEY and OPENAI_MODEL from environment variables.");

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
            .app_data(web::Data::new(context.clone()))
            .service(hello)
            .service(chat_handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

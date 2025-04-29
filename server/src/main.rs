use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use core::{
    context::{StreamingProxyContext, StreamingProxyContextBuilder},
    token_provider::StaticTokenProvider,
};
use std::{env, sync::Arc}; // For reading environment variables

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from Rust Intelligent Streaming Proxy Server!")
}

#[post("/chat")]
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

    // Create context with custom token provider
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        eprintln!("WARN: OPENAI_API_KEY not set, using placeholder.");
        "YOUR_STATIC_OPENAI_KEY".to_string()
    });

    let context = StreamingProxyContextBuilder::new()
        .with_token_provider(Arc::new(StaticTokenProvider::new(api_key)))
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

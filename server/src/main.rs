use actix_web::{App, Error, HttpResponse, HttpServer, Responder, get, post, web};
use core::{
    StaticUrlProvider,
    client_provider::StaticClientProvider,
    forwarder::StreamForwarder,
    openai_types::StreamEvent,
    parser::{JsonParser, RequestParser},
    processor_chain::ProcessorChain,
    token_provider::StaticTokenProvider,
};
use futures_util::StreamExt;
use std::env; // For reading environment variables

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from Rust Intelligent Streaming Proxy Server!") // Updated message
}

#[post("/chat")]
async fn chat_handler(req_body: web::Bytes) -> impl Responder {
    // --- Initialization (Consider using web::Data for shared state) ---
    let client_provider = Box::new(StaticClientProvider::new());
    // Read API Key and Model from environment variables or use defaults
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        eprintln!("WARN: OPENAI_API_KEY not set, using placeholder.");
        "YOUR_STATIC_OPENAI_KEY".to_string() // Replace with your actual key or better handling
    });

    let url_provider =
        StaticUrlProvider::new("https://api.openai.com/v1/chat/completions".to_string());
    let token_provider = StaticTokenProvider::new(api_key);
    // Ensure StreamForwarder::new is public
    let forwarder = StreamForwarder::new(client_provider);
    // Ensure RequestParser::new is public
    let parser = JsonParser::new();
    // Ensure ProcessorChain::new is public
    let processor_chain = ProcessorChain::new(vec![]); // Empty chain for MVP
    // --- Processing ---

    // 1. Parse Request
    // Ensure RequestParser::parse is public
    let openai_chat_completion_request = match parser.parse(&req_body) {
        Ok(openai_chat_completion_request) => openai_chat_completion_request,
        Err(e) => {
            eprintln!("Request parsing error: {:?}", e);
            return HttpResponse::BadRequest().body(format!("Invalid request format: {}", e));
        }
    };

    // 2. Processor Chain
    // Ensure ProcessorChain::execute is public
    let processed_messages = match processor_chain
        .execute(openai_chat_completion_request)
        .await
    {
        Ok(messages) => messages,
        Err(e) => {
            eprintln!("Processing error: {:?}", e);
            return HttpResponse::InternalServerError()
                .body(format!("Error during processing: {}", e));
        }
    };

    // 3. & 4. Forward to OpenAI and get response stream
    // Ensure StreamForwarder::forward is public
    let response_stream_result = forwarder
        .forward(processed_messages, &token_provider, &url_provider)
        .await;

    match response_stream_result {
        Ok(openai_stream) => {
            // 5. & 6. Map OpenAI stream to SSE format
            let sse_stream = openai_stream.map(|event_result| match event_result {
                Ok(event) => match event {
                    StreamEvent::Chunk(chunk) => {
                        let json = serde_json::to_string(&chunk)?;
                        Ok::<web::Bytes, Error>(web::Bytes::from(format!("data: {}\n\n", json)))
                    }
                    StreamEvent::Done => {
                        Ok::<web::Bytes, Error>(web::Bytes::from("data: [DONE]\n\n"))
                    }
                },
                Err(e) => {
                    eprintln!("Error in OpenAI response stream: {:?}", e);
                    let err_json = serde_json::json!({"error": e.to_string()});
                    Ok::<web::Bytes, Error>(web::Bytes::from(format!(
                        "event: error\ndata: {}\n\n",
                        err_json
                    )))
                }
            });

            HttpResponse::Ok()
                .content_type("text/event-stream")
                .insert_header(("Cache-Control", "no-cache")) // Recommended for SSE
                .streaming(sse_stream)
        }
        Err(e) => {
            eprintln!("Stream forwarding error: {:?}", e);
            HttpResponse::InternalServerError().body(format!("Error forwarding request: {}", e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info")); // Use env_logger

    println!("Starting server on http://127.0.0.1:8080");
    println!("Reading OPENAI_API_KEY and OPENAI_MODEL from environment variables.");

    HttpServer::new(|| App::new().service(hello).service(chat_handler))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

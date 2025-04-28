use actix_web::{App, Error, HttpResponse, HttpServer, Responder, get, post, web};
use core::{
    InternalStreamEvent,
    forwarder::StreamForwarder, // Make sure StreamForwarder is pub in core/src/forwarder.rs
    parser::RequestParser,
    processor::ProcessorChain,
    // sse::SseHandler, // Not used directly in this version, formatting is local
    token_provider::StaticTokenProvider,
};
use futures_util::StreamExt;
use reqwest::Client;
use std::env; // For reading environment variables

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from Rust Intelligent Streaming Proxy Server!") // Updated message
}

// Helper function to format SSE data
fn format_sse_event(event: &InternalStreamEvent) -> Result<web::Bytes, Error> {
    // Basic formatting, assuming content is the main data
    if let Some(content) = &event.content {
        // Ensure content doesn't contain newlines which break SSE
        let clean_content = content.replace('\n', " ");
        let json_data = serde_json::json!({
            "role": event.role,
            "content": clean_content, // Use cleaned content
        });
        // Format as "data: {...}\n\n"
        let formatted_string = format!("data: {}\n\n", json_data.to_string());
        Ok(web::Bytes::from(formatted_string))
    } else {
        // Decide how to handle events without content (e.g., skip, send metadata)
        Ok(web::Bytes::new()) // Send empty bytes for now
    }
}

#[post("/chat")]
async fn chat_handler(req_body: web::Bytes) -> impl Responder {
    // --- Initialization (Consider using web::Data for shared state) ---
    let client = Client::new();
    // Read API Key and Model from environment variables or use defaults
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        eprintln!("WARN: OPENAI_API_KEY not set, using placeholder.");
        "YOUR_STATIC_OPENAI_KEY".to_string() // Replace with your actual key or better handling
    });
    let model_name = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());

    let token_provider = StaticTokenProvider::new(api_key);
    // Ensure StreamForwarder::new is public
    let forwarder = StreamForwarder::new(client, model_name);
    // Ensure RequestParser::new is public
    let parser = RequestParser::new();
    // Ensure ProcessorChain::new is public
    let processor_chain = ProcessorChain::new(vec![]); // Empty chain for MVP
    // --- Processing ---

    // 1. Parse Request
    // Ensure RequestParser::parse is public
    let initial_events = match parser.parse(&req_body) {
        Ok(events) => events,
        Err(e) => {
            eprintln!("Request parsing error: {:?}", e);
            return HttpResponse::BadRequest().body(format!("Invalid request format: {}", e));
        }
    };

    // 2. Processor Chain
    // Ensure ProcessorChain::execute is public
    let processed_events = match processor_chain.execute(initial_events).await {
        Ok(events) => events,
        Err(e) => {
            eprintln!("Processing error: {:?}", e);
            return HttpResponse::InternalServerError()
                .body(format!("Error during processing: {}", e));
        }
    };

    // 3. & 4. Forward to OpenAI and get response stream
    // Ensure StreamForwarder::forward is public
    let response_stream_result = forwarder.forward(processed_events, &token_provider).await;

    match response_stream_result {
        Ok(openai_stream) => {
            // 5. & 6. Map OpenAI stream to SSE format
            let sse_stream = openai_stream.map(|event_result| match event_result {
                Ok(event) => format_sse_event(&event), // Format Ok(InternalStreamEvent) -> Result<Bytes, Error>
                Err(e) => {
                    // Log the stream error and potentially send an SSE error event
                    eprintln!("Error in OpenAI response stream: {:?}", e);
                    // Example: Send an error event (customize format as needed)
                    let err_json = serde_json::json!({"error": e.to_string()});
                    Ok(web::Bytes::from(format!(
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

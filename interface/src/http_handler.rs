use actix_web::{web, HttpResponse, ResponseError};
use app::ConversationStream;
use domain::event::InternalStreamEvent;
use domain::processor::Processor;
use domain::stream::StreamConfig;
use futures_util::stream::StreamExt;
use infra::{StreamDispatcher, TokenVault};
use std::fmt;
use std::sync::Arc;

#[derive(Debug)]
pub struct AppError(anyhow::Error);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::InternalServerError().json(format!("Internal Server Error: {}", self.0))
    }

    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError(err)
    }
}

/// Configuration for the HTTP handlers
pub struct HandlerConfig {
    pub token_vault: Arc<TokenVault>,
    pub stream_dispatcher: Arc<StreamDispatcher>,
    pub stream_config: StreamConfig,
}

/// Handles chat stream requests
pub async fn chat_handler(
    data: web::Data<HandlerConfig>,
    event: web::Json<InternalStreamEvent>,
) -> Result<HttpResponse, AppError> {
    // Create a new conversation stream with empty processor chain for now
    // In a real implementation, you would load processors from configuration
    let conversation = ConversationStream::with_config(
        Vec::<Box<dyn Processor>>::new(),
        data.stream_config.clone(),
    );

    // Process the event through the processor chain
    let processed_events = conversation.handle(event.into_inner()).await?;

    // Get token for the API call
    let token = data.token_vault.get_token("default").await?;

    // Forward processed events to external service
    let response_stream = data
        .stream_dispatcher
        .dispatch(
            processed_events,
            token.value,
            "https://api.example.com/v1/chat".into(),
        )
        .await?;

    // Convert raw bytes to SSE format
    let sse_stream = response_stream.map(|result| match result {
        Ok(bytes) => Ok(web::Bytes::from(format!(
            "data: {}\n\n",
            String::from_utf8_lossy(&bytes)
        ))),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    });

    Ok(HttpResponse::Ok()
        .insert_header(("Content-Type", "text/event-stream"))
        .streaming(Box::pin(sse_stream)))
}

use std::{collections::HashMap, sync::Arc};

use actix_cors::Cors;
use actix_web::{
    middleware,
    web::{self},
    App, HttpRequest, HttpResponse, HttpServer,
};
use anyhow::Result;
use bytes::BytesMut;
use futures_util::StreamExt;
use llm_proxy_core::Pipeline;
use llm_proxy_openai::ChatCompletionRequest;
use tracing::{error, info};

use crate::config;

/// Application state shared across request handlers
pub struct AppState {
    config: Arc<config::Config>,
    pipelines: Arc<tokio::sync::RwLock<PipelineRegistry>>,
}

/// Registry of pre-configured pipelines
pub struct PipelineRegistry {
    pipelines: HashMap<String, Arc<Pipeline<ChatCompletionRequest>>>,
}

impl PipelineRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
        }
    }

    #[must_use]
    pub fn get(&self, route_id: &str) -> Option<Arc<Pipeline<ChatCompletionRequest>>> {
        self.pipelines.get(route_id).cloned()
    }

    pub fn insert(&mut self, route_id: String, pipeline: Arc<Pipeline<ChatCompletionRequest>>) {
        self.pipelines.insert(route_id, pipeline);
    }
}

impl Default for PipelineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Configure and start the HTTP server
///
/// # Errors
///
/// This function will return an error if the server cannot be started.
#[must_use]
pub async fn run_server(config: config::Config) -> Result<()> {
    let config = Arc::new(config);
    let pipelines = Arc::new(tokio::sync::RwLock::new(PipelineRegistry::new()));
    let server_config = config.server.clone();

    let app_state = web::Data::new(AppState {
        config: config.clone(),
        pipelines,
    });

    let server = HttpServer::new(move || {
        let config = config.clone();
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin_fn(move |origin, _req_head| {
                let origin_str = origin.to_str().unwrap_or_default();
                config
                    .server
                    .cors_allowed_origins
                    .iter()
                    .any(|allowed| allowed == "*" || allowed == origin_str)
            })
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(app_state.clone())
            .default_service(web::route().to(handle_request))
    })
    .bind((server_config.host, server_config.port))?
    .run();

    info!(
        "Server running at http://{}:{}",
        server_config.host, server_config.port
    );

    server.await?;
    Ok(())
}

/// Generic request handler that routes requests based on configuration
#[allow(clippy::future_not_send, clippy::cognitive_complexity)]
async fn handle_request(
    req: HttpRequest,
    payload: web::Payload,
    state: web::Data<AppState>,
) -> HttpResponse {
    let path = req.uri().path();

    // Find matching route
    let Some(route) = state.config.find_route(path) else {
        return HttpResponse::NotFound().body(format!("No route found for path: {path}"));
    };

    // Get or create pipeline for this route
    let pipeline = match get_pipeline_for_route(&state, route).await {
        Ok(pipeline) => pipeline,
        Err(e) => {
            error!(error = %e, "Failed to get pipeline for route");
            return HttpResponse::InternalServerError().body(format!("Pipeline error: {e}"));
        }
    };

    // Read request body
    let body = match read_request_body(payload).await {
        Ok(body) => body,
        Err(e) => {
            error!(error = %e, "Failed to read request body");
            return HttpResponse::BadRequest().body(format!("Invalid request body: {e}"));
        }
    };

    // Execute pipeline
    let rx = match pipeline.execute(body.freeze()).await {
        Ok(rx) => rx,
        Err(e) => {
            error!(error = %e, "Pipeline execution failed");
            return HttpResponse::InternalServerError().body(format!("Pipeline error: {e}"));
        }
    };

    // Stream response back to client
    let receiver_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(receiver_stream)
}

/// Read the entire request body into a buffer
#[allow(clippy::future_not_send)]
async fn read_request_body(mut payload: web::Payload) -> Result<BytesMut> {
    let mut body = BytesMut::new();
    while let Some(chunk) = payload.next().await {
        body.extend_from_slice(&chunk?);
    }
    Ok(body)
}

/// Get or create a pipeline for the given route
async fn get_pipeline_for_route(
    state: &AppState,
    route: &config::RouteConfig,
) -> Result<Arc<Pipeline<ChatCompletionRequest>>> {
    // Check if we already have a pipeline for this route
    let value = state.pipelines.read().await.get(&route.path_prefix);
    if let Some(pipeline) = value {
        return Ok(pipeline);
    }

    // No existing pipeline - create one
    #[cfg(feature = "openai")]
    if let Some(llm_config) = state.config.llm.get(&route.target_llm) {
        if llm_config.provider == "openai" {
            let pipeline = create_openai_pipeline(llm_config, route);

            // Store it in the registry
            state
                .pipelines
                .write()
                .await
                .insert(route.path_prefix.clone(), pipeline.clone());

            return Ok(pipeline);
        }
    }

    Err(anyhow::anyhow!(
        "No pipeline implementation available for provider: {}",
        route.target_llm
    ))
}

#[cfg(feature = "openai")]
fn create_openai_pipeline(
    llm_config: &config::LLMConfig,
    route: &config::RouteConfig,
) -> Arc<Pipeline<ChatCompletionRequest>> {
    let processors = vec![];

    let pipeline = llm_proxy_openai::create_chat_pipeline(
        processors,
        Some(&llm_config.token_env),
        Some(&llm_config.base_url),
    );

    Arc::new(pipeline)
}

use anyhow::Result;
use axum::{
    extract::DefaultBodyLimit,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod handlers;
mod models;
mod services;
mod utils;

use config::Config;
use handlers::{case_handlers, document_handlers, health_handlers, user_handlers};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "enterprise_doccase_platform=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Enterprise Document and Case Management Platform");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Setup database
    let db_pool = setup_database(&config.database_url).await?;
    info!("Database connection established");

    // Build application with routes
    let app = create_app(db_pool).await;

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.server_address).await?;
    info!("Server listening on {}", config.server_address);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn setup_database(database_url: &str) -> Result<SqlitePool> {
    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        info!("Creating database {}", database_url);
        match Sqlite::create_database(database_url).await {
            Ok(_) => info!("Database created successfully"),
            Err(error) => panic!("Error creating database: {}", error),
        }
    }

    // Connect to database
    let db_pool = SqlitePool::connect(database_url).await?;

    // Run migrations
    info!("Running database migrations");
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    info!("Database migrations completed");

    Ok(db_pool)
}

async fn create_app(db_pool: SqlitePool) -> Router {
    Router::new()
        // Health check endpoints
        .route("/health", get(health_handlers::health_check))
        .route("/ready", get(health_handlers::readiness_check))
        
        // Authentication endpoints
        .route("/api/auth/register", post(user_handlers::register))
        .route("/api/auth/login", post(user_handlers::login))
        
        // Document management endpoints
        .route("/api/documents", get(document_handlers::list_documents))
        .route("/api/documents", post(document_handlers::create_document))
        .route("/api/documents/:id", get(document_handlers::get_document))
        .route("/api/documents/:id", post(document_handlers::update_document))
        .route("/api/documents/:id/upload", post(document_handlers::upload_file))
        .route("/api/documents/:id/download", get(document_handlers::download_file))
        
        // Case management endpoints
        .route("/api/cases", get(case_handlers::list_cases))
        .route("/api/cases", post(case_handlers::create_case))
        .route("/api/cases/:id", get(case_handlers::get_case))
        .route("/api/cases/:id", post(case_handlers::update_case))
        .route("/api/cases/:id/documents", get(case_handlers::get_case_documents))
        .route("/api/cases/:id/documents/:doc_id", post(case_handlers::add_document_to_case))
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit
                .layer(middleware::from_fn(request_logging_middleware))
        )
        .with_state(db_pool)
}

async fn request_logging_middleware(
    request: axum::extract::Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    
    let response = next.run(request).await;
    
    info!("{} {} - {}", method, uri, response.status());
    response
}
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

mod app_state;
mod config;
mod handlers;
mod models;
mod services;
mod utils;

use app_state::AppState;
use config::Config;
use handlers::{health_handlers};

mod user_handlers_secure;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "enterprise_doccase_platform=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting Enterprise Document and Case Management Platform");
    info!("🔒 Security-enhanced version with PR 3 improvements");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Setup database
    let db_pool = setup_database(&config.database_url).await?;
    info!("Database connection established");

    // Create application state
    let app_state = AppState::new(db_pool, config.clone());

    // Build application with enhanced security
    let app = create_secure_app(app_state).await;

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.server_address).await?;
    info!("🌐 Server listening on {}", config.server_address);
    info!("🔒 Enhanced security features enabled");
    info!("📖 Visit http://{}/health for health check", config.server_address);
    info!("📖 Visit http://{}/api for API documentation", config.server_address);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn setup_database(database_url: &str) -> Result<SqlitePool> {
    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        info!("Creating database {}", database_url);
        match Sqlite::create_database(database_url).await {
            Ok(_) => info!("Database created successfully"),
            Err(error) => panic!("error: {}", error),
        }
    }

    // Create connection pool
    let db_pool = SqlitePool::connect(database_url).await?;
    info!("Connected to database");

    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    info!("Database migrations completed");

    Ok(db_pool)
}

async fn create_secure_app(app_state: AppState) -> Router {
    Router::new()
        // Health check endpoints
        .route("/health", get(health_handlers::health_check))
        .route("/ready", get(health_handlers::readiness_check))
        
        // API info endpoint with security details
        .route("/api", get(secure_api_info))
        
        // Enhanced authentication endpoints
        .route("/api/auth/register", post(user_handlers_secure::register_secure))
        .route("/api/auth/login", post(user_handlers_secure::login_secure))
        
        // Add security middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(DefaultBodyLimit::max(10 * 1024 * 1024)) // 10MB limit
                .layer(middleware::from_fn(security_logging_middleware))
        )
        .with_state(app_state.db) // Use just the DB pool for compatibility
}

async fn security_logging_middleware(
    mut request: axum::extract::Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let user_agent = request.headers()
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();
    
    info!(
        "{} {} - {} ({}ms) - User-Agent: {}",
        method,
        uri,
        response.status(),
        duration.as_millis(),
        user_agent
    );
    
    response
}

async fn secure_api_info() -> axum::response::Json<serde_json::Value> {
    use serde_json::json;
    
    axum::response::Json(json!({
        "service": "Enterprise Document and Case Management Platform",
        "version": env!("CARGO_PKG_VERSION"),
        "security_level": "Enhanced",
        "features": {
            "secure_authentication": true,
            "role_based_access_control": true,
            "password_complexity_validation": true,
            "brute_force_protection": true,
            "audit_logging": true,
            "jwt_with_configurable_secret": true
        },
        "pr3_improvements": {
            "authentication": "✅ JWT secrets from environment, enhanced claims with roles/permissions",
            "validation": "✅ Password complexity requirements, enhanced input validation",
            "security": "✅ Brute force protection, audit logging, higher bcrypt cost",
            "error_handling": "✅ Structured error responses with security considerations"
        },
        "endpoints": {
            "authentication": {
                "register": "POST /api/auth/register - Enhanced with security validation",
                "login": "POST /api/auth/login - Enhanced with rate limiting and audit logging"
            },
            "health": {
                "health_check": "GET /health",
                "readiness": "GET /ready"
            }
        },
        "timestamp": chrono::Utc::now()
    }))
}
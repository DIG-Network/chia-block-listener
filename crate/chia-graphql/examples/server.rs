use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::{
    extract::Extension,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use chia_graphql::{ChiaGraphql, DatabaseConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

async fn graphql_handler(
    Extension(graphql): Extension<Arc<ChiaGraphql>>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    graphql.schema().execute(req.into_inner()).await.into()
}

async fn graphql_playground() -> impl IntoResponse {
    Html(async_graphql::http::playground_source(
        async_graphql::http::GraphQLPlaygroundConfig::new("/graphql"),
    ))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load database configuration from environment
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:chia_blockchain.db".to_string());

    let config = if db_url.starts_with("postgres") {
        DatabaseConfig::Postgres {
            connection_string: db_url,
        }
    } else {
        DatabaseConfig::Sqlite {
            file_path: db_url.replace("sqlite:", ""),
        }
    };

    // Create ChiaGraphql instance
    let graphql = Arc::new(ChiaGraphql::from_config(config).await?);

    // Build the router
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/playground", get(graphql_playground))
        .layer(Extension(graphql))
        .layer(CorsLayer::permissive());

    // Start the server
    let addr: SocketAddr = "0.0.0.0:8080".parse()?;
    println!("GraphQL playground available at http://localhost:8080/playground");
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
} 
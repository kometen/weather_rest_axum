mod repositories;
mod models;
mod persistence;

use axum::{
    extract::{Extension},
    routing::get,
    Router,
};

use bb8::{Pool};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{NoTls};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
/*    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_tokio_postgres=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // postgresql setup
    let manager =
        PostgresConnectionManager::new_from_stringlike(database_url, NoTls).unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();
*/
    let pool = persistence::db_setup().await;
    // routes
    let app = Router::new()
        .route("/measurements/:site_id/:rows", get(repositories::get_weather_single_location),
        )
        .layer(Extension(pool));

    // run with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

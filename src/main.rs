mod controllers;
mod models;
mod persistence;

use axum::{
    extract::{Extension},
    routing::get,
    Router,
};

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // db-setup
    let pool = persistence::db_setup().await;
    // routes
    let app = Router::new()
        .route("/measurements/:site_id/:rows", get(controllers::get_weather_single_location),
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

use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{NoTls};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn db_setup() -> Pool<PostgresConnectionManager<NoTls>> {
    dotenv::dotenv().ok();
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
    Pool::builder().build(manager).await.unwrap()
}
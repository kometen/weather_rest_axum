use axum::{
    extract::{Extension},
    http::StatusCode,
    routing::get,
    Router,
};

use bb8::{Pool};
use bb8_postgres::PostgresConnectionManager;
use std::net::SocketAddr;
use axum::extract::Path;
use chrono::{DateTime, TimeZone, Utc};
use dotenv::Error;
use serde::Serialize;
use tokio_postgres::{NoTls, Row};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Serialize)]
pub struct MeasurementsSingleLocation {
    id: i32,
    name: String,
    latitude: String,
    longitude: String,
    measurement_time_default: DateTime<Utc>,
    measurements: serde_json::Value
}

impl MeasurementsSingleLocation {
    pub fn new(
        id: i32,
        name: String,
        latitude: String,
        longitude: String,
        measurement_time_default: DateTime<Utc>,
        measurements: serde_json::Value
    ) -> Self {
        Self {
            id,
            name,
            latitude,
            longitude,
            measurement_time_default,
            measurements
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    println!("{}", database_url);

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

    // routes
    let app = Router::new()
        .route("/measurements/:site_id/:rows", get(get_weather_single_location),
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

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

async fn get_weather_single_location(
    Path((site_id, rows)): Path<(i32, i32)>,
    Extension(pool): Extension<ConnectionPool>,
) -> Result<String, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    let id = if site_id < 0 { 0 } else { site_id };
    let r = if rows > 144 { 144 } else if rows < 0 { 0 } else { rows };

    println!("get rows");
    let rows= conn
        .query("select * from measurements_single_location_function($1,$2)", &[&id, &r])
        .await
        .map_err(internal_error)?;

    let measurements: Vec<MeasurementsSingleLocation> = rows
        .into_iter()
        .map(|row| MeasurementsSingleLocation::try_from(&row).unwrap())
        .collect();

    let j_measurements = serde_json::to_string(&measurements).unwrap();

    Ok(j_measurements.to_string())
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

impl<'a> TryFrom<&'a Row> for MeasurementsSingleLocation {
    type Error = Error;

    fn try_from(row: &'a Row) -> Result<Self, Self::Error> {
        let tg_id = row.try_get("id");
        let id = match tg_id {
            Ok(id) => id,
            Err(_) => -1
        };

        let tg_name = row.try_get("name");
        let name = match tg_name {
            Ok(tg) => tg,
            Err(_) => "unknown site".to_string()
        };

        let tg_latitude = row.try_get("latitude");
        let latitude = match tg_latitude {
            Ok(tg) => tg,
            Err(_) => "-0.1".to_string()
        };

        let tg_longitude = row.try_get("longitude");
        let longitude = match tg_longitude {
            Ok(tg) => tg,
            Err(_) => "-0.1".to_string()
        };

        let tg_measurement_time_default = row.try_get("measurement_time_default");
        let measurement_time_default= match tg_measurement_time_default {
            Ok(tg) => tg,
            Err(_) => Utc.ymd(1967, 10, 28).and_hms(10, 11, 12)
        };

        let measurements = row.try_get("measurements").unwrap();
        /*let measurements = match tg_measurements {
            Ok(tg) => tg,
            Err(_) => serde_json::from_str(r#"[{"invalid":true]"#)
        };*/

        Ok(Self {
            id,
            name,
            latitude,
            longitude,
            measurement_time_default,
            measurements
        })
    }
}

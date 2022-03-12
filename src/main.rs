use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    routing::get,
    Router,
};

use bb8::{Pool, PooledConnection};
use bb8_postgres::PostgresConnectionManager;
use std::net::SocketAddr;
use chrono::{DateTime, TimeZone, Utc};
use dotenv::Error;
use serde_json::Value;
use tokio_postgres::{NoTls, Row};
use tokio_postgres::types::{Date, Timestamp, FromSql};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use rust_decimal::prelude::*;

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
        .route("/", get(using_connection_pool_extractor).post(using_connection_pool_extractor),
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

async fn using_connection_pool_extractor(
    Extension(pool): Extension<ConnectionPool>,
) -> Result<String, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    println!("get rows");
    let rows= conn
        .query("select * from measurements_single_location_function(1,10)", &[])
        .await
        .map_err(internal_error)?;

    let measurements: Vec<MeasurementsSingleLocation> = rows
        .into_iter()
        .map(|row| MeasurementsSingleLocation::try_from(&row).unwrap())
        .collect();

    /*
    for r in &rows {
        let tg_name = r.try_get(1);
        let name = match tg_name {
            Ok(tg_name) => tg_name,
            Err(_) => "Invalid site"
        };
        println!("name: {}", name);

        let tg_latitude = r.try_get(2);
        let latitude = match tg_latitude {
            Ok(tg) => Decimal::from_str(tg),
            Err(_) => Decimal::from_str("-1.0")
        };
        let latitude = latitude.unwrap();
        println!("latitude: {}", latitude);

        let tg_longitude = r.try_get(3);
        let longitude = match tg_longitude {
            Ok(tg) => Decimal::from_str(tg),
            Err(_) => Decimal::from_str("-1.0")
        };
        let longitude = longitude.unwrap();
        println!("latitude: {}", longitude);

        let tg_measurement_time_default = r.try_get(4);
        println!("measurement_time_default: {}", tg_measurement_time_default.unwrap());

    }*/

    //let two = rows.get(0);

    Ok("2".to_string())
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
        println!("id: {}", &id);
        let tg_name = row.try_get("name");
        let name = match tg_name {
            Ok(tg) => tg,
            Err(_) => "unknown site".to_string()
        };
        println!("name: {}", &name);

        let tg_latitude = row.try_get("latitude");
        let latitude = match tg_latitude {
            Ok(tg) => tg,
            Err(_) => "-0.1".to_string()
        };
        println!("latitude: {}", &latitude);

        let tg_longitude = row.try_get("longitude");
        let longitude = match tg_longitude {
            Ok(tg) => tg,
            Err(_) => "-0.1".to_string()
        };
        println!("longitude: {}", &longitude);

        let tg_measurement_time_default = row.try_get("measurement_time_default");
        let measurement_time_default= match tg_measurement_time_default {
            Ok(tg) => tg,
            Err(_) => Utc.ymd(1967, 10, 28).and_hms(10, 11, 12)
        };
        println!("measurement_time_default: {}", measurement_time_default);

        let measurements = row.try_get("measurements").unwrap();
        /*let measurements = match tg_measurements {
            Ok(tg) => tg,
            Err(_) => serde_json::from_str(r#"[{"invalid":true]"#)
        };*/
        println!("measurements: {}", measurements);

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

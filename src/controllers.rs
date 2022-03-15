use bb8::{Pool};
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{NoTls, Row};
use axum::{
    extract::{Path, Extension},
    http::StatusCode,
};
use dotenv::Error;
use chrono::{TimeZone, Utc};


type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

pub(crate) async fn get_weather_single_location(
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

    let measurements: Vec<crate::models::MeasurementsSingleLocation> = rows
        .into_iter()
        .map(|row| crate::models::MeasurementsSingleLocation::try_from(&row).unwrap())
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

impl<'a> TryFrom<&'a Row> for crate::models::MeasurementsSingleLocation {
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

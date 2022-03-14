use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MeasurementsSingleLocation {
    pub(crate) id: i32,
    pub(crate) name: String,
    pub(crate) latitude: String,
    pub(crate) longitude: String,
    pub(crate) measurement_time_default: DateTime<Utc>,
    pub(crate) measurements: serde_json::Value
}

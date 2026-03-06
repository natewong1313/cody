use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;
use tonic::Status;
use uuid::Uuid;

pub fn parse_uuid(field: &str, value: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value).map_err(|e| Status::invalid_argument(format!("invalid {field}: {e}")))
}

pub fn timestamp_to_naive_datetime(
    field: &str,
    value: Option<Timestamp>,
) -> Result<NaiveDateTime, Status> {
    let ts = value.ok_or_else(|| Status::invalid_argument(format!("missing {field}")))?;

    if !(0..1_000_000_000).contains(&ts.nanos) {
        return Err(Status::invalid_argument(format!(
            "invalid {field}: nanos out of range"
        )));
    }

    DateTime::<Utc>::from_timestamp(ts.seconds, ts.nanos as u32)
        .map(|dt| dt.naive_utc())
        .ok_or_else(|| Status::invalid_argument(format!("invalid {field}: timestamp out of range")))
}

pub fn naive_datetime_to_timestamp(dt: NaiveDateTime) -> Timestamp {
    let dt = dt.and_utc();
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}

pub fn optional_naive_datetime_to_timestamp(value: Option<NaiveDateTime>) -> Option<Timestamp> {
    value.map(naive_datetime_to_timestamp)
}

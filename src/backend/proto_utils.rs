use chrono::NaiveDateTime;
use tonic::Status;
use uuid::Uuid;

pub fn parse_uuid(field: &str, value: &str) -> Result<Uuid, Status> {
    Uuid::parse_str(value).map_err(|e| Status::invalid_argument(format!("invalid {field}: {e}")))
}

pub fn parse_naive_datetime(field: &str, value: &str) -> Result<NaiveDateTime, Status> {
    NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f"))
        .map_err(|e| Status::invalid_argument(format!("invalid {field}: {e}")))
}

pub fn format_naive_datetime(dt: NaiveDateTime) -> String {
    dt.format("%Y-%m-%d %H:%M:%S%.f").to_string()
}

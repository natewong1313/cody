use tonic::Status;

pub mod project;
pub mod session;

pub fn required_field<T>(field: Option<T>, field_name: &'static str) -> Result<T, Status> {
    field.ok_or_else(|| Status::invalid_argument(format!("missing {field_name}")))
}

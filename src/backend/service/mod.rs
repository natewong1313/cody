use tonic::Status;

pub mod message;
pub mod project;
pub mod session;

#[cfg(test)]
mod message_test;
#[cfg(test)]
mod mod_test;
#[cfg(test)]
mod project_test;
#[cfg(test)]
mod session_test;
#[cfg(test)]
mod test_helpers;

pub fn required_field<T>(field: Option<T>, field_name: &'static str) -> Result<T, Status> {
    field.ok_or_else(|| Status::invalid_argument(format!("missing {field_name}")))
}

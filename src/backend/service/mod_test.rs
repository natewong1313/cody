use tonic::Code;

use crate::backend::service::required_field;

#[test]
fn required_field_returns_value_when_present() {
    let value = required_field(Some(7_u32), "count").expect("present field should succeed");
    assert_eq!(value, 7);
}

#[test]
fn required_field_returns_invalid_argument_when_missing() {
    let err = required_field::<u32>(None, "project").expect_err("missing field should fail");

    assert_eq!(err.code(), Code::InvalidArgument);
    assert!(err.message().contains("missing project"));
}

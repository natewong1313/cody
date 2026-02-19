pub mod project;
pub mod session;

/// Grabs a database connection or returns an error if its mutex lock is poisoned
#[macro_export]
macro_rules! with_db_conn {
    ($self:expr, $conn:ident, $body:block) => {{
        let $conn = $self
            .ctx
            .db
            .lock()
            .map_err(|_| crate::backend::db::DatabaseError::PoisonedLock)?;
        $body
    }};
}

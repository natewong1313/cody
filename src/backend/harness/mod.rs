use crate::backend::repo::session::Session;

pub mod opencode;
mod opencode_client;

pub trait Harness: Sized {
    fn new() -> anyhow::Result<Self>;
    fn cleanup(&self);

    async fn create_session(&self, session: Session, directory: Option<&str>)
    -> anyhow::Result<()>;
}

use crate::backend;
use crate::backend::opencode_client::OpencodeApiClient;
use crate::backend::opencode_client::OpencodeCreateSessionRequest;

pub trait Harness: Sized {
    fn new() -> anyhow::Result<Self>;
    fn cleanup(&self);

    async fn create_session(&self, session: backend::Session) -> anyhow::Result<()>;
}

#[derive(Clone)]
pub struct OpencodeHarness {
    opencode_client: OpencodeApiClient,
}

impl Harness for OpencodeHarness {
    fn new() -> anyhow::Result<Self> {
        let port = 6767;
        let opencode_client = OpencodeApiClient::new(port);

        Ok(Self { opencode_client })
    }

    fn cleanup(&self) {}

    async fn create_session(&self, session: backend::Session) -> anyhow::Result<()> {
        let request = OpencodeCreateSessionRequest {
            parent_id: None,
            title: Some(session.name),
            permission: None,
        };

        self.opencode_client
            .create_session(Some(&request), None)
            .await?;

        Ok(())
    }
}

impl Drop for OpencodeHarness {
    fn drop(&mut self) {
        self.cleanup();
    }
}

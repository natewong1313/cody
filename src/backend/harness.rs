use libc::SIGTERM;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

use crate::backend::opencode_client::OpencodeApiClient;

pub struct Project {
    id: String,
    name: String,
    dir: String,
}

pub struct GetProjectRequest {
    id: Option<String>,
    name: Option<String>,
}

pub trait Harness: Sized {
    fn new() -> anyhow::Result<Self>;
    fn cleanup(&self);

    // async fn get_project(&self, req: &GetProjectRequest);
    // async fn get_projects(&self);
}

pub struct OpencodeHarness {
    proc_handle: Child,
    opencode_client: OpencodeApiClient,
}

impl Harness for OpencodeHarness {
    fn new() -> anyhow::Result<Self> {
        let port = 6767;
        let proc_handle = Command::new("opencode")
            .arg("serve")
            .arg("--port")
            .arg(port.to_string())
            .arg("--print-logs")
            .arg("--log-level")
            .arg("DEBUG")
            .process_group(0)
            .spawn()?;

        let opencode_client = OpencodeApiClient::new(port);

        Ok(Self {
            proc_handle,
            opencode_client,
        })
    }

    fn cleanup(&self) {
        let pid = self.proc_handle.id() as i32;
        unsafe {
            libc::kill(-pid, SIGTERM);
        }
    }

    // async fn get_project(&self, req: &GetProjectRequest) {
    //     todo!()
    // }
    //
    // async fn get_projects(&self) {
    //     todo!()
    // }
}

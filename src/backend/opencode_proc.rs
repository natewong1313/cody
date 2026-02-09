use libc::SIGTERM;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

pub struct OpencodeProcess {
    proc_handle: Child,
}

impl OpencodeProcess {
    pub fn start(port: u32) -> anyhow::Result<Self> {
        let proc = Command::new("opencode")
            .arg("serve")
            .arg("--port")
            .arg(port.to_string())
            .arg("--print-logs")
            .arg("--log-level")
            .arg("DEBUG")
            .process_group(0)
            .spawn()?;

        Ok(Self { proc_handle: proc })
    }

    pub fn stop(&self) {
        let pid = self.proc_handle.id() as i32;
        unsafe {
            libc::kill(-pid, SIGTERM);
        }
    }
}

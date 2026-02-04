use std::os::unix::process::CommandExt;
use std::process::{Child, Command};

pub fn spawn_opencode_server() -> anyhow::Result<Child> {
    let proc = unsafe {
        Command::new("opencode")
            .arg("serve")
            .arg("--port")
            .arg("6767")
            .arg("--print-logs")
            .arg("--log-level")
            .arg("DEBUG")
            .pre_exec(|| {
                libc::setpgid(0, 0);
                Ok(())
            })
            .spawn()?
    };

    Ok(proc)
}

pub fn kill_opencode_server(child: &Child) -> anyhow::Result<()> {
    let pid = child.id() as i32;
    unsafe {
        libc::kill(-pid, libc::SIGTERM);
    }
    Ok(())
}

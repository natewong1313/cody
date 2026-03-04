use chrono::NaiveDateTime;

pub fn fixed_datetime() -> NaiveDateTime {
    NaiveDateTime::parse_from_str("2025-01-02 03:04:05.123456", "%Y-%m-%d %H:%M:%S%.f")
        .expect("fixed datetime should parse")
}

pub fn closed_port() -> u32 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port bind should succeed");
    let port = listener
        .local_addr()
        .expect("listener local addr should exist")
        .port();
    drop(listener);
    port as u32
}

pub fn wait_for_port(port: u32) {
    let port = u16::try_from(port).expect("port must fit in u16");
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let timeout = std::time::Duration::from_secs(5);
    let retry_delay = std::time::Duration::from_millis(50);
    let connect_timeout = std::time::Duration::from_millis(100);
    let deadline = std::time::Instant::now() + timeout;

    while std::time::Instant::now() < deadline {
        if let Ok(stream) = std::net::TcpStream::connect_timeout(&addr, connect_timeout) {
            let _ = stream.shutdown(std::net::Shutdown::Both);
            return;
        }

        std::thread::sleep(retry_delay);
    }

    panic!("port {port} did not open within {timeout:?}");
}

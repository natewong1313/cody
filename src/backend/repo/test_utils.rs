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

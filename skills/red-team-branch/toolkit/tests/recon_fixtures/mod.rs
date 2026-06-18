use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use tiny_http::{Header, Response, Server};

// A normal HTTP service: 200 + Server header + <title>. Returns its port.
pub fn http_service() -> u16 {
    let server = Server::http("127.0.0.1:0").expect("bind");
    let port = server.server_addr().to_ip().expect("ip").port();
    std::thread::spawn(move || {
        for req in server.incoming_requests() {
            let mut resp = Response::from_string("<html><title>Live App</title></html>");
            resp.add_header(Header::from_bytes(&b"Server"[..], &b"nginx/1.25"[..]).unwrap());
            let _ = req.respond(resp);
        }
    });
    wait_ready(port);
    port
}

// An open TCP port that speaks no HTTP: it accepts, reads, then closes without
// ever writing a valid status line. Recon must list it open, never as http.
pub fn raw_tcp() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            if let Ok(mut sock) = conn {
                let mut buf = [0u8; 64];
                let _ = sock.read(&mut buf);
                let _ = sock.write_all(b"GARBAGE-NOT-HTTP\r\n");
            }
        }
    });
    port
}

// A live HTTP service whose body read fails: it sends a full status line +
// Server header (so send() succeeds and headers are in hand), promises a large
// body via Content-Length, sends a few bytes, then closes mid-body. The body
// read errors. Recon must still report status/server with an empty title.
pub fn truncated_body() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            if let Ok(mut sock) = conn {
                let mut buf = [0u8; 256];
                let _ = sock.read(&mut buf);
                let _ = sock.write_all(truncated_head().as_bytes());
                let _ = sock.flush();
                // drop(sock): close mid-body so the body read errors out.
            }
        }
    });
    port
}

// Status line + headers promising 100000 bytes, followed by a tiny partial body.
fn truncated_head() -> String {
    "HTTP/1.1 200 OK\r\nServer: stall/1.0\r\nContent-Length: 100000\r\n\r\n<html>x".into()
}

fn wait_ready(port: u16) {
    for _ in 0..100 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return;
        }
        std::thread::yield_now();
    }
}

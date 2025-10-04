use std::env::args_os;
use std::env::current_exe;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Result;
use std::net::TcpListener;
use std::path::Path;
use std::path::PathBuf;

use tiny_http::Header;
use tiny_http::Response;
use tiny_http::Server;
use tiny_http::StatusCode;


fn content_type(path: &Path) -> &'static str {
    let extension = match path.extension() {
        None => return "text/plain",
        Some(e) => e,
    };

    match extension.to_str().unwrap() {
        "gif" => "image/gif",
        "htm" => "text/html; charset=utf8",
        "html" => "text/html; charset=utf8",
        "jpeg" => "image/jpeg",
        "jpg" => "image/jpeg",
        "js" => "application/javascript",
        "pdf" => "application/pdf",
        "png" => "image/png",
        "txt" => "text/plain; charset=utf8",
        "wasm" => "application/wasm",
        _ => "text/plain; charset=utf8",
    }
}

fn serve(root: PathBuf) -> Result<()> {
    let host = "127.0.0.1";
    let mut port = 8080;

    let listener = loop {
        match TcpListener::bind(format!("{host}:{port}")) {
            Ok(listener) => break listener,
            Err(err) if err.kind() == ErrorKind::AddrInUse => {
                port = 0;
            },
            Err(err) => panic!("failed to bind TCP socket: {err}"),
        }
    };

    println!("Serving on {}", listener.local_addr().unwrap());
    let server = Server::from_listener(listener, None).expect("failed to create HTTP server");

    loop {
        let req = match server.recv() {
            Ok(req) => req,
            Err(err) => break Err(err),
        };

        let path = req.url().trim_start_matches('/');
        let result = if path.is_empty() {
            let response = Response::new_empty(StatusCode(308));
            let header = Header::from_bytes(b"Location", b"index.html").unwrap();
            let response = response.with_header(header);
            req.respond(response)
        } else {
            let path = Path::new(path);
            let breakout = path
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)));
            let path = root.join(path);

            if !breakout && let Ok(file) = File::open(&path) {
                let response = Response::from_file(file);
                let mime = content_type(&path);
                let header = Header::from_bytes(b"Content-Type", mime.as_bytes()).unwrap();
                let response = response.with_header(header);
                req.respond(response)
            } else {
                let response = Response::new_empty(StatusCode(404));
                req.respond(response)
            }
        };

        if let Err(err) = result {
            eprintln!("failed to send response: {err}");
        }
    }
}

fn main() -> Result<()> {
    match args_os().len() {
        2 if !args_os().any(|arg| &arg == "--help" || &arg == "-h") => {
            let mut args = args_os();
            let _prog = args.next().unwrap();
            let root = args.next().unwrap();
            serve(PathBuf::from(root))
        },
        _ => {
            print!(
                "USAGE:
  {name} [OPTIONS] <http-root-dir>

OPTIONS:
  -h, --help       Print help information
",
                name = current_exe().unwrap().display(),
            );
            Ok(())
        },
    }
}

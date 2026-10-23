//! Language Server Protocol implementation for Gala.
//!
//! The server speaks JSON-RPC 2.0 over stdio with `Content-Length` framing. It is implemented
//! directly rather than with tower-lsp, which is not available in the offline cargo cache.
//! Supported: lifecycle, full-document sync with publishDiagnostics, hover, definition,
//! completion, and the `gala/*` custom methods. See `README.md`.

pub mod analysis;
pub mod position;
pub mod server;

use serde_json::Value;
use std::io::{self, BufRead, Write};

pub use server::Server;

/// Serve the language protocol on the process's stdin and stdout until `exit`.
pub fn run_stdio() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    if let Err(e) = serve(&mut stdin.lock(), &mut stdout.lock()) {
        eprintln!("gala-lsp: {e}");
    }
}

/// Run the server over any reader and writer. Returns when the client sends `exit` or closes the
/// input.
pub fn serve<R: BufRead, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<()> {
    let mut server = Server::new();
    while let Some(body) = read_frame(reader)? {
        let outgoing = match serde_json::from_slice::<Value>(&body) {
            Ok(msg) => server.handle(&msg),
            Err(_) => vec![server::parse_error_response()],
        };
        for message in &outgoing {
            write_frame(writer, message)?;
        }
        if server.exit_requested() {
            break;
        }
    }
    Ok(())
}

/// Read one framed message body. `Ok(None)` at end of input.
pub fn read_frame<R: BufRead>(reader: &mut R) -> io::Result<Option<Vec<u8>>> {
    let mut content_length: Option<usize> = None;
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        let header = line.trim_end_matches(['\r', '\n']);
        if header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().ok();
            }
        }
    }
    let len = content_length
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length"))?;
    let mut body = vec![0; len];
    reader.read_exact(&mut body)?;
    Ok(Some(body))
}

/// Write one message with its `Content-Length` header and flush.
pub fn write_frame<W: Write>(writer: &mut W, message: &Value) -> io::Result<()> {
    let body = serde_json::to_string(message)?;
    write!(writer, "Content-Length: {}\r\n\r\n{body}", body.len())?;
    writer.flush()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::Cursor;

    fn frame(v: &Value) -> Vec<u8> {
        let mut out = Vec::new();
        write_frame(&mut out, v).expect("write to vec");
        out
    }

    #[test]
    fn framing_round_trip() {
        let msg = json!({"jsonrpc": "2.0", "id": 1, "result": null});
        let bytes = frame(&msg);
        let mut reader = Cursor::new(bytes);
        let body = read_frame(&mut reader).expect("read").expect("one frame");
        assert_eq!(serde_json::from_slice::<Value>(&body).expect("json"), msg);
        assert!(read_frame(&mut reader).expect("eof").is_none());
    }

    #[test]
    fn serve_runs_lifecycle_over_streams() {
        let mut input = Vec::new();
        for m in [
            json!({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {}}),
            json!({"jsonrpc": "2.0", "method": "initialized", "params": {}}),
            json!({"jsonrpc": "2.0", "id": 2, "method": "shutdown"}),
            json!({"jsonrpc": "2.0", "method": "exit"}),
            json!({"jsonrpc": "2.0", "id": 3, "method": "shutdown"}),
        ] {
            input.extend(frame(&m));
        }
        let mut reader = Cursor::new(input);
        let mut out = Vec::new();
        serve(&mut reader, &mut out).expect("serve");
        let mut frames = Vec::new();
        let mut out_reader = Cursor::new(out);
        while let Some(body) = read_frame(&mut out_reader).expect("frame") {
            frames.push(serde_json::from_slice::<Value>(&body).expect("json"));
        }
        assert_eq!(frames.len(), 2, "initialize and shutdown replies only: {frames:?}");
        assert_eq!(frames[0]["id"], 1);
        assert_eq!(frames[0]["result"]["capabilities"]["textDocumentSync"], 1);
        assert_eq!(frames[1]["id"], 2);
        assert_eq!(frames[1]["result"], Value::Null);
    }

    #[test]
    fn malformed_json_gets_parse_error() {
        let body = b"{not json";
        let mut input = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
        input.extend_from_slice(body);
        let mut out = Vec::new();
        serve(&mut Cursor::new(input), &mut out).expect("serve");
        assert!(String::from_utf8(out).expect("utf8").contains("-32700"));
    }
}

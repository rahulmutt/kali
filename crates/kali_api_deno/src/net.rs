//! Deterministic TCP/HTTP server surface for the Deno compatibility layer.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use kali_api_web::{Headers, Request, Response};

/// Deterministic TCP connection wrapper for the Deno compatibility surface.
#[derive(Debug)]
pub struct DenoTcpConnection {
    stream: TcpStream,
}

impl DenoTcpConnection {
    fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    /// Read the remaining bytes from the connection.
    pub fn read_to_end(&mut self) -> Result<Vec<u8>, std::io::Error> {
        let mut contents = Vec::new();
        self.stream.read_to_end(&mut contents)?;
        Ok(contents)
    }

    /// Write bytes to the connection.
    pub fn write_all(&mut self, contents: impl AsRef<[u8]>) -> Result<(), std::io::Error> {
        self.stream.write_all(contents.as_ref())
    }

    /// Flush buffered writes.
    pub fn flush(&mut self) -> Result<(), std::io::Error> {
        self.stream.flush()
    }

    /// Return the local socket address.
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.stream.local_addr()
    }

    /// Return the peer socket address.
    pub fn peer_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.stream.peer_addr()
    }

    /// Close the write half of the connection.
    pub fn shutdown_write(&self) -> Result<(), std::io::Error> {
        self.stream.shutdown(Shutdown::Write)
    }

    /// Close the connection in both directions.
    pub fn shutdown(&self) -> Result<(), std::io::Error> {
        self.stream.shutdown(Shutdown::Both)
    }
}

/// Deterministic TCP listener wrapper for the Deno compatibility surface.
#[derive(Debug)]
pub struct DenoTcpListener {
    listener: TcpListener,
}

impl DenoTcpListener {
    fn new(listener: TcpListener) -> Self {
        Self { listener }
    }

    /// Accept a single incoming connection.
    pub fn accept(&self) -> Result<(DenoTcpConnection, SocketAddr), std::io::Error> {
        let (stream, addr) = self.listener.accept()?;
        Ok((DenoTcpConnection::new(stream), addr))
    }

    /// Return the local socket address.
    pub fn local_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.listener.local_addr()
    }
}

/// Result of a Deno-style HTTP server helper.
#[derive(Debug)]
pub struct DenoHttpServer {
    local_addr: SocketAddr,
    join_handle: Option<JoinHandle<std::io::Result<()>>>,
}

impl DenoHttpServer {
    /// Return the bound address.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }

    /// Wait for the one-shot server worker to finish.
    pub fn join(mut self) -> Result<(), std::io::Error> {
        let handle = self
            .join_handle
            .take()
            .expect("DenoHttpServer join handle already consumed");
        match handle.join() {
            Ok(result) => result,
            Err(_) => Err(std::io::Error::other("Deno.serve worker panicked")),
        }
    }
}

/// Connect to one TCP peer using the deterministic Deno compatibility surface.
pub fn connect(hostname: impl AsRef<str>, port: u16) -> Result<DenoTcpConnection, std::io::Error> {
    let stream = TcpStream::connect((hostname.as_ref(), port))?;
    let _ = stream.set_nodelay(true);
    Ok(DenoTcpConnection::new(stream))
}

/// Bind one TCP listener using the deterministic Deno compatibility surface.
pub fn listen(hostname: impl AsRef<str>, port: u16) -> Result<DenoTcpListener, std::io::Error> {
    let listener = TcpListener::bind((hostname.as_ref(), port))?;
    Ok(DenoTcpListener::new(listener))
}

fn read_http_request(
    stream: &TcpStream,
    local_addr: SocketAddr,
) -> Result<Request, std::io::Error> {
    let mut reader = BufReader::new(stream.try_clone()?);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let request_line = request_line.trim_end_matches(['\r', '\n']);
    if request_line.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "missing HTTP request line",
        ));
    }

    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing HTTP method")
    })?;
    let path = request_parts.next().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "missing HTTP path")
    })?;

    let headers = Headers::new();
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;
        if bytes_read == 0 {
            break;
        }
        let line = line.trim_end_matches(['\r', '\n']);
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            let value = value.trim();
            if name.eq_ignore_ascii_case("content-length") {
                content_length = value.parse::<usize>().unwrap_or(0);
            }
            headers.append(name.trim(), value);
        }
    }

    let mut body = vec![0u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }

    let url = if path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else {
        format!("http://{}{}", local_addr, path)
    };

    Request::with_parts(url, method, headers, body)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error.to_string()))
}

fn write_http_response(stream: &mut TcpStream, response: Response) -> Result<(), std::io::Error> {
    let headers = response.headers();
    let mut entries = headers.entries();
    if !entries
        .iter()
        .any(|(name, _)| name.eq_ignore_ascii_case("content-length"))
    {
        entries.push((
            String::from("content-length"),
            response.body().len().to_string(),
        ));
    }

    write!(
        stream,
        "HTTP/1.1 {} {}\r\n",
        response.status(),
        response.status_text()
    )?;
    for (name, value) in entries {
        write!(stream, "{}: {}\r\n", name, value)?;
    }
    write!(stream, "\r\n")?;
    stream.write_all(response.body())?;
    stream.flush()?;
    Ok(())
}

/// Serve a single HTTP request on a deterministic Deno compatibility socket.
pub fn serve<F>(
    handler: F,
    hostname: impl AsRef<str>,
    port: u16,
) -> Result<DenoHttpServer, std::io::Error>
where
    F: Fn(Request) -> Response + Send + Sync + 'static,
{
    let listener = TcpListener::bind((hostname.as_ref(), port))?;
    let local_addr = listener.local_addr()?;
    let handler = Arc::new(handler);
    let join_handle = thread::spawn(move || -> Result<(), std::io::Error> {
        let (mut stream, _) = listener.accept()?;
        let request = read_http_request(&stream, local_addr)?;
        let response = handler(request);
        write_http_response(&mut stream, response)
    });

    Ok(DenoHttpServer {
        local_addr,
        join_handle: Some(join_handle),
    })
}

#[cfg(test)]
#[path = "net_tests.rs"]
mod net_tests;

//! Getting in: a connect token from the login, over HTTPS (`MULTIPLAYER.md`,
//! phase 2).
//!
//! This device has a secret of its own, sixteen random bytes made the first
//! time it ran (`net`). It sends that to the login, which answers with a
//! connect token for the game server: netcode's, naming this device's account
//! and signed with a key only the login and the game server have. The game
//! server lets nobody in without one.
//!
//! The login is on the game server's own machine, TCP 443 at the same
//! address, behind a certificate of its own rather than one a domain would
//! need. This trusts that certificate and no other ([`PINNED`]): not any
//! authority's, so nobody can stand in the middle with one they had issued.
//! The handshake is checked as any is: the server has to hold the key that
//! goes with the certificate.
//!
//! Both calls block, for up to a few seconds: `net` makes them on a thread of
//! their own.

use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

use bevy_replicon_renet::netcode::ConnectToken;
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, ClientConnection, DigitallySignedStruct, SignatureScheme, StreamOwned};

/// The SHA-256 of the login's certificate, made on the server on 2026-10-08
/// for its two addresses and good for ten years: the one certificate this app
/// trusts. A new one, or a move to a domain, means a new app.
const PINNED: [u8; 32] = [
    0x40, 0xf6, 0xb5, 0x0c, 0x14, 0x53, 0x46, 0x32, 0x80, 0x27, 0xfe, 0x3f, 0x3c, 0x6e, 0x92, 0x0e,
    0x54, 0x35, 0x0a, 0x85, 0xea, 0x9e, 0x87, 0xca, 0xc9, 0x3b, 0x50, 0x8e, 0x80, 0xa2, 0x84, 0x8c,
];

/// The login's port, beside the game's UDP one.
const LOGIN_PORT: u16 = 443;

/// How long to wait for the login, in seconds: to reach it, and to hear back.
const CONNECT_SECS: u64 = 4;
const ANSWER_SECS: u64 = 6;

/// A token for the game server at `server`, for the device whose secret is
/// `device`.
pub(crate) fn token(server: SocketAddr, device: [u8; 16]) -> Result<ConnectToken, String> {
    let body = format!(r#"{{"device":"{}","to":"{server}"}}"#, hex(&device));
    let (status, answer) = post(server.ip(), "/login", &body)?;
    if status != 200 {
        return Err(format!("the login answered {status}"));
    }
    ConnectToken::read(&mut answer.as_slice()).map_err(|error| format!("not a token: {error}"))
}

/// Deletes the account of the device whose secret is `device`, by the login
/// on the server at `server`.
pub(crate) fn delete(server: SocketAddr, device: [u8; 16]) -> Result<(), String> {
    let body = format!(r#"{{"device":"{}"}}"#, hex(&device));
    match post(server.ip(), "/delete", &body)? {
        (204 | 200, _) => Ok(()),
        (status, _) => Err(format!("the login answered {status}")),
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Posts `body`, JSON, to `path` on the login at `ip`, and returns the answer:
/// its status and what it said.
fn post(ip: IpAddr, path: &str, body: &str) -> Result<(u16, Vec<u8>), String> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let config = ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .map_err(|error| error.to_string())?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(Pinned(provider)))
        .with_no_client_auth();
    let address = SocketAddr::new(ip, LOGIN_PORT);
    let tcp = TcpStream::connect_timeout(&address, Duration::from_secs(CONNECT_SECS))
        .map_err(|error| format!("could not reach the login at {address}: {error}"))?;
    tcp.set_read_timeout(Some(Duration::from_secs(ANSWER_SECS)))
        .and_then(|()| tcp.set_write_timeout(Some(Duration::from_secs(ANSWER_SECS))))
        .map_err(|error| error.to_string())?;
    let connection = ClientConnection::new(Arc::new(config), ServerName::IpAddress(ip.into()))
        .map_err(|error| error.to_string())?;
    let mut tls = StreamOwned::new(connection, tcp);
    let host = match ip {
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) => format!("[{ip}]"),
    };
    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\n\
         Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    tls.write_all(request.as_bytes())
        .map_err(|error| format!("the login: {error}"))?;
    let mut answer = Vec::new();
    // A server that closes without saying so first still said all it had to.
    if let Err(error) = tls.read_to_end(&mut answer)
        && (error.kind() != std::io::ErrorKind::UnexpectedEof || answer.is_empty())
    {
        return Err(format!("the login: {error}"));
    }
    parse(&answer)
}

/// An HTTP/1.1 answer: its status, and its body, put back together if it came
/// in chunks.
fn parse(answer: &[u8]) -> Result<(u16, Vec<u8>), String> {
    let split = answer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or("half an answer")?;
    let head = String::from_utf8_lossy(&answer[..split]);
    let mut body = answer[split + 4..].to_vec();
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|status| status.parse().ok())
        .ok_or("no status")?;
    let chunked = head.lines().any(|line| {
        let line = line.to_ascii_lowercase();
        line.starts_with("transfer-encoding:") && line.contains("chunked")
    });
    if chunked {
        body = unchunk(&body).ok_or("broken chunks")?;
    }
    Ok((status, body))
}

fn unchunk(mut chunks: &[u8]) -> Option<Vec<u8>> {
    let mut body = Vec::new();
    loop {
        let end = chunks.windows(2).position(|window| window == b"\r\n")?;
        let size = std::str::from_utf8(&chunks[..end]).ok()?;
        let size = usize::from_str_radix(size.split(';').next()?.trim(), 16).ok()?;
        chunks = &chunks[end + 2..];
        if size == 0 {
            return Some(body);
        }
        body.extend_from_slice(chunks.get(..size)?);
        chunks = chunks.get(size + 2..)?;
    }
}

/// Trusts the login's own certificate, [`PINNED`], and nothing else.
#[derive(Debug)]
struct Pinned(Arc<CryptoProvider>);

impl ServerCertVerifier for Pinned {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let digest = ring::digest::digest(&ring::digest::SHA256, end_entity.as_ref());
        if digest.as_ref() == PINNED {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer,
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        signed: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls12_signature(message, cert, signed, &self.0.signature_verification_algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        signed: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        verify_tls13_signature(message, cert, signed, &self.0.signature_verification_algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_an_answer() {
        let (status, body) = parse(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc").unwrap();
        assert_eq!((status, body.as_slice()), (200, &b"abc"[..]));
        let (status, body) = parse(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n3\r\nabc\r\n2\r\nde\r\n0\r\n\r\n",
        )
        .unwrap();
        assert_eq!((status, body.as_slice()), (200, &b"abcde"[..]));
        assert_eq!(parse(b"HTTP/1.1 429 Too Many Requests\r\n\r\n").unwrap().0, 429);
    }
}

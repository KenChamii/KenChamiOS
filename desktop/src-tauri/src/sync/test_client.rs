// a TLS client that trusts exactly one certificate hash, the way the phone does
use super::codes::{secrets_equal, sha256_hex};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, aws_lc_rs};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{ClientConfig, DigitallySignedStruct, Error, SignatureScheme};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

#[derive(Debug)]
struct PinVerifier {
    pin: String,
    provider: Arc<CryptoProvider>,
}

impl ServerCertVerifier for PinVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        if secrets_equal(&sha256_hex(end_entity), &self.pin) {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(Error::General("certificate does not match the pin".into()))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.provider.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.provider
            .signature_verification_algorithms
            .supported_schemes()
    }
}

pub struct Reply {
    pub status: u16,
    pub body: String,
}

pub struct TestClient {
    addr: SocketAddr,
    connector: TlsConnector,
}

impl TestClient {
    pub fn pinned(addr: SocketAddr, pin: &str) -> Self {
        let provider = Arc::new(aws_lc_rs::default_provider());
        let config = ClientConfig::builder_with_provider(provider.clone())
            .with_safe_default_protocol_versions()
            .expect("protocol versions")
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(PinVerifier {
                pin: pin.to_string(),
                provider,
            }))
            .with_no_client_auth();
        Self {
            addr,
            connector: TlsConnector::from(Arc::new(config)),
        }
    }

    /// One request on a fresh connection; `Err` means the TLS handshake or the transport failed.
    pub async fn request(
        &self,
        method: &str,
        path: &str,
        bearer: Option<&str>,
        body: &str,
    ) -> Result<Reply, String> {
        let tcp = TcpStream::connect(self.addr)
            .await
            .map_err(|e| e.to_string())?;
        let name = ServerName::try_from("kenchamios.local").map_err(|e| e.to_string())?;
        let mut tls = self
            .connector
            .connect(name, tcp)
            .await
            .map_err(|e| e.to_string())?;
        let auth = bearer.map_or(String::new(), |b| format!("Authorization: Bearer {b}\r\n"));
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: kenchamios.local\r\n{auth}Content-Type: application/json\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        tls.write_all(request.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        let mut raw = Vec::new();
        // the server closes without a TLS close_notify, so a read error after data is fine
        let _ = tls.read_to_end(&mut raw).await;
        let text = String::from_utf8_lossy(&raw).to_string();
        let status = text
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("no http answer: {text}"))?;
        let body = text
            .split_once("\r\n\r\n")
            .map_or("", |(_, b)| b)
            .to_string();
        Ok(Reply { status, body })
    }
}

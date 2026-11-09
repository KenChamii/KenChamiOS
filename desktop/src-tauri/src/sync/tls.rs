use super::codes::sha256_hex;
use crate::error::{AppError, AppResult};
use crate::media::secrets::SecretStore;
use rcgen::{CertificateParams, KeyPair, date_time_ymd};
use rustls::ServerConfig;
use rustls::crypto::aws_lc_rs;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::sync::Arc;

const CERT_SECRET: &str = "sync-tls-certificate";
const KEY_SECRET: &str = "sync-tls-private-key";

fn tls_error(error: impl std::fmt::Display) -> AppError {
    AppError::Sync(format!("tls setup failed: {error}"))
}

/// The server's self-signed certificate and key. The phone pins the certificate hash.
pub struct Identity {
    cert_pem: String,
    key_pem: String,
    cert_der: Vec<u8>,
}

impl Identity {
    /// Loads the identity from the credential store, creating it on first use.
    pub fn load_or_create(store: &dyn SecretStore) -> AppResult<Self> {
        if let (Some(cert), Some(key)) = (store.get(CERT_SECRET)?, store.get(KEY_SECRET)?) {
            return Self::from_pem(cert, key);
        }
        let key_pair = KeyPair::generate().map_err(tls_error)?;
        let mut params =
            CertificateParams::new(vec!["kenchamios.local".to_string()]).map_err(tls_error)?;
        params.not_before = date_time_ymd(2024, 1, 1);
        params.not_after = date_time_ymd(2056, 1, 1);
        let cert = params.self_signed(&key_pair).map_err(tls_error)?;
        store.set(CERT_SECRET, &cert.pem())?;
        store.set(KEY_SECRET, &key_pair.serialize_pem())?;
        Self::from_pem(cert.pem(), key_pair.serialize_pem())
    }

    fn from_pem(cert_pem: String, key_pem: String) -> AppResult<Self> {
        let cert_der = CertificateDer::from_pem_slice(cert_pem.as_bytes())
            .map_err(tls_error)?
            .to_vec();
        Ok(Self {
            cert_pem,
            key_pem,
            cert_der,
        })
    }

    /// SHA-256 of the certificate, 64 lowercase hex characters.
    pub fn fingerprint(&self) -> String {
        sha256_hex(&self.cert_der)
    }

    pub fn server_config(&self) -> AppResult<Arc<ServerConfig>> {
        let cert = CertificateDer::from_pem_slice(self.cert_pem.as_bytes()).map_err(tls_error)?;
        let key = PrivateKeyDer::from_pem_slice(self.key_pem.as_bytes()).map_err(tls_error)?;
        let mut config =
            ServerConfig::builder_with_provider(Arc::new(aws_lc_rs::default_provider()))
                .with_safe_default_protocol_versions()
                .map_err(tls_error)?
                .with_no_client_auth()
                .with_single_cert(vec![cert], key)
                .map_err(tls_error)?;
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        Ok(Arc::new(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::test_support::SharedSecrets;

    #[test]
    fn the_identity_is_created_once_and_reloaded() {
        let store = SharedSecrets::default();

        let first = Identity::load_or_create(&store).expect("create");
        let second = Identity::load_or_create(&store).expect("reload");

        assert_eq!(first.fingerprint(), second.fingerprint());
        assert_eq!(first.fingerprint().len(), 64);
        assert!(first.server_config().is_ok());
    }

    #[test]
    fn two_stores_get_different_fingerprints() {
        let a = Identity::load_or_create(&SharedSecrets::default()).expect("a");
        let b = Identity::load_or_create(&SharedSecrets::default()).expect("b");
        assert_ne!(a.fingerprint(), b.fingerprint());
    }
}

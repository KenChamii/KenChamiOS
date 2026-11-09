use crate::error::AppResult;

const SERVICE: &str = "kenchamios";

/// Where API keys are kept; never the database, so a copy of it leaks nothing.
pub trait SecretStore {
    fn get(&self, name: &str) -> AppResult<Option<String>>;
    fn set(&self, name: &str, value: &str) -> AppResult<()>;
    fn clear(&self, name: &str) -> AppResult<()>;
}

/// The operating system credential store (Windows Credential Manager on this PC).
pub struct KeyringStore;

impl SecretStore for KeyringStore {
    fn get(&self, name: &str) -> AppResult<Option<String>> {
        match keyring::Entry::new(SERVICE, name)?.get_password() {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn set(&self, name: &str, value: &str) -> AppResult<()> {
        Ok(keyring::Entry::new(SERVICE, name)?.set_password(value)?)
    }

    fn clear(&self, name: &str) -> AppResult<()> {
        match keyring::Entry::new(SERVICE, name)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

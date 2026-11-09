use serde_json::Value;

/// A shared fixture from `shared/fixtures/sync`; the Android tests read the same files.
pub fn fixture(name: &str) -> Value {
    let path = format!(
        "{}/../../shared/fixtures/sync/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = std::fs::read_to_string(path).expect("read fixture");
    serde_json::from_str(&text).expect("parse fixture")
}

pub fn rejected_cases() -> Vec<Value> {
    fixture("rejected").as_array().expect("cases").clone()
}

/// A credential store that lives in memory and can be shared with the server task.
#[derive(Default)]
pub struct SharedSecrets(std::sync::Mutex<std::collections::HashMap<String, String>>);

impl crate::media::secrets::SecretStore for SharedSecrets {
    fn get(&self, name: &str) -> crate::error::AppResult<Option<String>> {
        Ok(self.0.lock().map_err(poisoned)?.get(name).cloned())
    }

    fn set(&self, name: &str, value: &str) -> crate::error::AppResult<()> {
        self.0
            .lock()
            .map_err(poisoned)?
            .insert(name.into(), value.into());
        Ok(())
    }

    fn clear(&self, name: &str) -> crate::error::AppResult<()> {
        self.0.lock().map_err(poisoned)?.remove(name);
        Ok(())
    }
}

fn poisoned<T>(_: std::sync::PoisonError<T>) -> crate::error::AppError {
    crate::error::AppError::Sync("test store poisoned".to_string())
}

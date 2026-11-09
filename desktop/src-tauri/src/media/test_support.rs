use super::http::{HttpClient, HttpRequest, HttpResponse};
use super::secrets::SecretStore;
use crate::error::{AppError, AppResult};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};

/// Plays back recorded answers in order and remembers what was asked.
#[derive(Default)]
pub struct FakeHttp {
    answers: RefCell<VecDeque<AppResult<HttpResponse>>>,
    pub requests: RefCell<Vec<HttpRequest>>,
}

impl FakeHttp {
    pub fn answering(status: u16, body: &str) -> Self {
        let fake = Self::default();
        fake.push(status, body);
        fake
    }

    pub fn push(&self, status: u16, body: &str) {
        self.push_bytes(status, body.as_bytes().to_vec());
    }

    pub fn push_bytes(&self, status: u16, body: Vec<u8>) {
        self.answers.borrow_mut().push_back(Ok(HttpResponse {
            status,
            retry_after_s: None,
            body,
        }));
    }

    pub fn push_failure(&self) {
        self.answers.borrow_mut().push_back(Err(AppError::Provider(
            "could not reach the service".into(),
        )));
    }
}

impl HttpClient for FakeHttp {
    async fn send(&self, request: HttpRequest) -> AppResult<HttpResponse> {
        self.requests.borrow_mut().push(request);
        self.answers
            .borrow_mut()
            .pop_front()
            .unwrap_or_else(|| Err(AppError::Provider("no recorded answer".into())))
    }
}

#[derive(Default)]
pub struct MemorySecrets(RefCell<HashMap<String, String>>);

impl SecretStore for MemorySecrets {
    fn get(&self, name: &str) -> AppResult<Option<String>> {
        Ok(self.0.borrow().get(name).cloned())
    }

    fn set(&self, name: &str, value: &str) -> AppResult<()> {
        self.0.borrow_mut().insert(name.into(), value.into());
        Ok(())
    }

    fn clear(&self, name: &str) -> AppResult<()> {
        self.0.borrow_mut().remove(name);
        Ok(())
    }
}

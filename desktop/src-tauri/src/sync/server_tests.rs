// the real server on 127.0.0.1 with a real TLS client that pins the certificate hash
use super::service::SyncService;
use super::test_client::{Reply, TestClient};
use super::test_support::{SharedSecrets, fixture};
use crate::db::{Db, open, paired_devices, projects};
use crate::sync::hlc::Hlc;
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

struct Harness {
    service: SyncService,
    client: TestClient,
    db: Arc<Db>,
    dir: PathBuf,
    addr: SocketAddr,
    fingerprint: String,
}

impl Harness {
    async fn start() -> Self {
        let dir = std::env::temp_dir().join(format!("kenchamios-sync-{}", crate::db::new_id()));
        let pool = open(&dir.join("live.db")).await.expect("open db");
        let db = Arc::new(Db::new(pool, Hlc::new(), "desktop-1".to_string()));
        let service = SyncService::new(db.clone(), Arc::new(SharedSecrets::default()), dir.clone())
            .expect("service");
        let addr = service
            .start_on("127.0.0.1:0".parse().expect("addr"), false)
            .await
            .expect("start");
        let fingerprint = service.view().await.expect("view").fingerprint.expect("fp");
        let client = TestClient::pinned(addr, &fingerprint);
        Self {
            service,
            client,
            db,
            dir,
            addr,
            fingerprint,
        }
    }

    async fn pair(&self, device_id: &str) -> String {
        let info = self.service.start_pairing().await.expect("pairing");
        let reply = self.pair_with(device_id, &info.payload.code).await;
        assert_eq!(reply.status, 200, "{}", reply.body);
        let body: Value = serde_json::from_str(&reply.body).expect("json");
        format!("{device_id}.{}", body["token"].as_str().expect("token"))
    }

    async fn pair_with(&self, device_id: &str, code: &str) -> Reply {
        let body = json!({ "code": code, "device_id": device_id, "device_name": "Test phone" });
        self.client
            .request("POST", "/pair", None, &body.to_string())
            .await
            .expect("pair request")
    }

    async fn push(&self, bearer: &str, body: &Value) -> Reply {
        self.client
            .request("POST", "/sync/push", Some(bearer), &body.to_string())
            .await
            .expect("push request")
    }

    async fn pull(&self, bearer: &str, query: &str) -> Reply {
        self.client
            .request("GET", &format!("/sync/pull{query}"), Some(bearer), "")
            .await
            .expect("pull request")
    }

    async fn finish(self) {
        self.service.stop().await;
        self.db.pool.close().await;
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn parse(reply: &Reply) -> Value {
    serde_json::from_str(&reply.body).unwrap_or_else(|_| panic!("not json: {}", reply.body))
}

#[tokio::test]
async fn the_server_is_off_until_it_is_started() {
    let dir = std::env::temp_dir().join(format!("kenchamios-sync-{}", crate::db::new_id()));
    let pool = open(&dir.join("live.db")).await.expect("open db");
    let db = Arc::new(Db::new(pool, Hlc::new(), "desktop-1".to_string()));
    let service = SyncService::new(db.clone(), Arc::new(SharedSecrets::default()), dir.clone())
        .expect("service");

    let view = service.view().await.expect("view");

    assert!(!view.enabled && !view.running && view.address.is_none());
    assert!(
        service.start_pairing().await.is_err(),
        "no pairing while off"
    );
    db.pool.close().await;
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn pairing_then_pushing_and_pulling_works() {
    let h = Harness::start().await;
    let bearer = h.pair("dev-phone").await;
    projects::create_for_test(&h.db, "Desktop project").await;

    let push = h.push(&bearer, &fixture("push-valid")).await;
    assert_eq!(push.status, 200, "{}", push.body);
    assert_eq!(parse(&push)["applied"], 11);

    let pull = h.pull(&bearer, "?since=0").await;
    assert_eq!(pull.status, 200);
    let devices: Vec<String> = parse(&pull)["changes"]
        .as_array()
        .expect("changes")
        .iter()
        .map(|c| c["row"]["device_id"].as_str().expect("device").to_string())
        .collect();
    assert!(
        devices.iter().all(|d| d == "desktop-1"),
        "the phone's own rows are not echoed"
    );
    assert!(!devices.is_empty());
    h.finish().await;
}

#[tokio::test]
async fn a_push_makes_a_backup_first() {
    let h = Harness::start().await;
    let bearer = h.pair("phone-1").await;

    assert_eq!(h.push(&bearer, &fixture("push-valid")).await.status, 200);

    let backups = std::fs::read_dir(h.dir.join("backups"))
        .expect("backups")
        .count();
    assert_eq!(backups, 1);
    h.finish().await;
}

#[tokio::test]
async fn pull_pages_and_records_what_the_device_has() {
    let h = Harness::start().await;
    let bearer = h.pair("phone-1").await;
    for name in ["A", "B", "C"] {
        projects::create_for_test(&h.db, name).await;
    }

    let first = parse(&h.pull(&bearer, "?since=0&limit=2").await);
    assert_eq!(first["has_more"], true);
    let next = first["next"].as_i64().expect("next");
    let second = parse(&h.pull(&bearer, &format!("?since={next}&limit=2")).await);
    assert!(!second["changes"].as_array().expect("changes").is_empty());

    let device = paired_devices::get(&h.db, "phone-1")
        .await
        .expect("get")
        .expect("device");
    assert_eq!(device.pulled_seq, next);
    assert!(device.last_sync_at.is_some());
    h.finish().await;
}

#[tokio::test]
async fn requests_without_a_valid_credential_are_refused() {
    let h = Harness::start().await;
    let bearer = h.pair("phone-1").await;
    let (id, secret) = bearer.split_once('.').expect("bearer");

    for bad in [
        None,
        Some("phone-1.wrong".to_string()),
        Some("unknown.".to_string() + secret),
        Some(format!("{id}.{}", "0".repeat(64))),
        Some("no-dot-at-all".to_string()),
    ] {
        let reply = h
            .client
            .request("GET", "/sync/pull", bad.as_deref(), "")
            .await
            .expect("request");
        assert_eq!(reply.status, 401, "{bad:?}");
    }
    assert_eq!(h.pull(&bearer, "").await.status, 200);
    h.finish().await;
}

#[tokio::test]
async fn revoking_a_device_takes_effect_at_once() {
    let h = Harness::start().await;
    let bearer = h.pair("phone-1").await;
    assert_eq!(h.pull(&bearer, "").await.status, 200);

    h.service.revoke("phone-1").await.expect("revoke");

    assert_eq!(h.pull(&bearer, "").await.status, 401);
    assert_eq!(h.push(&bearer, &fixture("push-valid")).await.status, 401);
    h.finish().await;
}

#[tokio::test]
async fn repeated_failures_lock_the_address_out() {
    let h = Harness::start().await;
    let bearer = h.pair("phone-1").await;

    for _ in 0..10 {
        h.pull("phone-1.wrong", "").await;
    }

    assert_eq!(
        h.pull(&bearer, "").await.status,
        429,
        "even the right credential waits"
    );
    h.finish().await;
}

#[tokio::test]
async fn the_pairing_code_works_once_and_not_after_five_mistakes() {
    let h = Harness::start().await;
    let info = h.service.start_pairing().await.expect("pairing");
    let code = info.payload.code;

    assert_eq!(h.pair_with("phone-1", &code).await.status, 200);
    assert_eq!(
        h.pair_with("phone-2", &code).await.status,
        404,
        "single use"
    );

    let info = h.service.start_pairing().await.expect("pairing");
    for _ in 0..5 {
        assert_eq!(h.pair_with("phone-3", "AAAAAAAAAA").await.status, 403);
    }
    assert_eq!(
        h.pair_with("phone-3", &info.payload.code).await.status,
        404,
        "locked"
    );
    h.finish().await;
}

#[tokio::test]
async fn nobody_can_pair_while_the_window_is_closed() {
    let h = Harness::start().await;
    assert_eq!(h.pair_with("phone-1", "AAAAAAAAAA").await.status, 404);

    let info = h.service.start_pairing().await.expect("pairing");
    h.service.cancel_pairing();
    assert_eq!(h.pair_with("phone-1", &info.payload.code).await.status, 404);
    h.finish().await;
}

#[tokio::test]
async fn a_client_with_the_wrong_pin_cannot_connect() {
    let h = Harness::start().await;
    let rogue = TestClient::pinned(h.addr, &"0".repeat(64));

    let result = rogue.request("GET", "/sync/pull", None, "").await;

    assert!(result.is_err(), "the handshake must fail");
    assert_eq!(h.fingerprint.len(), 64);
    h.finish().await;
}

#[tokio::test]
async fn hostile_payloads_are_rejected_and_change_nothing() {
    let h = Harness::start().await;
    let bearer = h.pair("phone-1").await;

    for case in fixture("rejected").as_array().expect("cases") {
        let reply = h
            .push(&bearer, &json!({ "changes": case["changes"] }))
            .await;
        assert_eq!(reply.status, 422, "{}: {}", case["name"], reply.body);
    }
    let unknown_field = h.push(&bearer, &json!({ "changes": [], "extra": 1 })).await;
    assert!(unknown_field.status == 422 || unknown_field.status == 400);
    let broken = h
        .client
        .request("POST", "/sync/push", Some(&bearer), "{not json")
        .await
        .expect("request");
    assert!(broken.status == 400 || broken.status == 422);

    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks")
        .fetch_one(&h.db.pool)
        .await
        .expect("count");
    assert_eq!(rows, 0);
    h.finish().await;
}

#[tokio::test]
async fn oversized_bodies_are_refused() {
    let h = Harness::start().await;
    let bearer = h.pair("phone-1").await;
    let huge = format!(
        "{{\"changes\":[],\"pad\":\"{}\"}}",
        "a".repeat(5 * 1024 * 1024)
    );

    let result = h
        .client
        .request("POST", "/sync/push", Some(&bearer), &huge)
        .await;

    // the server may cut the connection before the client finishes sending
    if let Ok(reply) = result {
        assert_eq!(reply.status, 413);
    }
    h.finish().await;
}

#[tokio::test]
async fn too_many_changes_in_one_push_are_refused() {
    let h = Harness::start().await;
    let bearer = h.pair("phone-1").await;
    let one = fixture("push-valid")["changes"][0].clone();
    let batch = json!({ "changes": vec![one; 501] });

    let reply = h.push(&bearer, &batch).await;

    assert_eq!(reply.status, 422);
    h.finish().await;
}

#[tokio::test]
async fn stopping_the_server_closes_the_port() {
    let h = Harness::start().await;
    h.service.stop().await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let result = h.client.request("GET", "/sync/pull", None, "").await;

    assert!(result.is_err());
    h.finish().await;
}

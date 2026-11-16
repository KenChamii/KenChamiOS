use super::tools;
use crate::db::{Db, mcp_calls};
use serde_json::{Value, json};

/// Protocol versions this server speaks; a client asking for another gets the newest.
const VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];
const SERVER_NAME: &str = "kenchamios";

const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

fn error(id: &Value, code: i64, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

fn result(id: &Value, value: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": value })
}

fn text_result(id: &Value, text: &str, is_error: bool) -> Value {
    result(
        id,
        json!({ "content": [{ "type": "text", "text": text }], "isError": is_error }),
    )
}

/// Answers one JSON-RPC message, or `None` for a notification, which gets no answer.
pub async fn handle(db: &Db, request: &Value, now: i64) -> Option<Value> {
    let Some(method) = request["method"].as_str() else {
        return Some(error(
            &request["id"],
            INVALID_REQUEST,
            "not a JSON-RPC request",
        ));
    };
    // a message without an id is a notification and gets no answer
    request.get("id")?;
    let id = &request["id"];
    Some(match method {
        "initialize" => {
            let asked = request["params"]["protocolVersion"].as_str().unwrap_or("");
            let version = VERSIONS
                .iter()
                .find(|v| **v == asked)
                .unwrap_or(&VERSIONS[0]);
            result(
                id,
                json!({
                    "protocolVersion": version,
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": SERVER_NAME, "version": env!("CARGO_PKG_VERSION") }
                }),
            )
        }
        "ping" => result(id, json!({})),
        "tools/list" => result(id, json!({ "tools": tools::definitions() })),
        "tools/call" => call_tool(db, id, &request["params"], now).await,
        _ => error(id, METHOD_NOT_FOUND, "method not found"),
    })
}

async fn call_tool(db: &Db, id: &Value, params: &Value, now: i64) -> Value {
    let Some(name) = params["name"].as_str() else {
        return error(id, INVALID_PARAMS, "a tool name is required");
    };
    let known = tools::definitions()
        .as_array()
        .is_some_and(|all| all.iter().any(|t| t["name"] == name));
    if !known {
        return error(id, INVALID_PARAMS, "unknown tool");
    }
    let args = if params["arguments"].is_null() {
        json!({})
    } else {
        params["arguments"].clone()
    };
    let outcome = if name == "undo_last" {
        undo_last(db, now).await
    } else {
        tools::call(db, name, &args, now).await
    };
    let args_text = args.to_string();
    match outcome {
        Ok(done) => {
            let undo = done.undo.map(|u| u.to_string());
            let _ =
                mcp_calls::record(db, name, &args_text, true, &done.text, undo.as_deref()).await;
            text_result(id, &done.text, false)
        }
        Err(failure) => {
            let message = failure.to_string();
            let _ = mcp_calls::record(db, name, &args_text, false, &message, None).await;
            text_result(id, &message, true)
        }
    }
}

async fn undo_last(db: &Db, now: i64) -> crate::error::AppResult<tools::Outcome> {
    let Some(call) = mcp_calls::last_undoable(db).await? else {
        return Ok(tools::Outcome {
            text: "There is nothing to undo.".into(),
            undo: None,
        });
    };
    let text = undo_call(db, &call, now).await?;
    Ok(tools::Outcome { text, undo: None })
}

/// Reverses one logged call and marks it undone; also used by the settings page.
pub async fn undo_call(
    db: &Db,
    call: &mcp_calls::McpCall,
    now: i64,
) -> crate::error::AppResult<String> {
    let undo: Value = serde_json::from_str(call.undo_json.as_deref().unwrap_or("null"))?;
    let text = tools::undo(db, &undo, now).await?;
    mcp_calls::mark_undone(db, call.id).await?;
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::projects::create_for_test;

    async fn ask(db: &Db, request: Value) -> Value {
        handle(db, &request, 20_777 * 86_400_000)
            .await
            .expect("answer")
    }

    fn call(name: &str, arguments: Value) -> Value {
        json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/call",
                "params": { "name": name, "arguments": arguments } })
    }

    fn text(answer: &Value) -> String {
        answer["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or("")
            .to_string()
    }

    #[tokio::test]
    async fn initialize_names_the_server_and_agrees_on_a_version() {
        let db = Db::in_memory().await;
        let answer = ask(
            &db,
            json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize",
                    "params": { "protocolVersion": "2025-03-26" } }),
        )
        .await;
        assert_eq!(answer["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(answer["result"]["serverInfo"]["name"], "kenchamios");
        assert!(answer["result"]["capabilities"]["tools"].is_object());

        let other = ask(
            &db,
            json!({ "jsonrpc": "2.0", "id": 2, "method": "initialize",
                    "params": { "protocolVersion": "1999-01-01" } }),
        )
        .await;
        assert_eq!(other["result"]["protocolVersion"], "2025-06-18");
    }

    #[tokio::test]
    async fn notifications_get_no_answer() {
        let db = Db::in_memory().await;
        let none = handle(
            &db,
            &json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }),
            0,
        )
        .await;
        assert!(none.is_none());
    }

    #[tokio::test]
    async fn tools_are_listed_with_schemas() {
        let db = Db::in_memory().await;
        let answer = ask(
            &db,
            json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }),
        )
        .await;
        let names: Vec<&str> = answer["result"]["tools"]
            .as_array()
            .expect("tools")
            .iter()
            .filter_map(|t| t["name"].as_str())
            .collect();
        for wanted in [
            "read_today",
            "add_task",
            "add_inbox_item",
            "log_time",
            "move_job_stage",
            "undo_last",
        ] {
            assert!(names.contains(&wanted), "{wanted}");
        }
        assert_eq!(
            answer["result"]["tools"][0]["inputSchema"]["type"],
            "object"
        );
    }

    #[tokio::test]
    async fn unknown_methods_and_tools_are_errors() {
        let db = Db::in_memory().await;
        let method = ask(&db, json!({ "jsonrpc": "2.0", "id": 1, "method": "nope" })).await;
        assert_eq!(method["error"]["code"], METHOD_NOT_FOUND);
        let tool = ask(&db, call("nope", json!({}))).await;
        assert_eq!(tool["error"]["code"], INVALID_PARAMS);
    }

    #[tokio::test]
    async fn a_task_is_added_logged_and_undone() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo App").await;

        let added = ask(
            &db,
            call(
                "add_task",
                json!({ "project": "demo", "title": "Write docs" }),
            ),
        )
        .await;
        assert_eq!(added["result"]["isError"], false, "{added}");
        let open = crate::db::tasks::list(&db, &project.id)
            .await
            .expect("tasks");
        assert_eq!(open.len(), 1);

        let calls = mcp_calls::list(&db, 10).await.expect("calls");
        assert_eq!(calls[0].call.tool, "add_task");
        assert!(calls[0].can_undo);

        let undone = ask(&db, call("undo_last", json!({}))).await;
        assert!(text(&undone).contains("trash"));
        assert!(
            crate::db::tasks::list(&db, &project.id)
                .await
                .expect("tasks")
                .is_empty()
        );
        let again = ask(&db, call("undo_last", json!({}))).await;
        assert_eq!(text(&again), "There is nothing to undo.");
    }

    #[tokio::test]
    async fn a_failing_tool_is_reported_in_the_result_and_logged() {
        let db = Db::in_memory().await;
        let answer = ask(
            &db,
            call("add_task", json!({ "project": "ghost", "title": "x" })),
        )
        .await;
        assert_eq!(answer["result"]["isError"], true);
        assert!(text(&answer).contains("no project matches"));
        assert!(!mcp_calls::list(&db, 1).await.expect("calls")[0].call.ok);
    }

    #[tokio::test]
    async fn the_inbox_time_log_and_stage_moves_work_and_undo() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        ask(&db, call("add_inbox_item", json!({ "text": "call mum" }))).await;
        ask(
            &db,
            call(
                "log_time",
                json!({ "project": "Demo", "minutes": 25, "note": "sketch" }),
            ),
        )
        .await;
        ask(
            &db,
            call(
                "move_project_stage",
                json!({ "project": "Demo", "stage": "build" }),
            ),
        )
        .await;

        let inbox = crate::db::tasks::list(&db, crate::domain::project::INBOX_ID)
            .await
            .expect("inbox");
        assert_eq!(inbox[0].title, "call mum");
        let entries = crate::db::time_entries::list(&db, &project.id)
            .await
            .expect("time");
        assert_eq!(entries[0].duration_s, 25 * 60);
        let stage = crate::db::projects::get(&db, &project.id)
            .await
            .expect("project")
            .project
            .stage;
        assert_eq!(stage.as_str(), "build");

        ask(&db, call("undo_last", json!({}))).await;
        let back = crate::db::projects::get(&db, &project.id)
            .await
            .expect("project")
            .project
            .stage;
        assert_ne!(back.as_str(), "build");
    }

    #[tokio::test]
    async fn bad_arguments_are_refused_with_a_reason() {
        let db = Db::in_memory().await;
        create_for_test(&db, "Demo").await;
        let no_title = ask(&db, call("add_task", json!({ "project": "Demo" }))).await;
        assert!(text(&no_title).contains("'title' is required"));
        let minutes = ask(
            &db,
            call("log_time", json!({ "project": "Demo", "minutes": 9999 })),
        )
        .await;
        assert!(text(&minutes).contains("1 to 1440"));
        let stage = ask(
            &db,
            call(
                "move_project_stage",
                json!({ "project": "Demo", "stage": "dancing" }),
            ),
        )
        .await;
        assert_eq!(stage["result"]["isError"], true);
    }
}

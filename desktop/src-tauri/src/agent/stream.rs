use serde::Serialize;
use serde_json::Value;
use std::collections::HashSet;

const MAX_TEXT: usize = 4000;
const MAX_INPUT: usize = 500;

/// One thing worth showing from a `--output-format stream-json` line.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StreamEvent {
    Init {
        model: String,
        tools: Vec<String>,
    },
    Text {
        message_id: String,
        text: String,
    },
    ToolUse {
        message_id: String,
        name: String,
        input: String,
    },
    ToolResult {
        content: String,
        is_error: bool,
    },
    Result {
        is_error: bool,
        text: String,
        turns: u64,
        cost_usd: Option<f64>,
    },
    /// a line the app does not know, kept as it came
    Raw {
        text: String,
    },
}

/// The events of one output line; a line the app does not understand becomes `Raw`.
pub fn parse_line(line: &str) -> Vec<StreamEvent> {
    let line = line.trim();
    if line.is_empty() {
        return Vec::new();
    }
    let parsed = serde_json::from_str::<Value>(line)
        .ok()
        .and_then(|value| known(&value));
    parsed.unwrap_or_else(|| {
        vec![StreamEvent::Raw {
            text: truncate(line, MAX_TEXT),
        }]
    })
}

fn known(value: &Value) -> Option<Vec<StreamEvent>> {
    match (value["type"].as_str()?, value["subtype"].as_str()) {
        ("system", Some("init")) => Some(vec![StreamEvent::Init {
            model: text(&value["model"]),
            tools: value["tools"]
                .as_array()
                .map(|tools| tools.iter().filter_map(Value::as_str).map(String::from))
                .into_iter()
                .flatten()
                .collect(),
        }]),
        ("assistant", _) => {
            let id = text(&value["message"]["id"]);
            let events = blocks(value)?.filter_map(|block| assistant_block(&id, block));
            Some(events.collect())
        }
        ("user", _) => Some(blocks(value)?.filter_map(tool_result).collect()),
        ("result", _) => Some(vec![StreamEvent::Result {
            is_error: value["is_error"].as_bool().unwrap_or(true),
            text: truncate(&text(&value["result"]), MAX_TEXT),
            turns: value["num_turns"].as_u64().unwrap_or(0),
            cost_usd: value["total_cost_usd"].as_f64(),
        }]),
        _ => None,
    }
}

fn blocks(value: &Value) -> Option<impl Iterator<Item = &Value>> {
    Some(value["message"]["content"].as_array()?.iter())
}

// thinking and other block types are not shown
fn assistant_block(message_id: &str, block: &Value) -> Option<StreamEvent> {
    match block["type"].as_str()? {
        "text" => Some(StreamEvent::Text {
            message_id: message_id.to_string(),
            text: truncate(&text(&block["text"]), MAX_TEXT),
        }),
        "tool_use" => Some(StreamEvent::ToolUse {
            message_id: message_id.to_string(),
            name: text(&block["name"]),
            input: truncate(&block["input"].to_string(), MAX_INPUT),
        }),
        _ => None,
    }
}

fn tool_result(block: &Value) -> Option<StreamEvent> {
    if block["type"].as_str()? != "tool_result" {
        return None;
    }
    // the content is a string or a list of text blocks
    let content = match &block["content"] {
        Value::Array(parts) => parts
            .iter()
            .filter_map(|part| part["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        other => text(other),
    };
    Some(StreamEvent::ToolResult {
        content: truncate(&content, MAX_TEXT),
        is_error: block["is_error"].as_bool().unwrap_or(false),
    })
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

fn truncate(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((end, _)) => format!("{}…", &text[..end]),
        None => text.to_string(),
    }
}

/// Counts model turns: one per distinct assistant message, however many lines it spans.
#[derive(Debug, Default)]
pub struct TurnCounter {
    seen: HashSet<String>,
}

impl TurnCounter {
    pub fn observe(&mut self, event: &StreamEvent) -> usize {
        if let StreamEvent::Text { message_id, .. } | StreamEvent::ToolUse { message_id, .. } =
            event
        {
            self.seen.insert(message_id.clone());
        }
        self.seen.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("fixtures/stream.jsonl");

    fn all_events() -> Vec<StreamEvent> {
        FIXTURE.lines().flat_map(parse_line).collect()
    }

    #[test]
    fn parses_every_kind_of_line_in_the_fixture() {
        let events = all_events();
        assert_eq!(
            events[0],
            StreamEvent::Init {
                model: "claude-opus-5-5".into(),
                tools: vec!["Read".into(), "Edit".into(), "Bash".into()],
            }
        );
        assert!(
            matches!(&events[1], StreamEvent::Raw { text } if text.contains("rate_limit_event"))
        );
        assert_eq!(
            events[2],
            StreamEvent::Text {
                message_id: "msg_01".into(),
                text: "I'll read the file first.".into(),
            }
        );
        assert_eq!(
            events[3],
            StreamEvent::ToolUse {
                message_id: "msg_01".into(),
                name: "Read".into(),
                input: r#"{"file_path":"src/lib.rs"}"#.into(),
            }
        );
        assert_eq!(
            events[4],
            StreamEvent::ToolResult {
                content: "pub fn add(a: i32, b: i32) -> i32 { a + b }".into(),
                is_error: false,
            }
        );
        assert_eq!(
            events[6],
            StreamEvent::ToolResult {
                content: "test result: FAILED".into(),
                is_error: true,
            }
        );
        assert_eq!(
            events[8],
            StreamEvent::Result {
                is_error: false,
                text: "Done.".into(),
                turns: 3,
                cost_usd: Some(0.0150312),
            }
        );
        assert_eq!(
            events[9],
            StreamEvent::Raw {
                text: "not json at all".into()
            }
        );
        assert_eq!(events.len(), 10);
    }

    #[test]
    fn thinking_blocks_and_blank_lines_are_skipped() {
        let line = r#"{"type":"assistant","message":{"id":"m","content":[{"type":"thinking","thinking":"x"}]}}"#;
        assert!(parse_line(line).is_empty());
        assert!(parse_line("   ").is_empty());
    }

    #[test]
    fn a_result_without_is_error_counts_as_an_error() {
        let events = parse_line(r#"{"type":"result","result":"Not logged in"}"#);
        assert!(
            matches!(&events[0], StreamEvent::Result { is_error: true, text, .. } if text == "Not logged in")
        );
    }

    #[test]
    fn known_type_with_an_unexpected_shape_is_kept_raw() {
        let events = parse_line(r#"{"type":"assistant","message":"oops"}"#);
        assert!(matches!(&events[0], StreamEvent::Raw { .. }));
    }

    #[test]
    fn long_text_is_cut_on_a_char_boundary() {
        let long = "é".repeat(MAX_TEXT + 10);
        let cut = truncate(&long, MAX_TEXT);
        assert_eq!(cut.chars().count(), MAX_TEXT + 1);
        assert!(cut.ends_with('…'));
    }

    #[test]
    fn turns_count_distinct_assistant_messages() {
        let mut counter = TurnCounter::default();
        let turns: Vec<usize> = all_events().iter().map(|e| counter.observe(e)).collect();
        assert_eq!(turns.last(), Some(&3));
        assert_eq!(turns[3], 1, "two lines of msg_01 are one turn");
    }
}

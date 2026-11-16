use serde::Serialize;

/// Same rules as `.githooks/pre-commit`; a test fails when the two drift apart.
const NAMES: [&str; 7] = [
    "api_key", "api-key", "apikey", "secret", "token", "password", "passwd",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecretHit {
    pub rule: &'static str,
    pub path: String,
}

/// The first rule an added line breaks, if any.
pub fn check_line(line: &str) -> Option<&'static str> {
    if has_token(line, "sk-ant-", 20, is_key_char) {
        Some("Anthropic API key")
    } else if has_github_token(line) {
        Some("GitHub token")
    } else if has_token(line, "AKIA", 16, |c| {
        c.is_ascii_digit() || c.is_ascii_uppercase()
    }) {
        Some("AWS access key")
    } else if line.contains("-----BEGIN ") && line.contains("PRIVATE KEY-----") {
        Some("private key")
    } else if has_key_assignment(line) {
        Some("key assignment")
    } else {
        None
    }
}

fn is_key_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-'
}

fn is_value_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || "/+_=-".contains(c)
}

fn run_len(text: &str, accept: impl Fn(char) -> bool) -> usize {
    text.chars().take_while(|c| accept(*c)).count()
}

fn has_token(line: &str, prefix: &str, min: usize, accept: impl Fn(char) -> bool + Copy) -> bool {
    line.match_indices(prefix)
        .any(|(at, _)| run_len(&line[at + prefix.len()..], accept) >= min)
}

fn has_github_token(line: &str) -> bool {
    let alnum = |c: char| c.is_ascii_alphanumeric();
    ["ghp_", "gho_", "ghu_", "ghs_", "ghr_"]
        .iter()
        .any(|prefix| has_token(line, prefix, 36, alnum))
        || has_token(line, "github_pat_", 22, |c| {
            c.is_ascii_alphanumeric() || c == '_'
        })
}

// only values that mix letters and digits count, so long plain words and numbers pass
fn has_key_assignment(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    NAMES.iter().any(|name| {
        lower.match_indices(name).any(|(at, _)| {
            let rest = line[at + name.len()..]
                .trim_start_matches(['"', '\''])
                .trim_start();
            let Some(rest) = rest.strip_prefix([':', '=']) else {
                return false;
            };
            let value = rest
                .trim_start()
                .trim_start_matches(['"', '\''])
                .chars()
                .take_while(|c| is_value_char(*c))
                .collect::<String>();
            value.len() >= 32
                && value.chars().any(|c| c.is_ascii_digit())
                && value.chars().any(|c| c.is_ascii_alphabetic())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // built at run time so the commit hook does not flag this file
    fn join(parts: &[&str]) -> String {
        parts.concat()
    }

    #[test]
    fn finds_each_kind_of_secret() {
        let cases = [
            (
                join(&["key = sk-ant-", "abcdefghijklmnopqrstuvwx"]),
                "Anthropic API key",
            ),
            (
                join(&["ghp_", "abcdefghijklmnopqrstuvwxyz0123456789"]),
                "GitHub token",
            ),
            (
                join(&["github_pat_", "abcdefghijklmnopqrstuv"]),
                "GitHub token",
            ),
            (join(&["id AKIA", "ABCDEFGHIJKLMNOP"]), "AWS access key"),
            (
                join(&["-----BEGIN RSA ", "PRIVATE KEY-----"]),
                "private key",
            ),
            (
                join(&["API_KEY = \"", "abcd1234abcd1234abcd1234abcd1234\""]),
                "key assignment",
            ),
            (
                join(&["password: '", "Zx9Zx9Zx9Zx9Zx9Zx9Zx9Zx9Zx9Zx9Zx9'"]),
                "key assignment",
            ),
        ];
        for (line, rule) in cases {
            assert_eq!(check_line(&line), Some(rule), "{line}");
        }
    }

    #[test]
    fn ordinary_code_passes() {
        for line in [
            "let token = next_token();",
            "const SECRET_NAME = \"abcdefabcdefabcdefabcdefabcdefabcdef\";",
            "password = \"12345678901234567890123456789012\"",
            "sk-ant- is the prefix",
            "AKIA is a prefix",
        ] {
            assert_eq!(check_line(line), None, "{line}");
        }
    }

    #[test]
    fn the_commit_hook_still_has_the_same_rules() {
        let hook = include_str!("../../../../.githooks/pre-commit");
        for fragment in [
            "sk-ant-[A-Za-z0-9_-]{20,}",
            "gh[pousr]_[A-Za-z0-9]{36,}",
            "github_pat_[A-Za-z0-9_]{22,}",
            "AKIA[0-9A-Z]{16}",
            "-----BEGIN ([A-Z]+ )*PRIVATE KEY-----",
            "api[_-]?key|secret|token|passw(or)?d",
            "[A-Za-z0-9/+_=-]{32,}",
        ] {
            assert!(hook.contains(fragment), "hook changed: {fragment}");
        }
    }
}

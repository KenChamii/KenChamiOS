/// Trims tags, drops empty ones and removes duplicates, keeping the first.
pub fn clean(tags: Vec<String>) -> Vec<String> {
    let mut cleaned: Vec<String> = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if !tag.is_empty() && !cleaned.iter().any(|t| t == tag) {
            cleaned.push(tag.to_string());
        }
    }
    cleaned
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_dedupes() {
        let tags = vec![" rust ".into(), "".into(), "rust".into(), "ui".into()];
        assert_eq!(clean(tags), vec!["rust", "ui"]);
    }
}

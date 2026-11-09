/** "ui, rust" to ["ui", "rust"]; blanks are dropped. */
export function parseTags(text: string): string[] {
  return text
    .split(",")
    .map((tag) => tag.trim())
    .filter(Boolean);
}

export function formatTags(tags: string[]): string {
  return tags.join(", ");
}

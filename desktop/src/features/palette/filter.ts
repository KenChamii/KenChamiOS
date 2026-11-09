export type Command = {
  id: string;
  label: string;
  group: string;
  /** extra words that should find it, like "timer" for focus */
  keywords?: string;
  /** listed only once something is typed, so long lists stay out of the way */
  searchOnly?: boolean;
  run: () => void;
};

const MAX_RESULTS = 50;

/** Lower is better; null when the query does not match at all. */
export function matchScore(text: string, query: string): number | null {
  const haystack = text.toLowerCase();
  if (haystack.startsWith(query)) return 0;
  if (haystack.split(/\s+/).some((word) => word.startsWith(query))) return 1;
  if (haystack.includes(query)) return 2;
  return isSubsequence(haystack, query) ? 4 : null;
}

// "nwp" finds "new project"
function isSubsequence(text: string, query: string): boolean {
  let at = 0;
  for (const char of text) {
    if (char === query[at]) at += 1;
    if (at === query.length) return true;
  }
  return false;
}

export function filterCommands(commands: Command[], query: string): Command[] {
  const needle = query.trim().toLowerCase();
  if (!needle) return commands.filter((command) => !command.searchOnly).slice(0, MAX_RESULTS);

  return commands
    .map((command, index) => {
      const label = matchScore(command.label, needle) ?? Infinity;
      const keywords = command.keywords?.toLowerCase().includes(needle) ? 3 : Infinity;
      return { command, index, score: Math.min(label, keywords) };
    })
    .filter((match) => match.score !== Infinity)
    .sort((a, b) => a.score - b.score || a.index - b.index)
    .slice(0, MAX_RESULTS)
    .map((match) => match.command);
}

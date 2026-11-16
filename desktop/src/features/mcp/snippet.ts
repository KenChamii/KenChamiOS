/** The command that connects Claude Code to the local server. */
export function claudeCommand(port: number, token: string): string {
  return `claude mcp add --transport http kenchamios http://127.0.0.1:${port}/mcp --header "Authorization: Bearer ${token}"`;
}

/** Which tool a call used, as a short readable name. */
export function toolLabel(tool: string): string {
  return tool.replace(/_/g, " ");
}

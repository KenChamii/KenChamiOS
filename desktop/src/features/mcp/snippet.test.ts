import { describe, expect, it } from "vitest";
import { claudeCommand, toolLabel } from "./snippet";

describe("claudeCommand", () => {
  it("names the transport, the local address and the token header", () => {
    const command = claudeCommand(47821, "abc123");
    expect(command).toContain("--transport http");
    expect(command).toContain("http://127.0.0.1:47821/mcp");
    expect(command).toContain('--header "Authorization: Bearer abc123"');
  });
});

describe("toolLabel", () => {
  it("turns a tool name into words", () => {
    expect(toolLabel("add_inbox_item")).toBe("add inbox item");
  });
});

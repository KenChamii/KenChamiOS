import { describe, expect, it } from "vitest";
import { filterCommands, matchScore, type Command } from "./filter";

function command(label: string, extra: Partial<Command> = {}): Command {
  return { id: label, label, group: "Test", run: () => {}, ...extra };
}

describe("matchScore", () => {
  it("ranks a prefix above a word start above a plain substring", () => {
    expect(matchScore("Projects", "pro")).toBe(0);
    expect(matchScore("New project", "pro")).toBe(1);
    expect(matchScore("Reproduce", "pro")).toBe(2);
  });

  it("matches letters in order as a last resort", () => {
    expect(matchScore("New project", "nwp")).toBe(4);
    expect(matchScore("New project", "pwn")).toBeNull();
  });
});

describe("filterCommands", () => {
  const commands = [
    command("Dashboard"),
    command("New project"),
    command("Projects"),
    command("Start focus", { keywords: "timer pomodoro" }),
    command("Dune", { searchOnly: true }),
  ];

  it("hides search-only commands until something is typed", () => {
    expect(filterCommands(commands, "  ").map((c) => c.label)).toEqual([
      "Dashboard",
      "New project",
      "Projects",
      "Start focus",
    ]);
    expect(filterCommands(commands, "dun").map((c) => c.label)).toEqual(["Dune"]);
  });

  it("puts better matches first and keeps the original order for ties", () => {
    expect(filterCommands(commands, "pro").map((c) => c.label)).toEqual([
      "Projects",
      "New project",
    ]);
  });

  it("finds commands by their keywords, ignoring case", () => {
    expect(filterCommands(commands, "TIMER").map((c) => c.label)).toEqual(["Start focus"]);
  });
});

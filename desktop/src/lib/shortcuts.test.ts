import { describe, expect, it } from "vitest";
import { IDLE, readShortcut, type KeyPress } from "./shortcuts";

function press(key: string, extra: Partial<KeyPress> = {}): KeyPress {
  return {
    key,
    ctrlKey: false,
    metaKey: false,
    altKey: false,
    shiftKey: false,
    typing: false,
    at: 0,
    ...extra,
  };
}

describe("readShortcut", () => {
  it("opens the palette and quick add with ctrl or cmd, even while typing", () => {
    expect(readShortcut(press("k", { ctrlKey: true, typing: true }), IDLE).action).toEqual({
      type: "palette",
    });
    expect(readShortcut(press("K", { metaKey: true }), IDLE).action).toEqual({ type: "palette" });
    expect(readShortcut(press("n", { ctrlKey: true }), IDLE).action).toEqual({
      type: "quick-add",
    });
  });

  it("opens plan my day with ctrl+shift+p only", () => {
    expect(readShortcut(press("P", { ctrlKey: true, shiftKey: true }), IDLE).action).toEqual({
      type: "plan-day",
    });
    expect(readShortcut(press("p", { ctrlKey: true }), IDLE).action).toBeNull();
  });

  it("reads G then a letter as a jump", () => {
    const first = readShortcut(press("g", { at: 1000 }), IDLE);
    expect(first.action).toBeNull();
    const second = readShortcut(press("p", { at: 1500 }), first.state);
    expect(second.action).toEqual({ type: "go", key: "p" });
    expect(second.state).toEqual(IDLE);
  });

  it("forgets G after a short wait", () => {
    const first = readShortcut(press("g", { at: 0 }), IDLE);
    expect(readShortcut(press("d", { at: 5000 }), first.state).action).toBeNull();
  });

  it("ignores plain keys while typing", () => {
    const first = readShortcut(press("g", { typing: true }), IDLE);
    expect(first.state).toEqual(IDLE);
    expect(readShortcut(press("?", { typing: true }), IDLE).action).toBeNull();
  });

  it("opens help with ?", () => {
    expect(readShortcut(press("?"), IDLE).action).toEqual({ type: "help" });
  });

  it("drops a pending G when another modifier is used", () => {
    const first = readShortcut(press("g"), IDLE);
    const second = readShortcut(press("d", { altKey: true }), first.state);
    expect(second.action).toBeNull();
    expect(second.state).toEqual(IDLE);
  });
});

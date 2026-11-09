export type KeyPress = {
  key: string;
  ctrlKey: boolean;
  metaKey: boolean;
  altKey: boolean;
  /** focus is in a text field, so plain letters belong to it */
  typing: boolean;
  at: number;
};

export type ShortcutAction =
  { type: "palette" } | { type: "quick-add" } | { type: "help" } | { type: "go"; key: string };

/** When G was pressed, while waiting for the second key. */
export type ShortcutState = { goAt: number | null };

export const IDLE: ShortcutState = { goAt: null };

const GO_WINDOW_MS = 1500;

export function readShortcut(
  press: KeyPress,
  state: ShortcutState,
): { action: ShortcutAction | null; state: ShortcutState } {
  const key = press.key.toLowerCase();
  const withModifier = press.ctrlKey || press.metaKey;

  if (withModifier && !press.altKey) {
    if (key === "k") return { action: { type: "palette" }, state: IDLE };
    if (key === "n") return { action: { type: "quick-add" }, state: IDLE };
  }
  if (withModifier || press.altKey || press.typing) return { action: null, state: IDLE };

  const waiting = state.goAt !== null && press.at - state.goAt <= GO_WINDOW_MS;
  if (waiting && /^[a-z]$/.test(key)) return { action: { type: "go", key }, state: IDLE };
  if (key === "g") return { action: null, state: { goAt: press.at } };
  if (press.key === "?") return { action: { type: "help" }, state: IDLE };
  return { action: null, state: IDLE };
}

import { describe, expect, it } from "vitest";
import fixture from "../../../../shared/fixtures/quick-add/parse.json";
import { parseQuickAdd } from "./parse";

describe("parseQuickAdd", () => {
  it("matches every shared case", () => {
    for (const testCase of fixture.cases) {
      const today = ("today" in testCase && testCase.today) || fixture.today;
      expect(parseQuickAdd(testCase.input, today), testCase.input).toEqual(testCase.expected);
    }
  });
});

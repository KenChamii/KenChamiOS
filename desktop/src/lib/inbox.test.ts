import { describe, expect, it } from "vitest";
import known from "../../../shared/schema/well-known.json";
import { INBOX_ID } from "./inbox";

describe("INBOX_ID", () => {
  it("is the id every device shares", () => {
    expect(INBOX_ID).toBe(known.inbox_project.id);
  });
});

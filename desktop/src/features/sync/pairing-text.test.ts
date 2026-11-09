import { describe, expect, it } from "vitest";
import { countdown, groupFingerprint, pairingText } from "./pairing-text";

describe("pairing text", () => {
  it("keeps every field the phone needs", () => {
    const payload = { v: 1, host: "192.168.1.20", port: 47821, fp: "ab", code: "7K3M9QX2TV" };
    expect(JSON.parse(pairingText(payload))).toEqual(payload);
  });

  it("counts down in minutes and seconds and stops at zero", () => {
    expect(countdown(299)).toBe("4:59");
    expect(countdown(60)).toBe("1:00");
    expect(countdown(-3)).toBe("0:00");
  });

  it("groups the fingerprint by eight characters", () => {
    expect(groupFingerprint("0123456789abcdef")).toBe("01234567 89abcdef");
  });
});

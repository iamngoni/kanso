import { describe, expect, it } from "vitest";

import { isWireId } from "../src/protocol";

const TAIL = "AbCdEfGhIjKlMnOpQrStUv";

describe("wire ids", () => {
  it("accepts every kind the clients sync, including review documents", () => {
    for (const kind of ["note", "book", "tag", "file", "review"]) {
      expect(isWireId(`${kind}:${TAIL}`)).toBe(true);
    }
  });

  it("refuses an unknown kind or a malformed tail", () => {
    expect(isWireId(`zzz:${TAIL}`)).toBe(false);
    expect(isWireId(`reviews:${TAIL}`)).toBe(false);
    expect(isWireId(`review:${TAIL}x`)).toBe(false);
    expect(isWireId(`review:${TAIL.slice(1)}`)).toBe(false);
    expect(isWireId(`review:${TAIL.slice(1)}!`)).toBe(false);
    expect(isWireId("review:")).toBe(false);
  });
});

/**
 * The price page's numbers. They must equal the server defaults in docs/ACCOUNTS.md
 * ("Quotas and capacity"): 104,857,600 bytes of notes, 1,073,741,824 bytes of files
 * and of uploads a day. Change both together.
 */
import { describe, expect, it } from "vitest";
import { QUOTA_FILES_BYTES, QUOTA_NOTES_BYTES, QUOTA_UPLOAD_PER_DAY_BYTES, formatSize } from "./quota";

describe("quota constants", () => {
  it("equal the server defaults", () => {
    expect(QUOTA_NOTES_BYTES).toBe(104_857_600);
    expect(QUOTA_FILES_BYTES).toBe(1_073_741_824);
    expect(QUOTA_UPLOAD_PER_DAY_BYTES).toBe(1_073_741_824);
  });

  it("print as whole units", () => {
    expect(formatSize(QUOTA_NOTES_BYTES)).toBe("100 MiB");
    expect(formatSize(QUOTA_FILES_BYTES)).toBe("1 GiB");
    expect(formatSize(QUOTA_UPLOAD_PER_DAY_BYTES)).toBe("1 GiB");
  });
});

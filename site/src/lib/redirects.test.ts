/**
 * Tests for the site's redirects: www to the apex keeping the path and query,
 * /metrics to the usage-count note, and everything else served.
 */
import { describe, expect, it } from "vitest";
import { METRICS_NOTE_URL, redirectFor } from "./redirects";

const redirect = (href: string) => {
  const response = redirectFor(new URL(href));
  return response && { status: response.status, location: response.headers.get("location") };
};

describe("redirectFor", () => {
  it("sends www to the apex for good, keeping the path and query", () => {
    expect(redirect("https://www.kansoapp.app/download?x=1")).toEqual({ status: 301, location: "https://kansoapp.app/download?x=1" });
  });

  it("sends /metrics to the usage-count note, temporarily", () => {
    expect(METRICS_NOTE_URL).toBe("https://github.com/iamngoni/kanso/blob/master/docs/METRICS-NOTE.md");
    for (const href of ["https://kansoapp.app/metrics", "https://kansoapp.app/metrics/", "https://kansoapp.app/metrics?from=app"]) {
      expect(redirect(href), href).toEqual({ status: 302, location: METRICS_NOTE_URL });
    }
  });

  it("serves every other path", () => {
    for (const href of ["https://kansoapp.app/", "https://kansoapp.app/privacy", "https://kansoapp.app/metrics/x", "https://kansoapp.app/metricsx"]) {
      expect(redirect(href), href).toBeNull();
    }
  });
});

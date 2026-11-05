/**
 * Tests for the tour's library and its date helpers: due-date tokens are all
 * replaced relative to "today", dayFrom is a correct day calendar (month and
 * leap-year edges, additive, monotonic), and the note list's "edited" labels.
 */
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import { NOTEBOOKS, dayFrom, editedLabel, library } from "./notes";

const dates = fc.date({ min: new Date(1971, 0, 1), max: new Date(2098, 11, 31), noInvalidDate: true });
const parseDay = (iso: string): Date => {
  const [y, m, d] = iso.split("-").map(Number) as [number, number, number];
  return new Date(y, m - 1, d);
};

describe("dayFrom", () => {
  const today = new Date(2026, 9, 20, 15, 45);

  it("gives today's date at offset zero, whatever the time of day", () => {
    expect(dayFrom(today, 0)).toBe("2026-10-20");
    expect(dayFrom(new Date(2026, 9, 20, 0, 0, 0), 0)).toBe("2026-10-20");
    expect(dayFrom(new Date(2026, 9, 20, 23, 59, 59), 0)).toBe("2026-10-20");
  });

  it("counts days forwards and backwards", () => {
    expect(dayFrom(today, 1)).toBe("2026-10-21");
    expect(dayFrom(today, -1)).toBe("2026-10-19");
    expect(dayFrom(today, 12)).toBe("2026-11-01");
  });

  it("crosses year ends and leap days", () => {
    expect(dayFrom(new Date(2026, 11, 31), 1)).toBe("2027-01-01");
    expect(dayFrom(new Date(2027, 0, 1), -1)).toBe("2026-12-31");
    expect(dayFrom(new Date(2028, 1, 28), 1)).toBe("2028-02-29");
    expect(dayFrom(new Date(2027, 1, 28), 1)).toBe("2027-03-01");
  });

  it("does not change the date it is given", () => {
    const d = new Date(2026, 9, 20);
    dayFrom(d, 40);
    expect(d.getTime()).toBe(new Date(2026, 9, 20).getTime());
  });

  it("at offset zero is the date's own local day, always as YYYY-MM-DD", () => {
    fc.assert(
      fc.property(dates, (d) => {
        const out = dayFrom(d, 0);
        return /^\d{4}-\d{2}-\d{2}$/.test(out) && out === d.toLocaleDateString("sv-SE");
      }),
    );
  });

  it("increases strictly with the offset", () => {
    fc.assert(
      fc.property(dates, fc.integer({ min: -4000, max: 4000 }), fc.integer({ min: 1, max: 4000 }), (d, a, gap) => dayFrom(d, a) < dayFrom(d, a + gap)),
    );
  });

  it("adds: a day found from a found day is the sum of the offsets", () => {
    fc.assert(
      fc.property(dates, fc.integer({ min: -2000, max: 2000 }), fc.integer({ min: -2000, max: 2000 }), (d, a, b) => {
        return dayFrom(parseDay(dayFrom(d, a)), b) === dayFrom(d, a + b);
      }),
    );
  });
});

describe("library", () => {
  const today = new Date(2026, 9, 20);

  it("shows the agents' setup as a key in a config entry, never a login or a command", () => {
    const agents = library(today).find((n) => n.id === "agents");
    expect(agents?.body).toContain('"Authorization": "Bearer kcl_…"');
    expect(agents?.body).not.toContain("Basic");
    expect(agents?.body).not.toContain("password");
    expect(agents?.body).not.toContain("mcp add");
  });

  it("fills the tasks note's due dates relative to today", () => {
    const tasks = library(today).find((n) => n.id === "tasks");
    expect(tasks?.body).toContain("Send the draft to Sam @2026-10-19");
    expect(tasks?.body).toContain("Book the plumber @2026-10-20");
    expect(tasks?.body).toContain("Back up the photos @2026-10-17");
    expect(tasks?.body).toContain("Plan the garden @2026-10-24");
  });

  it("fills a positive token written with or without a plus", () => {
    const sunday = library(today).find((n) => n.id === "sunday");
    expect(sunday?.body).toContain("@2026-10-22");
  });

  it("leaves the app's own template placeholders alone", () => {
    const capture = library(today).find((n) => n.id === "capture");
    expect(capture?.body).toContain("{{date}}");
  });

  it("has unique ids, known notebooks, and starts on the welcome note", () => {
    const notes = library(today);
    expect(new Set(notes.map((n) => n.id)).size).toBe(notes.length);
    expect(notes.every((n) => (NOTEBOOKS as readonly string[]).includes(n.notebook))).toBe(true);
    expect(notes[0]?.id).toBe("welcome");
  });

  it("is the same library each time and does not change a previous one", () => {
    const first = library(today);
    const bodies = first.map((n) => n.body);
    library(new Date(2030, 0, 1));
    expect(first.map((n) => n.body)).toEqual(bodies);
    expect(library(today)).toEqual(first);
  });

  it("leaves no due-date token behind, for any day", () => {
    fc.assert(
      fc.property(dates, (d) => {
        const notes = library(d);
        return notes.every((n) => !/\{\{[+-]?\d/.test(n.body) && !n.body.includes("undefined") && !n.body.includes("NaN"));
      }),
    );
  });

  it("only changes the notes that had tokens, and keeps every other field", () => {
    fc.assert(
      fc.property(dates, (d) => {
        const a = library(d);
        const b = library(new Date(2000, 0, 1));
        return a.every((n, i) => n.id === b[i]?.id && n.notebook === b[i]?.notebook && n.edited === b[i]?.edited);
      }),
    );
  });
});

describe("editedLabel", () => {
  it.each([
    [0, "Just now"],
    [0.5, "Just now"],
    [-3, "Just now"],
    [1, "1 min ago"],
    [14, "14 min ago"],
    [59, "59 min ago"],
    [60, "1 h ago"],
    [95, "2 h ago"],
    [720, "12 h ago"],
    [1440, "Yesterday"],
    [2000, "Yesterday"],
    [2880, "2 days ago"],
    [10080, "7 days ago"],
  ])("editedLabel(%d) = %s", (minutes, label) => {
    expect(editedLabel(minutes)).toBe(label);
  });

  it("says one of the five forms for any whole number of minutes", () => {
    fc.assert(
      fc.property(fc.nat(1_000_000), (m) => /^(Just now|\d+ min ago|\d+ h ago|Yesterday|\d+ days ago)$/.test(editedLabel(m))),
    );
  });

  it("uses the unit that matches the size of the time", () => {
    fc.assert(
      fc.property(fc.nat(1_000_000), (m) => {
        const label = editedLabel(m);
        if (m < 1) return label === "Just now";
        if (m < 60) return label.endsWith(" min ago");
        if (m < 1410) return label.endsWith(" h ago");
        return label === "Yesterday" || label.endsWith(" days ago");
      }),
    );
  });
});

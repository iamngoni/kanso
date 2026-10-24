/**
 * Tests for the in-browser Kanso's state: every reducer action, the note
 * list's filtering and ordering, tag counts, quick open's destinations and
 * title lookup. A model-style property drives random action sequences over
 * the tour library and checks the state invariants after every step.
 */
import fc from "fast-check";
import { describe, expect, it } from "vitest";
import {
  type Action,
  type Filter,
  type State,
  allTags,
  destinations,
  findByTitle,
  initialState,
  reducer,
  selectedNote,
  visibleNotes,
} from "./library";
import { tagsOf, title } from "./markdown";
import { type Note, NOTEBOOKS, library } from "./notes";

const TODAY = new Date(2026, 9, 20);

const FIXTURE: Note[] = [
  { id: "alpha", notebook: "N1", edited: 5, pinned: true, body: "# Alpha\n\n- [ ] one @2026-10-20 #work\n- [x] two" },
  { id: "beta", notebook: "N2", edited: 1, body: "# Beta\n\ntext #work/sub and #other" },
  { id: "gamma", notebook: "N1", edited: 9, body: "# Gamma\n\n## Part One\n\n### Deep dive\n\n#workshop" },
  { id: "my-new-note", notebook: "N2", edited: 3, body: "# Not the same title" },
];
const fresh = (): State => initialState(FIXTURE);
const ids = (notes: readonly Note[]) => notes.map((n) => n.id);
const run = (state: State, ...actions: Action[]) => actions.reduce(reducer, state);

describe("initialState", () => {
  it("selects the first note, shows everything in split mode, with nothing open", () => {
    expect(fresh()).toEqual({ notes: FIXTURE, selected: "alpha", filter: { kind: "all" }, mode: "split", server: false, palette: false, query: "" });
  });

  it("copes with no notes", () => {
    const state = initialState([]);
    expect(state.selected).toBe("");
    expect(selectedNote(state)).toBeUndefined();
    expect(visibleNotes(state)).toEqual([]);
  });
});

describe("reducer", () => {
  it("select opens a note and closes the palette", () => {
    const s = run(fresh(), { type: "palette", open: true }, { type: "select", id: "gamma" });
    expect(s.selected).toBe("gamma");
    expect(s.palette).toBe(false);
    expect(selectedNote(s)?.id).toBe("gamma");
  });

  it("select of an unknown id is a no-op (same state object)", () => {
    const s = fresh();
    expect(reducer(s, { type: "select", id: "nope" })).toBe(s);
  });

  it("edit changes only that note's body and marks it edited just now", () => {
    const s = reducer(fresh(), { type: "edit", id: "beta", body: "# Beta 2" });
    expect(s.notes.find((n) => n.id === "beta")).toMatchObject({ body: "# Beta 2", edited: 0, notebook: "N2" });
    expect(s.notes.filter((n) => n.id !== "beta")).toEqual(FIXTURE.filter((n) => n.id !== "beta"));
    expect(s.notes.map((n) => n.id)).toEqual(ids(FIXTURE));
  });

  it("edit of an unknown id changes no note", () => {
    expect(reducer(fresh(), { type: "edit", id: "nope", body: "x" }).notes).toEqual(FIXTURE);
  });

  it("filter sets the filter and clears the search", () => {
    const s = run(fresh(), { type: "query", query: "al" }, { type: "filter", filter: { kind: "pinned" } });
    expect(s.filter).toEqual({ kind: "pinned" });
    expect(s.query).toBe("");
  });

  it("mode, query and palette set their fields", () => {
    const s = run(fresh(), { type: "mode", mode: "edit" }, { type: "query", query: "q" }, { type: "palette", open: true });
    expect(s).toMatchObject({ mode: "edit", query: "q", palette: true });
  });

  it("server switches the server view on and off, and closes the palette", () => {
    const on = run(fresh(), { type: "palette", open: true }, { type: "server", on: true });
    expect(on.server).toBe(true);
    expect(on.palette).toBe(false);
    expect(reducer(on, { type: "server", on: false }).server).toBe(false);
  });

  it("toggle-task ticks the line in the note, marks it edited, and leaves other lines alone", () => {
    const s = reducer(fresh(), { type: "toggle-task", id: "alpha", line: 2 });
    expect(s.notes[0]?.body).toBe("# Alpha\n\n- [x] one @2026-10-20 #work\n- [x] two");
    expect(s.notes[0]?.edited).toBe(0);
    expect(reducer(s, { type: "toggle-task", id: "alpha", line: 2 }).notes[0]?.body).toBe(FIXTURE[0]?.body);
  });

  it("toggle-task on a line that is not a task leaves the text as it was", () => {
    expect(reducer(fresh(), { type: "toggle-task", id: "alpha", line: 0 }).notes[0]?.body).toBe(FIXTURE[0]?.body);
  });

  describe("open-wiki", () => {
    it("opens the note whose title matches, ignoring case and surrounding space", () => {
      const s = reducer(fresh(), { type: "open-wiki", target: "  bETa " });
      expect(s.selected).toBe("beta");
      expect(s.notes).toEqual(FIXTURE);
    });

    it("creates a note titled by the link when none matches, in the current notebook", () => {
      const s = reducer(fresh(), { type: "open-wiki", target: "Brand New" });
      expect(s.notes).toHaveLength(FIXTURE.length + 1);
      expect(s.notes[0]).toEqual({ id: "brand-new", notebook: "N1", edited: 0, body: "# Brand New\n\n" });
      expect(s.selected).toBe("brand-new");
      expect(s.query).toBe("");
    });

    it("gives a created note an id no other note has", () => {
      const s = reducer(fresh(), { type: "open-wiki", target: "My New Note" });
      expect(s.selected).toBe("my-new-note-2");
      const again = reducer(s, { type: "open-wiki", target: "Not the same title" });
      expect(again.selected).toBe("my-new-note");
      const twice = reducer({ ...s, notes: s.notes }, { type: "open-wiki", target: "My New Note" });
      expect(twice.selected).toBe("my-new-note-2");
    });

    it("names a note Untitled for a blank target and falls back to a plain id for non-Latin titles", () => {
      expect(reducer(fresh(), { type: "open-wiki", target: "   " }).selected).toBe("untitled");
      const jp = reducer(fresh(), { type: "open-wiki", target: "日本語" });
      expect(jp.selected).toBe("note");
      expect(jp.notes[0]?.body).toBe("# 日本語\n\n");
    });

    it("switches Preview to Edit so the new note can be written, and keeps other modes", () => {
      expect(reducer({ ...fresh(), mode: "preview" }, { type: "open-wiki", target: "X" }).mode).toBe("edit");
      expect(reducer({ ...fresh(), mode: "split" }, { type: "open-wiki", target: "X" }).mode).toBe("split");
      expect(reducer({ ...fresh(), mode: "edit" }, { type: "open-wiki", target: "X" }).mode).toBe("edit");
    });
  });

  describe("new-note", () => {
    it("adds an empty heading note at the top and selects it", () => {
      const s = reducer(fresh(), { type: "new-note" });
      expect(s.notes[0]).toEqual({ id: "untitled", notebook: "N1", edited: 0, body: "# " });
      expect(s.selected).toBe("untitled");
      expect(title(s.notes[0]?.body ?? "")).toBe("Untitled");
    });

    it("numbers further new notes", () => {
      const s = run(fresh(), { type: "new-note" }, { type: "new-note" }, { type: "new-note" });
      expect(ids(s.notes).slice(0, 3)).toEqual(["untitled-3", "untitled-2", "untitled"]);
    });

    it("goes into the filtered notebook and keeps the notebook filter", () => {
      const s = run(fresh(), { type: "filter", filter: { kind: "notebook", name: "N2" } }, { type: "new-note" });
      expect(s.notes[0]?.notebook).toBe("N2");
      expect(s.filter).toEqual({ kind: "notebook", name: "N2" });
    });

    it("resets a tag, pinned or tasks filter to All, so the new note is visible", () => {
      for (const filter of [{ kind: "tag", name: "work" }, { kind: "pinned" }, { kind: "tasks" }] as const) {
        const s = run(fresh(), { type: "filter", filter }, { type: "new-note" });
        expect(s.filter).toEqual({ kind: "all" });
        expect(ids(visibleNotes(s))).toContain(s.selected);
      }
    });

    it("leaves Preview for Edit and clears the search", () => {
      const s = run({ ...fresh(), mode: "preview" }, { type: "query", query: "zz" }, { type: "new-note" });
      expect(s.mode).toBe("edit");
      expect(s.query).toBe("");
    });

    it("falls back to Start here when there is no current note", () => {
      const s = reducer({ ...initialState([]) }, { type: "new-note" });
      expect(s.notes[0]).toMatchObject({ notebook: "Start here", id: "untitled" });
    });
  });
});

describe("findByTitle", () => {
  it("finds a note by its title, ignoring case and space", () => {
    expect(findByTitle(FIXTURE, " GAMMA ")?.id).toBe("gamma");
    expect(findByTitle(FIXTURE, "Part One")).toBeUndefined();
    expect(findByTitle([], "x")).toBeUndefined();
  });
});

describe("visibleNotes", () => {
  const view = (filter: Filter, query = "") => ids(visibleNotes({ ...fresh(), filter, query }));

  it("lists everything for All and Tasks, pinned first and then the most recently edited", () => {
    expect(view({ kind: "all" })).toEqual(["alpha", "beta", "my-new-note", "gamma"]);
    expect(view({ kind: "tasks" })).toEqual(["alpha", "beta", "my-new-note", "gamma"]);
  });

  it("filters by pinned and by notebook", () => {
    expect(view({ kind: "pinned" })).toEqual(["alpha"]);
    expect(view({ kind: "notebook", name: "N1" })).toEqual(["alpha", "gamma"]);
    expect(view({ kind: "notebook", name: "none" })).toEqual([]);
  });

  it("filters by tag, including nested tags under the name but not longer names", () => {
    expect(view({ kind: "tag", name: "work" })).toEqual(["alpha", "beta"]);
    expect(view({ kind: "tag", name: "work/sub" })).toEqual(["beta"]);
    expect(view({ kind: "tag", name: "workshop" })).toEqual(["gamma"]);
    expect(view({ kind: "tag", name: "wor" })).toEqual([]);
  });

  it("ranks by the search over title and snippet, dropping non-matches", () => {
    expect(view({ kind: "all" }, "gam")).toEqual(["gamma"]);
    expect(view({ kind: "all" }, "zzzz")).toEqual([]);
    expect(view({ kind: "all" }, "   ")).toEqual(["alpha", "beta", "my-new-note", "gamma"]);
  });

  it("searches within the filter, not around it", () => {
    expect(view({ kind: "notebook", name: "N2" }, "alpha")).toEqual([]);
  });

  it("does not reorder the state's own notes", () => {
    const s = fresh();
    visibleNotes(s);
    expect(ids(s.notes)).toEqual(ids(FIXTURE));
  });
});

describe("allTags", () => {
  it("counts the notes carrying each tag, sorted by name", () => {
    expect(allTags(FIXTURE)).toEqual([
      { name: "other", count: 1 },
      { name: "work", count: 1 },
      { name: "work/sub", count: 1 },
      { name: "workshop", count: 1 },
    ]);
  });

  it("counts a tag once per note however often it appears", () => {
    expect(allTags([{ id: "a", notebook: "x", edited: 1, body: "#t #t #t" }, { id: "b", notebook: "x", edited: 1, body: "#t" }])).toEqual([{ name: "t", count: 2 }]);
  });

  it("finds the tour's tags", () => {
    expect(allTags(library(TODAY)).map((t) => t.name)).toEqual(["agents", "home", "home/garden", "writing"]);
  });

  it("adds up to the notes' own tags, with no tag repeated", () => {
    fc.assert(
      fc.property(fc.array(fc.array(fc.constantFrom("a", "b", "c/d", "e"), { maxLength: 6 }), { maxLength: 8 }), (tagLists) => {
        const notes: Note[] = tagLists.map((tags, i) => ({ id: `n${i}`, notebook: "x", edited: i, body: `# T\n\n${tags.map((t) => `#${t}`).join(" ")}` }));
        const counted = allTags(notes);
        const total = counted.reduce((n, t) => n + t.count, 0);
        const perNote = notes.reduce((n, note) => n + tagsOf(note.body).length, 0);
        return total === perNote && new Set(counted.map((t) => t.name)).size === counted.length;
      }),
    );
  });
});

describe("destinations", () => {
  const notes = library(TODAY);

  it("offers only notes, at most eight, for an empty query", () => {
    const out = destinations(notes, "");
    expect(out).toHaveLength(8);
    expect(out.every((d) => d.anchor === undefined)).toBe(true);
    expect(out[0]).toEqual({ id: "welcome", label: "Welcome to Kanso", detail: "Start here" });
    expect(destinations(notes, "   ").every((d) => d.anchor === undefined)).toBe(true);
  });

  it("offers headings inside notes once there is a query, but not the title again", () => {
    const out = destinations(FIXTURE, "part");
    expect(out).toEqual([{ id: "gamma", label: "Part One", detail: "Gamma", anchor: "part-one" }]);
    expect(destinations(FIXTURE, "gamma").map((d) => d.anchor)).toEqual([undefined]);
  });

  it("finds notes by fuzzy title", () => {
    expect(destinations(FIXTURE, "bt")[0]?.id).toBe("beta");
    expect(destinations(FIXTURE, "zzzzq")).toEqual([]);
  });

  // A hundred searches of the whole library: slow runners need longer than the default.
  it("never offers more than eight, whatever the query", () => {
    fc.assert(fc.property(fc.string({ maxLength: 4 }), (q) => destinations(notes, q).length <= 8));
  }, 30_000);
});

describe("any sequence of actions", () => {
  const knownIds = library(TODAY).map((n) => n.id);
  const titles = library(TODAY).map((n) => title(n.body));
  const filterArb: fc.Arbitrary<Filter> = fc.oneof(
    fc.constant<Filter>({ kind: "all" }),
    fc.constant<Filter>({ kind: "pinned" }),
    fc.constant<Filter>({ kind: "tasks" }),
    fc.constantFrom(...NOTEBOOKS).map((name): Filter => ({ kind: "notebook", name })),
    fc.constantFrom("home", "writing", "nope", "home/garden").map((name): Filter => ({ kind: "tag", name })),
  );
  const actionArb: fc.Arbitrary<Action> = fc.oneof(
    fc.oneof(fc.constantFrom(...knownIds), fc.string()).map((id): Action => ({ type: "select", id })),
    fc.record({ id: fc.constantFrom(...knownIds), body: fc.string() }).map((a): Action => ({ type: "edit", ...a })),
    filterArb.map((filter): Action => ({ type: "filter", filter })),
    fc.constantFrom("edit", "preview", "split").map((mode): Action => ({ type: "mode", mode })),
    fc.boolean().map((on): Action => ({ type: "server", on })),
    fc.boolean().map((open): Action => ({ type: "palette", open })),
    fc.string({ maxLength: 5 }).map((query): Action => ({ type: "query", query })),
    fc.record({ id: fc.constantFrom(...knownIds), line: fc.integer({ min: -2, max: 30 }) }).map((a): Action => ({ type: "toggle-task", ...a })),
    fc.oneof(fc.constantFrom(...titles), fc.string({ maxLength: 12 })).map((target): Action => ({ type: "open-wiki", target })),
    fc.constant<Action>({ type: "new-note" }),
  );

  it("keeps the selection on an existing note, ids unique, and never loses a note", () => {
    fc.assert(
      fc.property(fc.array(actionArb, { maxLength: 40 }), (actions) => {
        let state = initialState(library(TODAY));
        for (const action of actions) {
          const before = state;
          state = reducer(state, action);
          const present = new Set(ids(state.notes));
          if (present.size !== state.notes.length) return false;
          if (!present.has(state.selected)) return false;
          if (selectedNote(state) === undefined) return false;
          if (state.notes.length < before.notes.length || state.notes.length > before.notes.length + 1) return false;
          if (!ids(before.notes).every((id) => present.has(id))) return false;
          if (!ids(visibleNotes(state)).every((id) => present.has(id))) return false;
        }
        return true;
      }),
    );
  }, 20_000);

  it("only grows the library on new-note and open-wiki, and a created note is the one selected and on top", () => {
    fc.assert(
      fc.property(fc.array(actionArb, { maxLength: 25 }), (actions) => {
        let state = initialState(library(TODAY));
        for (const action of actions) {
          const before = state.notes.length;
          state = reducer(state, action);
          if (state.notes.length > before) {
            if (action.type !== "new-note" && action.type !== "open-wiki") return false;
            if (state.notes[0]?.id !== state.selected) return false;
          }
        }
        return true;
      }),
    );
  }, 20_000);

  it("makes a created note visible: after new-note it is in the list that is shown", () => {
    fc.assert(
      fc.property(fc.array(actionArb, { maxLength: 15 }), (actions) => {
        const state = reducer(actions.reduce(reducer, initialState(library(TODAY))), { type: "new-note" });
        return ids(visibleNotes({ ...state, query: "" })).includes(state.selected);
      }),
    );
  });
});

/**
 * Draws a ```mermaid fence as an SVG diagram in the demo's own colours.
 * mermaid is large, so it is fetched with a dynamic import the first time a
 * diagram is on screen and never sooner, and never in the server build (the
 * Worker would carry ~2 MB of it for nothing). Strict security level, no HTML
 * labels: visitors can edit the text, so a diagram can't carry markup or
 * click handlers.
 */

let loading: Promise<typeof import("mermaid").default> | null = null;
let queue: Promise<unknown> = Promise.resolve();
let counter = 0;

function loadMermaid() {
  // import.meta.env.SSR is a build-time constant, so the server build drops the import.
  loading ??= import.meta.env.SSR
    ? Promise.reject(new Error("mermaid only draws in the browser"))
    : import("mermaid").then((m) => m.default);
  loading.catch(() => {
    loading = null;
  });
  return loading;
}

const cssVar = (style: CSSStyleDeclaration, name: string, fallback: string) =>
  style.getPropertyValue(name).trim() || fallback;

/** mermaid's theme variables, read from the CSS variables the Preview uses. */
export function themeVariables(dark: boolean): Record<string, string | boolean> {
  const style = getComputedStyle(document.documentElement);
  const fg = cssVar(style, "--fg", dark ? "#f1efe7" : "#26241f");
  const fg2 = cssVar(style, "--fg-2", fg);
  const fg3 = cssVar(style, "--fg-3", fg2);
  const line = cssVar(style, "--line", fg2);
  const code = cssVar(style, "--code", dark ? "#30302b" : "#f4f1ea");
  const elevated = cssVar(style, "--elevated", code);
  // A node needs to stand out from the code-block background it sits on.
  const node = elevated.toLowerCase() === code.toLowerCase() ? cssVar(style, "--editor", elevated) : elevated;
  return {
    darkMode: dark,
    background: code,
    fontFamily: cssVar(style, "--font-ui", "system-ui, sans-serif"),
    primaryColor: node,
    primaryTextColor: fg,
    primaryBorderColor: fg3,
    secondaryColor: node,
    tertiaryColor: code,
    // The base theme's light grey shadow glows around nodes on a dark page.
    ...(dark ? { dropShadow: "none" } : {}),
    textColor: fg,
    lineColor: fg2,
    nodeBorder: fg3,
    mainBkg: node,
    clusterBkg: code,
    clusterBorder: line,
    edgeLabelBackground: code,
    titleColor: fg,
  };
}

/**
 * Resolves with the diagram's SVG; rejects when mermaid can't parse or draw it.
 * `stale` is asked when this diagram's turn comes: true (the text or theme has
 * moved on) skips the drawing, so typing never builds a queue of old drawings.
 */
export function renderMermaid(code: string, dark: boolean, stale: () => boolean = () => false): Promise<string> {
  const run = async () => {
    if (stale()) throw new Error("superseded");
    const mermaid = await loadMermaid();
    mermaid.initialize({
      startOnLoad: false,
      securityLevel: "strict",
      theme: "base",
      themeVariables: themeVariables(dark),
      // The root key; mermaid 12 ignores flowchart.htmlLabels when this is set,
      // and labels default to HTML. `secure` (merged with mermaid's own list)
      // stops a %%{init}%% directive in the text from turning them back on.
      htmlLabels: false,
      secure: ["htmlLabels"],
      // mermaid 12 widens every flowchart node to 120px of label; sized to
      // their text, Draft → Review → Publish fits the Split column without scrolling.
      flowchart: { minNodeWidth: 0 },
      suppressErrorRendering: true,
    });
    const id = `kanso-mermaid-${++counter}`;
    try {
      return (await mermaid.render(id, code)).svg;
    } finally {
      // A failed render can leave its scratch element in the page.
      document.getElementById(id)?.remove();
      document.getElementById(`d${id}`)?.remove();
    }
  };
  // mermaid renders one diagram at a time.
  const result = queue.then(run, run);
  queue = result.catch(() => undefined);
  return result;
}

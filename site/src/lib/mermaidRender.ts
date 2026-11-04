/**
 * Draws a ```mermaid fence as an SVG diagram in the demo's own colours.
 * mermaid is large, so it is fetched with a dynamic import the first time a
 * diagram is on screen and never sooner. Strict security level, no HTML
 * labels: visitors can edit the text, so a diagram can't carry markup or
 * click handlers.
 */

let loading: Promise<typeof import("mermaid").default> | null = null;
let queue: Promise<unknown> = Promise.resolve();
let counter = 0;

function loadMermaid() {
  loading ??= import("mermaid").then((m) => m.default);
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

/** Resolves with the diagram's SVG; rejects when mermaid can't parse or draw it. */
export function renderMermaid(code: string, dark: boolean): Promise<string> {
  const run = async () => {
    const mermaid = await loadMermaid();
    mermaid.initialize({
      startOnLoad: false,
      securityLevel: "strict",
      theme: "base",
      themeVariables: themeVariables(dark),
      flowchart: { htmlLabels: false },
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

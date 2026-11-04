/**
 * A note as the app's preview shows it: rendered Markdown where checkboxes
 * tick the note, [[wiki-links]] open the note they name, and code blocks can
 * be copied. Headings carry ids so quick open can scroll to them.
 */
import { Check, Copy } from "lucide-react";
import { type ReactNode, useEffect, useState, useSyncExternalStore } from "react";
import { renderMermaid } from "../../lib/mermaidRender";
import { languageFor, scopeClass, tokenize } from "../../lib/codeHighlight";
import { type Block, type Inline, parse } from "../../lib/markdown";
import { useHighlighter } from "./useHighlighter";

export interface PreviewProps {
  source: string;
  onToggleTask: (line: number) => void;
  onOpenWiki: (target: string) => void;
}

export function Preview({ source, onToggleTask, onOpenWiki }: PreviewProps) {
  const blocks = parse(source);
  return (
    <div className="prose-note" data-testid="preview">
      {blocks.map((block, i) => (
        <BlockView key={i} block={block} onToggleTask={onToggleTask} onOpenWiki={onOpenWiki} />
      ))}
    </div>
  );
}

function BlockView({ block, onToggleTask, onOpenWiki }: { block: Block } & Omit<PreviewProps, "source">) {
  const inl = (c: readonly Inline[]) => <Inlines c={c} onOpenWiki={onOpenWiki} />;
  switch (block.t) {
    case "heading": {
      const Tag = (["h1", "h2", "h3", "h4", "h5", "h6"] as const)[block.level - 1] ?? "h6";
      return <Tag id={`note-${block.id}`}>{inl(block.c)}</Tag>;
    }
    case "paragraph":
      return <p>{inl(block.c)}</p>;
    case "quote":
      return <blockquote>{inl(block.c)}</blockquote>;
    case "hr":
      return <hr />;
    case "code":
      if (block.lang.toLowerCase() === "mermaid") return <MermaidBlock code={block.v} />;
      return <CodeBlock lang={block.lang} code={block.v} />;
    case "table":
      return (
        <table>
          <thead>
            <tr>
              {block.head.map((cell, i) => (
                <th key={i}>{inl(cell)}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {block.rows.map((row, r) => (
              <tr key={r}>
                {row.map((cell, i) => (
                  <td key={i}>{inl(cell)}</td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      );
    case "list": {
      const List = block.ordered ? "ol" : "ul";
      return (
        <List className={block.ordered ? "list-decimal pl-6" : "grid gap-1"}>
          {block.items.map((item) => (
            <li
              key={item.line}
              style={{ marginLeft: `${item.depth * 1.4}rem` }}
              className={block.ordered ? "" : "flex items-start gap-2.5"}
            >
              {item.checked === null ? (
                !block.ordered && <span aria-hidden="true" className="mt-[0.62em] size-[5px] shrink-0 rounded-full bg-fg-3" />
              ) : (
                <button
                  type="button"
                  role="checkbox"
                  aria-checked={item.checked}
                  aria-label={item.checked ? "Mark as not done" : "Mark as done"}
                  onClick={() => onToggleTask(item.line)}
                  className={
                    "mt-[0.28em] grid size-[1.05em] shrink-0 cursor-pointer place-items-center rounded-full border transition-colors " +
                    (item.checked ? "border-accent bg-accent text-btn-fg" : "border-fg-3 hover:border-accent")
                  }
                >
                  {item.checked && <Check className="size-[0.7em]" strokeWidth={3.5} />}
                </button>
              )}
              <span className={item.checked ? "text-fg-3 line-through" : undefined}>{inl(item.c)}</span>
            </li>
          ))}
        </List>
      );
    }
  }
}

const DARK_QUERY = "(prefers-color-scheme: dark)";

/** Whether the page is in its dark theme; re-renders its caller when that changes. */
function useDark(): boolean {
  return useSyncExternalStore(
    (notify) => {
      const query = window.matchMedia?.(DARK_QUERY);
      query?.addEventListener("change", notify);
      return () => query?.removeEventListener("change", notify);
    },
    () => window.matchMedia?.(DARK_QUERY).matches ?? false,
    () => false,
  );
}

/**
 * A mermaid diagram. The code block shows while mermaid loads and whenever it
 * can't draw the source, so there is never an error box or an empty gap.
 */
function MermaidBlock({ code }: { code: string }) {
  const dark = useDark();
  const [drawn, setDrawn] = useState<{ code: string; svg: string } | null>(null);
  useEffect(() => {
    let current = true;
    renderMermaid(code, dark).then(
      (svg) => current && setDrawn({ code, svg }),
      () => current && setDrawn(null),
    );
    return () => {
      current = false;
    };
  }, [code, dark]);
  // Only a drawing of this exact source is shown; a theme change keeps the old drawing up until the new one is ready.
  if (!drawn || drawn.code !== code) return <CodeBlock lang="mermaid" code={code} />;
  return (
    <div
      role="img"
      aria-label="Mermaid diagram"
      data-testid="mermaid-diagram"
      className="mermaid-diagram"
      dangerouslySetInnerHTML={{ __html: drawn.svg }}
    />
  );
}

function CodeBlock({ lang, code }: { lang: string; code: string }) {
  const [copied, setCopied] = useState(false);
  useHighlighter(languageFor(lang) !== null);
  const copy = () => {
    navigator.clipboard?.writeText(code).then(
      () => {
        setCopied(true);
        setTimeout(() => setCopied(false), 1600);
      },
      () => undefined,
    );
  };
  return (
    <div className="group relative">
      {/* Room above the code for the language label, so it never sits on the first line. */}
      <pre className={lang ? "pt-[2.1em]" : undefined}>
        <code>
          {tokenize(code, lang).map((token, i) => (
            <span key={i} className={scopeClass(token.scope)}>
              {token.text}
            </span>
          ))}
        </code>
      </pre>
      <div className="absolute top-1.5 right-1.5 flex items-center gap-2 font-[family-name:var(--font-ui)] text-[11px] text-fg-3">
        {lang && <span>{lang}</span>}
        <button
          type="button"
          onClick={copy}
          aria-label={copied ? "Copied" : `Copy the ${lang || "code"}`}
          className="grid size-6 place-items-center rounded-md bg-elevated/80 text-fg-2 opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
        >
          {copied ? <Check className="size-3.5" /> : <Copy className="size-3.5" />}
        </button>
      </div>
    </div>
  );
}

export function Inlines({ c, onOpenWiki }: { c: readonly Inline[]; onOpenWiki: (target: string) => void }): ReactNode {
  return c.map((i, k) => {
    switch (i.t) {
      case "text":
        return i.v;
      case "code":
        return <code key={k}>{i.v}</code>;
      case "kbd":
        return (
          <kbd key={k} className="kbd">
            {i.v}
          </kbd>
        );
      case "strong":
        return (
          <strong key={k} className="font-semibold">
            <Inlines c={i.c} onOpenWiki={onOpenWiki} />
          </strong>
        );
      case "em":
        return (
          <em key={k}>
            <Inlines c={i.c} onOpenWiki={onOpenWiki} />
          </em>
        );
      case "del":
        return (
          <del key={k} className="text-fg-3">
            <Inlines c={i.c} onOpenWiki={onOpenWiki} />
          </del>
        );
      case "link": {
        const external = /^https?:/.test(i.href);
        return (
          <a key={k} href={i.href} {...(external ? { target: "_blank", rel: "noreferrer" } : {})}>
            <Inlines c={i.c} onOpenWiki={onOpenWiki} />
          </a>
        );
      }
      case "wiki":
        return (
          <button
            key={k}
            type="button"
            onClick={() => onOpenWiki(i.target)}
            className="cursor-pointer rounded-[3px] text-link underline decoration-dotted decoration-1 underline-offset-[0.2em] hover:bg-link/10"
          >
            {i.label}
          </button>
        );
      case "tag":
        return (
          <span key={k} className="rounded-full bg-str/12 px-1.5 py-px text-[0.85em] text-str">
            #{i.v}
          </span>
        );
      case "date":
        return (
          <span key={k} className="rounded-[4px] bg-kw/12 px-1.5 py-px text-[0.85em] text-kw">
            {i.v}
          </span>
        );
    }
  });
}

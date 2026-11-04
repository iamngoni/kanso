/**
 * Tests for the Preview's mermaid blocks, with mermaid mocked: a diagram
 * container once it has drawn, the code block while it loads and when the
 * source can't be parsed, and strict security with no HTML labels.
 */
import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mermaid = vi.hoisted(() => ({ initialize: vi.fn(), render: vi.fn() }));
vi.mock("mermaid", () => ({ default: mermaid }));

import { Preview } from "./Preview";

const FLOW = "```mermaid\nflowchart LR\n  Draft --> Review --> Publish\n```\n\nAfter the diagram.";
const show = (source: string) => render(<Preview source={source} onToggleTask={vi.fn()} onOpenWiki={vi.fn()} />);

beforeEach(() => {
  mermaid.initialize.mockReset();
  mermaid.render.mockReset();
});

describe("Preview mermaid blocks", () => {
  it("shows the code block while mermaid loads, then the diagram", async () => {
    mermaid.render.mockResolvedValue({ svg: '<svg data-testid="flow"></svg>' });
    const { container } = show(FLOW);
    expect(container.querySelector("pre")).toHaveTextContent("Draft --> Review --> Publish");
    expect(await screen.findByTestId("mermaid-diagram")).toContainElement(await screen.findByTestId("flow"));
    expect(container.querySelector("pre")).toBeNull();
    expect(screen.getByText("After the diagram.")).toBeInTheDocument();
  });

  it("falls back to the code block when the source doesn't parse", async () => {
    mermaid.render.mockRejectedValue(new Error("Parse error"));
    const { container } = show("```mermaid\nnot a diagram\n```");
    await waitFor(() => expect(mermaid.render).toHaveBeenCalled());
    expect(screen.queryByTestId("mermaid-diagram")).toBeNull();
    expect(container.querySelector("pre")).toHaveTextContent("not a diagram");
  });

  it("draws with strict security and no HTML labels", async () => {
    mermaid.render.mockResolvedValue({ svg: "<svg></svg>" });
    show(FLOW);
    await screen.findByTestId("mermaid-diagram");
    expect(mermaid.initialize).toHaveBeenCalledWith(
      expect.objectContaining({ securityLevel: "strict", htmlLabels: false, secure: ["htmlLabels"], flowchart: { minNodeWidth: 0 } }),
    );
  });

  it("keeps the last drawing up while an edit redraws, then shows the new one", async () => {
    let finish: (v: { svg: string }) => void = () => {};
    mermaid.render.mockResolvedValueOnce({ svg: '<svg data-testid="first"></svg>' });
    const { container, rerender } = show(FLOW);
    await screen.findByTestId("first");
    mermaid.render.mockReturnValueOnce(new Promise((resolve) => (finish = resolve)));
    rerender(<Preview source={FLOW.replace("Publish", "Ship")} onToggleTask={vi.fn()} onOpenWiki={vi.fn()} />);
    await waitFor(() => expect(mermaid.render).toHaveBeenCalledTimes(2));
    expect(screen.getByTestId("first")).toBeInTheDocument();
    expect(container.querySelector("pre")).toBeNull();
    finish({ svg: '<svg data-testid="second"></svg>' });
    expect(await screen.findByTestId("second")).toBeInTheDocument();
  });

  it("skips drawings the text has already moved past", async () => {
    let finish: (v: { svg: string }) => void = () => {};
    mermaid.render.mockReturnValueOnce(new Promise((resolve) => (finish = resolve)));
    mermaid.render.mockResolvedValue({ svg: '<svg data-testid="latest"></svg>' });
    const { rerender } = show(FLOW);
    await waitFor(() => expect(mermaid.render).toHaveBeenCalledTimes(1));
    // Two edits queue behind the drawing in progress; only the newest is drawn.
    for (const name of ["Ship", "Release"]) {
      rerender(<Preview source={FLOW.replace("Publish", name)} onToggleTask={vi.fn()} onOpenWiki={vi.fn()} />);
    }
    finish({ svg: "<svg></svg>" });
    await screen.findByTestId("latest");
    expect(mermaid.render).toHaveBeenCalledTimes(2);
    expect(mermaid.render.mock.calls[1]?.[1]).toContain("Release");
  });

  it("leaves other fences as code blocks", () => {
    const { container } = show("```js\nlet a = 1\n```");
    expect(container.querySelector("pre")).not.toBeNull();
    expect(mermaid.render).not.toHaveBeenCalled();
  });
});

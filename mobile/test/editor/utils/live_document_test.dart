// Kanso — live_document_test.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/painting.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/editor/utils/live_document/live_display.dart';
import 'package:kanso/editor/utils/live_document/live_layout_data.dart';
import 'package:kanso/editor/utils/live_document/live_rows.dart';
import 'package:kanso/note/views/widgets/preview/preview_list.dart';
import 'package:kanso/note/views/widgets/preview/preview_style.dart';

PreviewStyle get style => PreviewStyle(
  palette: KansoPalette.kansoLight.preview,
  typography: KansoTheme.light.typography,
);

TextRange r(int start, int end) => TextRange(start: start, end: end);

void main() {
  group('LiveLayoutData.shiftedTo', () {
    test('a block takes in what is typed at its end; a mark does not', () {
      const text = 'a **b**\n\nc';
      final layout = LiveLayoutData(
        text: text,
        version: 1,
        conceal: <TextRange>[r(2, 4), r(5, 7)],
        blocks: <LiveBlockData>[
          LiveBlockData(kind: LiveBlockKind.paragraph, lines: r(0, 7)),
          LiveBlockData(kind: LiveBlockKind.blank, lines: r(8, 8)),
          LiveBlockData(kind: LiveBlockKind.paragraph, lines: r(9, 10)),
        ],
      );
      final moved = layout.shiftedTo('a **b**x\n\nc');
      expect(moved.blocks[0].lines, r(0, 8));
      expect(moved.blocks[2].lines, r(10, 11));
      expect(moved.conceal, <TextRange>[r(2, 4), r(5, 7)]);
      // Typed inside a mark: it ends where the change starts.
      final inside = layout.shiftedTo('a *x*b**\n\nc');
      expect(inside.blocks[0].lines, r(0, 8));
    });
  });

  group('buildLiveDisplay', () {
    test('hides the marks and maps offsets both ways', () {
      const text = 'a **bold** c';
      final layout = LiveLayoutData(
        text: text,
        version: 1,
        conceal: <TextRange>[r(2, 4), r(8, 10)],
        styles: <LiveStyleData>[
          LiveStyleData(range: r(4, 8), kind: LiveStyleKind.strong),
        ],
        blocks: <LiveBlockData>[
          LiveBlockData(kind: LiveBlockKind.paragraph, lines: r(0, 12)),
        ],
      );
      final row = LiveRow(kind: LiveRowKind.text, range: r(0, 12));
      final shown = buildLiveDisplay(layout: layout, row: row, style: style);
      expect(shown.span.toPlainText(), 'a bold c');
      expect(shown.length, 8);
      // Into the bold word, past its hidden marks, and back.
      expect(shown.toDisplay(4), 2);
      expect(shown.toSource(2), 4);
      expect(shown.toDisplay(9), 6);
      expect(shown.toSource(6), 8);
      expect(shown.toSource(8), 12);
    });

    test('the :: of a flashcard line is drawn in the muted colour', () {
      const text = 'Q :: A';
      final layout = LiveLayoutData(
        text: text,
        version: 1,
        styles: <LiveStyleData>[
          LiveStyleData(range: r(2, 4), kind: LiveStyleKind.cardSeparator),
        ],
        blocks: <LiveBlockData>[
          LiveBlockData(kind: LiveBlockKind.paragraph, lines: r(0, 6)),
        ],
      );
      final row = LiveRow(kind: LiveRowKind.text, range: r(0, 6));
      final shown = buildLiveDisplay(layout: layout, row: row, style: style);
      expect(shown.span.toPlainText(), text, reason: 'nothing is hidden');
      final colours = <String, Color?>{};
      shown.span.visitChildren((span) {
        if (span is TextSpan && span.text != null) {
          colours[span.text!] = span.style?.color;
        }
        return true;
      });
      expect(colours['::'], style.muted);
      expect(colours['Q '], isNot(style.muted));
    });

    test('a line break in a paragraph is a space off the caret', () {
      const text = 'one\ntwo';
      final layout = LiveLayoutData(
        text: text,
        version: 1,
        blocks: <LiveBlockData>[
          LiveBlockData(kind: LiveBlockKind.paragraph, lines: r(0, 7)),
        ],
      );
      final off = buildLiveDisplay(
        layout: layout,
        row: LiveRow(kind: LiveRowKind.text, range: r(0, 7)),
        style: style,
      );
      expect(off.span.toPlainText(), 'one two');
      final on = buildLiveDisplay(
        layout: layout,
        row: LiveRow(kind: LiveRowKind.text, range: r(0, 7), revealed: true),
        style: style,
      );
      expect(on.span.toPlainText(), 'one\ntwo');
    });
  });

  group('planLiveRows', () {
    test('spaces blocks as the preview does', () {
      const text = '# A\n## B\nPara\n\n- one\n- two';
      final layout = LiveLayoutData(
        text: text,
        version: 1,
        blocks: <LiveBlockData>[
          LiveBlockData(
            kind: LiveBlockKind.heading,
            level: 1,
            lines: r(0, 3),
            top: 0,
          ),
          LiveBlockData(
            kind: LiveBlockKind.heading,
            level: 2,
            lines: r(4, 8),
            top: 1,
          ),
          LiveBlockData(kind: LiveBlockKind.paragraph, lines: r(9, 13), top: 2),
          LiveBlockData(kind: LiveBlockKind.blank, lines: r(14, 14)),
          LiveBlockData(
            kind: LiveBlockKind.paragraph,
            lines: r(15, 20),
            listDepth: 1,
            itemStart: true,
            top: 3,
          ),
          LiveBlockData(
            kind: LiveBlockKind.paragraph,
            lines: r(21, 26),
            listDepth: 1,
            itemStart: true,
            top: 3,
          ),
        ],
        marks: <LiveMarkData>[
          LiveMarkData(range: r(15, 17), kind: LiveMarkKind.bullet, text: '•'),
          LiveMarkData(range: r(21, 23), kind: LiveMarkKind.bullet, text: '•'),
        ],
      );
      final rows = planLiveRows(layout: layout, style: style);
      final size = style.size;
      final (h1, _) = PreviewStyle.headingMetrics(1);
      final (h2, _) = PreviewStyle.headingMetrics(2);
      expect(rows[0].above, 0);
      // A heading right after a heading: its own room above, or the first
      // heading's below, whichever is more, as in the preview.
      expect(
        rows[1].above,
        closeTo(math.max(0.25 * h2 * size, 0.35 * h1 * size), 1e-9),
      );
      // A paragraph after a heading: the heading's room below.
      expect(rows[2].above, closeTo(0.35 * h2 * size, 1e-9));
      expect(rows[3].kind, LiveRowKind.hidden);
      // A list after a paragraph: the paragraph's gap.
      expect(rows[4].above, closeTo(style.gap, 1e-9));
      // A tight list's items.
      expect(
        rows[5].above,
        closeTo(PreviewList.itemGap(style, tight: true), 1e-9),
      );
      expect(rows[4].indent, PreviewList.markerWidth(style));
      expect(rows[4].marker?.text, '•');
    });
  });
}

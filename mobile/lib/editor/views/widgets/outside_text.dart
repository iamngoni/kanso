// Kanso — outside_text.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart' show EditorRun, Range16;
import '../../../core/utils/context_extensions.dart';
import '../../../note/views/widgets/note_source_text.dart';
import '../../models/data/outside_side.dart';
import '../../utils/editor_spans.dart';
import '../controllers/kanso_markdown_controller.dart';

/// The part of a long note that is before or after the editor's window,
/// drawn read-only in the editor's font and colours, a line at a time as it
/// scrolls into view.
///
/// A tap puts the caret there: the window moves to it, and [onTapAt] says
/// first where on the screen the tap was, for the page to keep that place.
/// Read by a screen reader as plain lines of text.
///
/// The lines are laid out from the one nearest the editor outward, which is
/// what a scroll view does with the slivers before its centre ([reverse],
/// for the text before the window): a part of the note far from where the
/// page is scrolled to costs nothing.
///
/// It has no height when the window is the whole note.
class OutsideText extends StatefulWidget {
  const OutsideText({
    required this.controller,
    required this.side,
    this.reverse = false,
    this.onTapAt,
    super.key,
  });

  final KansoMarkdownController controller;
  final OutsideSide side;

  /// Whether the first item is the last line: for a sliver before the
  /// centre of the scroll view.
  final bool reverse;

  /// Called with the screen's y of a tap, before the window moves to it.
  final ValueChanged<double>? onTapAt;

  @override
  State<OutsideText> createState() => _OutsideTextState();
}

class _OutsideTextState extends State<OutsideText> {
  // The text the lines were cut from, and where each starts.
  String? source;
  List<int> starts = const <int>[0];

  /// The side's text: without the line break that joins it to the window.
  String sideText() {
    final text = widget.side == OutsideSide.before
        ? widget.controller.before
        : widget.controller.after;
    if (text.isEmpty) {
      return '';
    }
    return widget.side == OutsideSide.before
        ? text.substring(0, text.length - 1)
        : text.substring(1);
  }

  void cut(String text) {
    final found = <int>[0];
    var at = text.indexOf('\n');
    while (at >= 0) {
      found.add(at + 1);
      at = text.indexOf('\n', at + 1);
    }
    starts = found;
    source = text;
  }

  /// The note's offset of the first unit of the side's text.
  int base() => widget.side == OutsideSide.before
      ? 0
      : widget.controller.windowStart + widget.controller.text.length + 1;

  @override
  Widget build(BuildContext context) {
    final controller = widget.controller;
    final side = widget.side;
    final syntax = context.syntaxColors;
    final style = context.typography.editorBody.copyWith(color: syntax.text);
    final blank =
        MediaQuery.textScalerOf(context).scale(style.fontSize ?? 15) *
        (style.height ?? 1.4);
    return ListenableBuilder(
      listenable: controller.outsideChanged,
      builder: (context, _) {
        final text = sideText();
        if (text.isEmpty &&
            (side == OutsideSide.before
                ? controller.before.isEmpty
                : controller.after.isEmpty)) {
          return const SliverToBoxAdapter(child: SizedBox.shrink());
        }
        if (!identical(text, source) && text != source) {
          cut(text);
        }
        final lines = starts;
        return SliverList.builder(
          itemCount: lines.length,
          itemBuilder: (context, item) {
            final at = widget.reverse ? lines.length - 1 - item : item;
            final start = lines[at];
            final end = at + 1 < lines.length ? lines[at + 1] - 1 : text.length;
            Widget line;
            if (end <= start) {
              line = SizedBox(height: blank);
            } else {
              final runs = controller.outsideRunsOf(side, start, end);
              // As the field draws it: the style as it is, not merged with
              // the page's default.
              line = RichText(
                textScaler: MediaQuery.textScalerOf(context),
                text: editorSpans(
                  text: text.substring(start, end),
                  runs: <EditorRun>[
                    for (final run in runs)
                      EditorRun(
                        range: Range16(
                          start: run.range.start - start,
                          end: run.range.end - start,
                        ),
                        color: run.color,
                        italic: run.italic,
                        weight: run.weight,
                        quiet: run.quiet,
                      ),
                  ],
                  base: style,
                ),
              );
            }
            return Builder(
              builder: (lineContext) => GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTapUp: (tap) {
                  var inLine = 0;
                  if (end > start) {
                    final paragraph = NoteSourceText.paragraphIn(lineContext);
                    if (paragraph != null) {
                      inLine = paragraph
                          .getPositionForOffset(tap.localPosition)
                          .offset;
                    }
                  }
                  widget.onTapAt?.call(tap.globalPosition.dy);
                  controller.placeCaret(base() + start + inLine);
                },
                child: line,
              ),
            );
          },
        );
      },
    );
  }
}

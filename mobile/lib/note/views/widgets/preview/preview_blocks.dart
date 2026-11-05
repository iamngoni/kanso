// Kanso — preview_blocks.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../../canvas/views/widgets/sketch_block.dart';
import '../../../../core/bridge/api/types.dart';
import '../../../state/preview/preview_bloc.dart';
import 'preview_code.dart';
import 'preview_figure.dart';
import 'preview_list.dart';
import 'preview_style.dart';
import 'preview_table.dart';
import 'preview_text.dart';

/// Blocks of a note one below the other, spaced as the page spaces them:
/// the room between two blocks is the larger of the first's below and the
/// second's above, as margins fold together on the page. In a [tight] list
/// item, paragraphs have no room below them.
class PreviewBlocks extends StatelessWidget {
  const PreviewBlocks(this.blocks, {this.tight = false, super.key});

  final List<PreviewBlock> blocks;
  final bool tight;

  /// The room above [block] and below it, in points.
  static (double above, double below) margins(
    PreviewBlock block,
    PreviewStyle style, {
    bool tight = false,
  }) {
    return switch (block) {
      PreviewBlock_Heading(:final level) => room(
        PreviewRoom.heading,
        style,
        level: level,
      ),
      PreviewBlock_Rule() => room(PreviewRoom.rule, style),
      PreviewBlock_Paragraph() || PreviewBlock_Html() => room(
        PreviewRoom.text,
        style,
        tight: tight,
      ),
      PreviewBlock_Figure(kind: PreviewFigureKind.math) => room(
        PreviewRoom.math,
        style,
      ),
      _ => room(PreviewRoom.other, style),
    };
  }

  /// The room above a block of [kind] and below it, in points: a heading of
  /// [level]; text in a [tight] list item has none.
  static (double above, double below) room(
    PreviewRoom kind,
    PreviewStyle style, {
    int level = 1,
    bool tight = false,
  }) {
    final size = style.size;
    return switch (kind) {
      PreviewRoom.heading => () {
        final (ratio, gap) = PreviewStyle.headingMetrics(level);
        return (gap * ratio * size, 0.35 * ratio * size);
      }(),
      PreviewRoom.rule => (1.6 * size, 1.6 * size),
      PreviewRoom.text when tight => (0.0, 0.0),
      PreviewRoom.math => (0.4 * size, math.max(0.4 * size, style.gap)),
      _ => (0.0, style.gap),
    };
  }

  /// The room between [before] and [block] below it, in points.
  static double between(
    PreviewBlock before,
    PreviewBlock block,
    PreviewStyle style, {
    bool tight = false,
  }) {
    final (_, below) = margins(before, style, tight: tight);
    var (above, _) = margins(block, style, tight: tight);
    if (before is PreviewBlock_Heading && block is PreviewBlock_Heading) {
      above = headingAfterHeading(block.level, style);
    }
    return math.max(above, below);
  }

  /// The room above a heading of [level] right after another heading.
  static double headingAfterHeading(int level, PreviewStyle style) {
    final (ratio, _) = PreviewStyle.headingMetrics(level);
    return 0.25 * ratio * style.size;
  }

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    final children = <Widget>[];
    for (var at = 0; at < blocks.length; at++) {
      final block = blocks[at];
      if (at > 0) {
        final gap = between(blocks[at - 1], block, style, tight: tight);
        if (gap > 0) {
          children.add(SizedBox(height: gap));
        }
      }
      children.add(PreviewBlockView(block, tight: tight));
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: children,
    );
  }
}

/// What a block is to the room around it ([PreviewBlocks.room]).
enum PreviewRoom {
  heading,
  rule,

  /// A paragraph or HTML: none in a tight list item.
  text,

  /// A formula on a line of its own.
  math,
  other,
}

/// One block of a note.
class PreviewBlockView extends StatelessWidget {
  const PreviewBlockView(this.block, {this.tight = false, super.key});

  final PreviewBlock block;
  final bool tight;

  @override
  Widget build(BuildContext context) {
    final scope = PreviewScope.of(context);
    final style = scope.style;
    return switch (block) {
      final PreviewBlock_Heading heading => Semantics(
        header: true,
        child: PreviewText(
          heading.inlines,
          key: scope.headingKey(heading),
          style: style.heading(heading.level),
        ),
      ),
      PreviewBlock_Paragraph(:final inlines) => PreviewText(inlines),
      PreviewBlock_List(
        :final ordered,
        :final start,
        :final tight,
        :final items,
      ) =>
        PreviewList(ordered: ordered, start: start, tight: tight, items: items),
      PreviewBlock_Quote(:final blocks) => _Quote(blocks),
      PreviewBlock_Code(:final language, :final runs, :final copy) =>
        PreviewCode(language: language, runs: runs, copy: copy),
      PreviewBlock_Rule() => Container(height: 1, color: style.border),
      PreviewBlock_Table(:final aligns, :final head, :final rows) =>
        PreviewTable(aligns: aligns, head: head, rows: rows),
      PreviewBlock_Figure(:final kind, :final svg) => PreviewFigure(
        kind: kind,
        svg: svg,
      ),
      PreviewBlock_Sketch(
        :final id,
        :final width,
        :final height,
        :final svg,
      ) =>
        _sketch(context, id, width, height, svg),
      PreviewBlock_EmptyDrawing() => const PreviewEmptyDrawing(),
      PreviewBlock_Failed(
        :final badge,
        :final detail,
        :final language,
        :final code,
        :final copy,
      ) =>
        PreviewFailed(
          badge: badge,
          detail: detail,
          language: language,
          code: code,
          copy: copy,
        ),
      PreviewBlock_Query(:final columns, :final rows) => PreviewQuery(
        columns: columns,
        rows: rows,
      ),
      PreviewBlock_QueryInvalid(:final code, :final message) =>
        PreviewQueryInvalid(code: code, message: message),
      PreviewBlock_Html(:final text) => Text(text, style: style.body),
    };
  }
}

/// An inline sketch: drawn on in place when the preview belongs to a note
/// (it has a [PreviewBloc]), a picture otherwise.
Widget _sketch(
  BuildContext context,
  String id,
  int width,
  int height,
  PreviewSvg svg,
) {
  String? noteId;
  try {
    noteId = context.read<PreviewBloc>().state.noteId;
  } on Object {
    noteId = null;
  }
  if (noteId == null || noteId.isEmpty) {
    return PreviewFigure(kind: PreviewFigureKind.drawing, svg: svg);
  }
  return SketchBlock(
    key: ValueKey<String>('sketch-$noteId-$id'),
    noteId: noteId,
    id: id,
    width: width,
    height: height,
    svg: svg,
  );
}

/// A quote: its blocks beside a bar, in the quote colour.
class _Quote extends StatelessWidget {
  const _Quote(this.blocks);

  final List<PreviewBlock> blocks;

  @override
  Widget build(BuildContext context) {
    final style = PreviewScope.of(context).style;
    return Container(
      padding: const EdgeInsets.only(left: PreviewStyle.quotePadding),
      decoration: BoxDecoration(
        border: Border(
          left: BorderSide(
            color: style.quoteBar,
            width: PreviewStyle.quoteBarWidth,
          ),
        ),
      ),
      child: PreviewScope.nested(
        context,
        style: style.withText(style.quote),
        child: PreviewBlocks(blocks),
      ),
    );
  }
}

// Kanso — preview_list.dart
// Created 2026-10-30.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../../../core/bridge/api/types.dart';
import '../../../../core/configs/kanso_icons.dart';
import '../../../../core/utils/context_extensions.dart';
import '../../../../core/views/widgets/kanso_icon.dart';
import 'preview_blocks.dart';
import 'preview_list_marker.dart';
import 'preview_style.dart';

/// A list: bullets, or numbers from [start], or task boxes, each item's
/// blocks beside its marker. A [tight] list has no paragraph gaps.
///
/// Each list in an item is set in by a marker's width, until the room left
/// beside a marker would be less than [minTextWidth] body sizes: from there
/// a list in an item is drawn below the item's text, as far in as the item,
/// so lists nested as deep as the core allows still fit.
class PreviewList extends StatelessWidget {
  const PreviewList({
    required this.ordered,
    required this.start,
    required this.tight,
    required this.items,
    super.key,
  });

  final bool ordered;
  final int start;
  final bool tight;
  final List<PreviewListItem> items;

  /// The narrowest room, in body sizes, left beside a marker before nested
  /// lists stop being set in further.
  static const double minTextWidth = 8;

  static const List<String> _bullets = <String>['•', '◦', '▪'];

  /// The bullet of an item [depth] lists deep.
  static String bulletAt(int depth) => _bullets[depth % _bullets.length];

  /// The room a bullet or a number takes beside its item's text.
  static double markerWidth(PreviewStyle style) => 1.4 * style.size;

  /// The room between two items of a list, [tight] or not.
  static double itemGap(PreviewStyle style, {required bool tight}) =>
      tight ? 0.25 * style.size : math.max(0.25 * style.size, style.gap);

  @override
  Widget build(BuildContext context) {
    final scope = PreviewScope.of(context);
    final style = scope.style;
    final size = style.size;
    final between = itemGap(style, tight: tight);
    final bulletWidth = markerWidth(style);
    return LayoutBuilder(
      builder: (context, constraints) {
        final widest = math.max(bulletWidth, PreviewTaskCheckbox.width);
        final flat = constraints.maxWidth - widest < minTextWidth * size;
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            for (var at = 0; at < items.length; at++) ...<Widget>[
              if (at > 0) SizedBox(height: between),
              ..._item(
                context,
                items[at],
                flat: flat,
                markerWidth: items[at].task == null
                    ? bulletWidth
                    : PreviewTaskCheckbox.width,
                marker: switch (items[at].task) {
                  final task? => PreviewTaskCheckbox(
                    checked: task.checked,
                    index: task.index,
                  ),
                  null => PreviewListMarker(
                    ordered ? '${start + at}.' : bulletAt(scope.depth),
                  ),
                },
              ),
            ],
          ],
        );
      },
    );
  }

  /// The item's blocks beside its [marker]; when [flat], the lists in it
  /// below its text instead, as far in as the item.
  List<Widget> _item(
    BuildContext context,
    PreviewListItem item, {
    required bool flat,
    required double markerWidth,
    required Widget marker,
  }) {
    final scope = PreviewScope.of(context);
    final style = scope.style;
    Widget nested(List<PreviewBlock> blocks) => PreviewTaskPlace(
      index: item.task?.index,
      child: PreviewScope.nested(
        context,
        depth: scope.depth + 1,
        style: item.task?.checked ?? false
            ? style.withText(style.taskDone)
            : null,
        child: PreviewBlocks(blocks, tight: tight),
      ),
    );
    Widget beside(Widget lead, List<PreviewBlock> blocks) => Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        lead,
        Expanded(child: nested(blocks)),
      ],
    );
    if (!flat) {
      return <Widget>[beside(marker, item.blocks)];
    }
    // Runs of the item's blocks: text beside the marker, each list below.
    final runs = <List<PreviewBlock>>[];
    for (final block in item.blocks) {
      if (block is! PreviewBlock_List &&
          runs.isNotEmpty &&
          runs.last.last is! PreviewBlock_List) {
        runs.last.add(block);
      } else {
        runs.add(<PreviewBlock>[block]);
      }
    }
    final children = <Widget>[];
    PreviewBlock? before;
    for (final run in runs) {
      final list = run.first is PreviewBlock_List;
      if (children.isEmpty && list) {
        // A marker with no text of its own, the list below it.
        children.add(beside(marker, const <PreviewBlock>[]));
      }
      if (before != null) {
        final (_, below) = PreviewBlocks.margins(before, style, tight: tight);
        final (above, _) = PreviewBlocks.margins(
          run.first,
          style,
          tight: tight,
        );
        final room = math.max(above, below);
        if (room > 0) {
          children.add(SizedBox(height: room));
        }
      }
      children.add(
        list
            ? nested(run)
            : beside(
                children.isEmpty ? marker : SizedBox(width: markerWidth),
                run,
              ),
      );
      before = run.last;
    }
    return children;
  }
}

/// A task's box, ticked or not, as the page draws it. A box with an
/// [index] can be tapped, which asks to tick it (`kanso-task:<index>`); one
/// without can't.
class PreviewTaskCheckbox extends StatelessWidget {
  const PreviewTaskCheckbox({required this.checked, this.index, super.key});

  final bool checked;
  final int? index;

  /// The room a task's box takes beside its text.
  static const double width = 15 + 8 + 8;

  @override
  Widget build(BuildContext context) {
    final scope = PreviewScope.of(context);
    final style = scope.style;
    final line =
        MediaQuery.textScalerOf(context).scale(style.size) *
        (style.body.height ?? 1.45);
    final box = Container(
      width: 15,
      height: 15,
      decoration: BoxDecoration(
        color: checked ? style.accent : null,
        border: Border.all(
          color: checked ? style.accent : style.taskDone,
          width: 1.5,
        ),
        borderRadius: BorderRadius.circular(4),
      ),
      alignment: Alignment.center,
      child: checked
          ? KansoIcon(
              KansoIcons.check,
              size: 12,
              color: style.palette.dark
                  ? const Color(0xFF000000)
                  : const Color(0xFFFFFFFF),
            )
          : null,
    );
    final tappable = index;
    final onTap = tappable == null
        ? null
        : () => scope.onLink('kanso-task:$tappable');
    return Semantics(
      container: true,
      label: context.l10n.previewTask,
      checked: checked,
      enabled: tappable != null,
      button: tappable != null,
      onTap: onTap,
      excludeSemantics: true,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: onTap,
        child: SizedBox(
          width: width,
          height: math.max(line, 15),
          child: Align(alignment: Alignment.centerLeft, child: box),
        ),
      ),
    );
  }
}

/// The place in the note's task list of the task whose text is below it
/// ([index]; null in an item that isn't a task, or a box that can't be
/// ticked from here): what a due chip on that line names when it asks to
/// stop repeating.
class PreviewTaskPlace extends InheritedWidget {
  const PreviewTaskPlace({
    required this.index,
    required super.child,
    super.key,
  });

  final int? index;

  /// The task place around [context], or null outside a task item.
  static int? of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<PreviewTaskPlace>()?.index;

  @override
  bool updateShouldNotify(PreviewTaskPlace oldWidget) =>
      index != oldWidget.index;
}

// Kanso — kanso_large_title.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';

/// A page's title, large and at the start, as the first thing in what
/// scrolls, with an optional quiet [subtitle] under it. Its page's
/// `KansoTopBar` shows the small title once this one has scrolled away: see
/// [KansoLargeTitleScroll].
class KansoLargeTitle extends StatelessWidget {
  const KansoLargeTitle({
    required this.title,
    this.subtitle,
    this.padding = const EdgeInsetsDirectional.fromSTEB(
      KansoSpacing.md,
      KansoSpacing.xxs,
      KansoSpacing.md,
      KansoSpacing.sm,
    ),
    super.key,
  });

  final String title;
  final String? subtitle;
  final EdgeInsetsGeometry padding;

  /// How far a page scrolls before the bar takes the title over, in points.
  static const double collapseAt = 36;

  @override
  Widget build(BuildContext context) {
    final typography = context.typography;
    final note = subtitle;
    return Padding(
      padding: padding,
      child: Semantics(
        header: true,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(title, style: typography.largeTitle),
            if (note != null)
              Padding(
                padding: const EdgeInsets.only(top: KansoSpacing.xxs),
                child: Text(note, style: typography.secondary),
              ),
          ],
        ),
      ),
    );
  }
}

/// Listens to the page's scrolling and says whether its [KansoLargeTitle]
/// has gone under the bar: [builder] gets that, to hand to the bar's
/// `titleShown`.
class KansoLargeTitleScroll extends StatefulWidget {
  const KansoLargeTitleScroll({required this.builder, super.key});

  final Widget Function(BuildContext context, ValueListenable<bool> collapsed)
  builder;

  @override
  State<KansoLargeTitleScroll> createState() => _KansoLargeTitleScrollState();
}

class _KansoLargeTitleScrollState extends State<KansoLargeTitleScroll> {
  final ValueNotifier<bool> collapsed = ValueNotifier<bool>(false);

  @override
  void dispose() {
    collapsed.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return NotificationListener<ScrollNotification>(
      onNotification: (notification) {
        // The page's own list, not one nested in it or one that runs
        // sideways.
        if (notification.depth == 0 &&
            notification.metrics.axis == Axis.vertical) {
          collapsed.value =
              notification.metrics.pixels > KansoLargeTitle.collapseAt;
        }
        return false;
      },
      child: widget.builder(context, collapsed),
    );
  }
}

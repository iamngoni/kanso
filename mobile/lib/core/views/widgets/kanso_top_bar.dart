// Kanso — kanso_top_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../../configs/kanso_motion.dart';
import '../../configs/kanso_sizes.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_divider.dart';

/// The bar at the top of a page: [leading], a centred title and [trailing].
///
/// It takes the status bar's height as well, so its colour reaches the top
/// of the screen; the content sits below it.
class KansoTopBar extends StatelessWidget {
  const KansoTopBar({
    this.leading,
    this.title,
    this.subtitle,
    this.trailing = const <Widget>[],
    this.backgroundColor,
    this.bordered = true,
    this.titleShown,
    super.key,
  });

  final Widget? leading;
  final String? title;

  /// A quiet line under the title.
  final String? subtitle;
  final List<Widget> trailing;

  /// The bar's colour; the colour of the page below when null.
  final Color? backgroundColor;

  /// A line along the bottom edge.
  final bool bordered;

  /// Whether the [title] and the line are shown, for a page whose large
  /// title scrolls with it: they fade in once it has gone under the bar.
  /// Always shown when null.
  final ValueListenable<bool>? titleShown;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final name = title;
    final note = subtitle;
    // The bar is 52 points, and taller when the device's text size makes
    // its title (and subtitle) taller than that.
    final scaler = MediaQuery.textScalerOf(context);
    double line(TextStyle style) =>
        scaler.scale(style.fontSize ?? 0) * (style.height ?? 1);
    final height = math.max(
      KansoSizes.barHeight,
      (name == null ? 0 : line(typography.barTitle)) +
          (note == null ? 0 : line(typography.meta)) +
          2 * KansoSpacing.xxs,
    );
    return ColoredBox(
      color: backgroundColor ?? colors.editor,
      child: SafeArea(
        bottom: false,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            SizedBox(
              height: height,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.xxs,
                ),
                child: NavigationToolbar(
                  leading: leading,
                  middle: name == null
                      ? null
                      : _shown(
                          context,
                          Column(
                            mainAxisSize: MainAxisSize.min,
                            children: <Widget>[
                              Text(
                                name,
                                maxLines: 1,
                                overflow: TextOverflow.ellipsis,
                                style: typography.barTitle,
                              ),
                              if (note != null)
                                Text(
                                  note,
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                  style: typography.meta,
                                ),
                            ],
                          ),
                        ),
                  trailing: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: trailing,
                  ),
                  middleSpacing: 0,
                ),
              ),
            ),
            if (bordered) _shown(context, const KansoDivider()),
          ],
        ),
      ),
    );
  }

  /// [child], fading in and out with [titleShown].
  Widget _shown(BuildContext context, Widget child) {
    final shown = titleShown;
    if (shown == null) {
      return child;
    }
    return ValueListenableBuilder<bool>(
      valueListenable: shown,
      child: child,
      // Gone, not just clear, while hidden: the large title is the one a
      // screen reader finds.
      builder: (context, visible, child) => AnimatedSwitcher(
        duration: KansoMotion.resolve(context, KansoMotion.fast),
        child: visible ? child : const SizedBox.shrink(),
      ),
    );
  }
}

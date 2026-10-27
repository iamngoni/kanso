// Kanso — kanso_swipe_scope.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'kanso_swipe_group.dart';

/// Gives the rows of a list their [KansoSwipeGroup].
class KansoSwipeScope extends InheritedWidget {
  const KansoSwipeScope({required this.group, required super.child, super.key});

  final KansoSwipeGroup group;

  /// The group of the list above [context], or null when there is none.
  static KansoSwipeGroup? maybeOf(BuildContext context) {
    return context.getInheritedWidgetOfExactType<KansoSwipeScope>()?.group;
  }

  @override
  bool updateShouldNotify(KansoSwipeScope oldWidget) =>
      group != oldWidget.group;
}

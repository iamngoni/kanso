// Kanso — editor_extras_scope.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../models/data/editor_extras.dart';

/// Puts [extras] in reach of the editor below it.
class EditorExtrasScope extends InheritedWidget {
  const EditorExtrasScope({
    required this.extras,
    required super.child,
    super.key,
  });

  final EditorExtras extras;

  /// The extras in scope, or none.
  static EditorExtras of(BuildContext context) =>
      context.dependOnInheritedWidgetOfExactType<EditorExtrasScope>()?.extras ??
      const NoEditorExtras();

  @override
  bool updateShouldNotify(EditorExtrasScope oldWidget) =>
      !identical(extras, oldWidget.extras);
}

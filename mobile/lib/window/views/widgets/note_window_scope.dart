// Kanso — note_window_scope.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../models/repos/abs/note_window_repository.dart';

/// Hands the screens below the engine's [NoteWindowRepository], so that a
/// note's menu can offer Open in New Window. Without one (a screen in a test,
/// a platform with a single window) the command is not offered.
class NoteWindowScope extends InheritedWidget {
  const NoteWindowScope({
    required this.repository,
    required super.child,
    super.key,
  });

  final NoteWindowRepository repository;

  /// The repository above [context], or null when there is none.
  static NoteWindowRepository? maybeOf(BuildContext context) {
    return context
        .dependOnInheritedWidgetOfExactType<NoteWindowScope>()
        ?.repository;
  }

  /// Whether the device can show a note in a window of its own, and the
  /// scope says so.
  static bool supportedAt(BuildContext context) {
    return maybeOf(context)?.supported ?? false;
  }

  @override
  bool updateShouldNotify(NoteWindowScope oldWidget) =>
      repository != oldWidget.repository;
}

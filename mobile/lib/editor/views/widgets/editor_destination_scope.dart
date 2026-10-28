// Kanso — editor_destination_scope.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../models/data/editor_destination.dart';

/// Takes over where a link in the text leads, for the editors below it.
///
/// Without one, a link goes to the note in the app's own routes. A note's
/// window has no routes of its own: it hands the link to the main window.
class EditorDestinationScope extends InheritedWidget {
  const EditorDestinationScope({
    required this.onDestination,
    required super.child,
    super.key,
  });

  /// Called with the destination, once what was typed is saved.
  final void Function(BuildContext context, EditorDestination destination)
  onDestination;

  /// The scope above [context], or null when there is none.
  static EditorDestinationScope? maybeOf(BuildContext context) {
    return context.getInheritedWidgetOfExactType<EditorDestinationScope>();
  }

  @override
  bool updateShouldNotify(EditorDestinationScope oldWidget) => false;
}

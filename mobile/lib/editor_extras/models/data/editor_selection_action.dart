// Kanso — editor_selection_action.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import 'editor_text_target.dart';

/// An item another feature adds to the editor's selection menu, after Cut,
/// Copy and Paste: the inline assistant's actions.
@immutable
class EditorSelectionAction {
  const EditorSelectionAction({
    required this.id,
    required this.label,
    required this.onPressed,
  });

  /// Stable, for tests: `assist`.
  final String id;
  final String label;
  final void Function(BuildContext context, EditorTextTarget target) onPressed;
}

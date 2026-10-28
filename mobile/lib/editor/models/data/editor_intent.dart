// Kanso — editor_intent.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../../core/bridge/api/types.dart' show FormatKind;

/// What a keyboard shortcut of the note's editor asks for.
sealed class EditorIntent extends Intent {
  const EditorIntent();
}

/// Format the selection: bold or italic.
class FormatIntent extends EditorIntent {
  const FormatIntent(this.format);

  final FormatKind format;
}

/// Save now.
class SaveIntent extends EditorIntent {
  const SaveIntent();
}

/// Indent (Tab) or outdent (Shift-Tab) the lines, or move through a table.
class TabIntent extends EditorIntent {
  const TabIntent({required this.out});

  final bool out;
}

/// Escape: closes the completions, otherwise goes to Preview.
class EscapeIntent extends EditorIntent {
  const EscapeIntent();
}

/// Move through the completions by [delta] rows.
class CompletionStepIntent extends EditorIntent {
  const CompletionStepIntent(this.delta);

  final int delta;
}

/// Choose the row of the completions that is active.
class CompletionChooseIntent extends EditorIntent {
  const CompletionChooseIntent();
}

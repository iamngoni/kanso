// Kanso — combined_editor_extras.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../editor_extras/models/data/editor_extras.dart';

/// Two sets of extras as one, [first]'s keys and actions before [second]'s:
/// the editor reads one `EditorExtras`, and dictation and the inline
/// assistant each bring theirs.
class CombinedEditorExtras implements EditorExtras {
  const CombinedEditorExtras(this.first, this.second);

  final EditorExtras first;
  final EditorExtras second;

  @override
  List<EditorAccessoryExtra> accessoryKeys(EditorTextTarget target) =>
      <EditorAccessoryExtra>[
        ...first.accessoryKeys(target),
        ...second.accessoryKeys(target),
      ];

  @override
  List<EditorSelectionAction> selectionActions(EditorTextTarget target) =>
      <EditorSelectionAction>[
        ...first.selectionActions(target),
        ...second.selectionActions(target),
      ];

  @override
  void release(EditorTextTarget target) {
    first.release(target);
    second.release(target);
  }
}

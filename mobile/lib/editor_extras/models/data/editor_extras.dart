// Kanso — editor_extras.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'editor_accessory_extra.dart';
import 'editor_selection_action.dart';
import 'editor_text_target.dart';

export 'editor_accessory_extra.dart';
export 'editor_selection_action.dart';
export 'editor_text_target.dart';

/// What other features add to the editor: the seam step 6's editor calls,
/// so dictation and the inline assistant never edit the editor's files.
///
/// The editor reads it with `EditorExtrasScope.of(context)`, adds
/// [accessoryKeys] after its own keys in the accessory bar, and
/// [selectionActions] to its selection menu when text is selected.
abstract interface class EditorExtras {
  /// The extra keys for the note [target] edits; empty when none apply.
  List<EditorAccessoryExtra> accessoryKeys(EditorTextTarget target);

  /// The extra selection menu items for [target]'s current selection.
  List<EditorSelectionAction> selectionActions(EditorTextTarget target);

  /// Releases any note-scoped work when the editor goes away or changes.
  void release(EditorTextTarget target);
}

/// No extras: what the editor gets when nothing is in scope (tests).
class NoEditorExtras implements EditorExtras {
  const NoEditorExtras();

  @override
  List<EditorAccessoryExtra> accessoryKeys(EditorTextTarget target) =>
      const <EditorAccessoryExtra>[];

  @override
  List<EditorSelectionAction> selectionActions(EditorTextTarget target) =>
      const <EditorSelectionAction>[];

  @override
  void release(EditorTextTarget target) {}
}

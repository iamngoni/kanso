// Kanso — sketch_editor_extras.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../core/configs/kanso_icons.dart';
import '../../core/utils/context_extensions.dart';
import '../../core/views/widgets/kanso_feedback.dart';
import '../../editor_extras/models/data/editor_extras.dart';
import '../models/repos/abs/canvas_repository.dart';
import '../views/widgets/sketch_size_sheet.dart';

/// "Insert sketch" in the editor: a pen key above the keyboard that asks for
/// a size and puts an empty drawing box (a `sketch` block) in the note at
/// the caret, on lines of its own, as one undo step. It is drawn on in the
/// note's preview.
class SketchEditorExtras implements EditorExtras {
  SketchEditorExtras({required this.repository, required this.label});

  final CanvasRepository repository;

  /// What a screen reader says of the key.
  final String label;

  /// The id of the key in the accessory bar.
  static const String keyId = 'sketch';

  @override
  List<EditorAccessoryExtra> accessoryKeys(EditorTextTarget target) =>
      <EditorAccessoryExtra>[
        EditorAccessoryExtra(
          id: keyId,
          icon: KansoIcons.penLine,
          semanticLabel: label,
          onTap: insert,
        ),
      ];

  @override
  List<EditorSelectionAction> selectionActions(EditorTextTarget target) =>
      const <EditorSelectionAction>[];

  @override
  void release(EditorTextTarget target) {}

  /// Asks for a size, then puts the sketch in at the caret. A shared note
  /// refuses before any size is asked for (two people replacing one
  /// sketch's text would break it); the library refuses it too.
  Future<void> insert(BuildContext context, EditorTextTarget target) async {
    if (!repository.takesSketches(target.noteId)) {
      KansoFeedback.show(
        context,
        context.l10n.sketchSharedInsert,
        tone: KansoFeedbackTone.error,
      );
      return;
    }
    final size = await SketchSizeSheet.show(context);
    if (size == null || !target.active || !context.mounted) {
      return;
    }
    final value = target.value;
    final at = value.selection.isValid
        ? value.selection.end.clamp(0, value.text.length)
        : value.text.length;
    final made = await repository.insertSketch(
      noteId: target.noteId,
      body: value.text,
      at: at,
      width: size.$1,
      height: size.$2,
    );
    if (!target.active || !context.mounted) {
      return;
    }
    final l10n = context.l10n;
    made.fold(
      (_) => KansoFeedback.show(
        context,
        repository.takesSketches(target.noteId)
            ? l10n.sketchNoteFull
            : l10n.sketchSharedInsert,
        tone: KansoFeedbackTone.error,
      ),
      (done) {
        final text = value.text;
        // What the core added: everything the new text has that the old had
        // not, around the caret.
        final added = done.body.substring(
          at,
          done.body.length - (text.length - at),
        );
        target.replace(TextRange.collapsed(at), added);
      },
    );
  }
}

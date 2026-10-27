// Kanso — replace_notes_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'replace_notes_bloc.dart';

@freezed
abstract class ReplaceNotesState with _$ReplaceNotesState {
  const factory ReplaceNotesState({
    /// What to find and the toggles.
    @Default(
      ReplaceOptions(
        find: '',
        matchCase: false,
        wholeWord: false,
        regex: false,
        skipCode: false,
        includeTrash: false,
      ),
    )
    ReplaceOptions options,

    /// What to put instead.
    @Default('') String replacement,
    @Default(ReplacePreviewStatus.idle) ReplacePreviewStatus previewStatus,

    /// Every match, note by note; null until there is text to find.
    ReplacePreview? preview,

    /// The notes left out of the replace, by id.
    @Default(<String>{}) Set<String> excluded,

    /// The replace is running.
    @Default(false) bool applying,

    /// What the last replace did, to say and to undo.
    ReplaceOutcome? outcome,

    /// The undo is running.
    @Default(false) bool undoing,

    /// What undoing the last replace did.
    ReplaceUndone? undone,

    /// Why the last replace or undo did not happen.
    ReplaceProblem? problem,
  }) = _ReplaceNotesState;

  const ReplaceNotesState._();

  /// The notes the replace would change: those of the preview not left out.
  List<ReplaceNote> get chosen => <ReplaceNote>[
    for (final note in preview?.notes ?? const <ReplaceNote>[])
      if (!excluded.contains(note.id)) note,
  ];

  /// Those notes at the revisions the preview found them in.
  List<PlannedNote> get planned => <PlannedNote>[
    for (final note in chosen) PlannedNote(id: note.id, rev: note.rev),
  ];

  /// How many matches the replace would change.
  int get matches => chosen.fold<int>(0, (sum, note) => sum + note.count);
}

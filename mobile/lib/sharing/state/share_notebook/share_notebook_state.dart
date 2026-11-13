// Kanso — share_notebook_state.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'share_notebook_bloc.dart';

@freezed
abstract class ShareNotebookState with _$ShareNotebookState {
  const factory ShareNotebookState({
    @Default('') String notebookId,
    @Default(ShareSheetStatus.loading) ShareSheetStatus status,

    /// Why the notebook can't be shared, in the library's words.
    String? reason,

    /// What sharing it would do, while it isn't shared.
    BookShareCheck? check,

    /// Its sharing, once it is shared.
    BookShareInfo? info,
    @Default(BookRole.editor) BookRole inviteRole,

    /// A step is running: nothing can be pressed.
    @Default(false) bool busy,

    /// How far sharing it has got, while it is being shared.
    BookShareProgress? progress,
    BookPrompt? prompt,
    ShareProblem? problem,
    BookDone? done,
  }) = _ShareNotebookState;

  const ShareNotebookState._();

  /// The notebook's name, as the sheet's title says it.
  String get name => info?.name ?? check?.name ?? '';
}

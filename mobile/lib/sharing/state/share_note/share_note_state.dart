// Kanso — share_note_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'share_note_bloc.dart';

@freezed
abstract class ShareNoteState with _$ShareNoteState {
  const factory ShareNoteState({
    /// The note the sheet is for.
    @Default('') String noteId,
    @Default(ShareSheetStatus.loading) ShareSheetStatus status,

    /// Why the note can't be shared, in the library's words, while the
    /// status is `notShareable`.
    String? reason,

    /// The note's sharing; null while it is not shared.
    ShareInfo? info,

    /// A step is running.
    @Default(false) bool busy,

    /// A question before someone's keys are trusted.
    SharePrompt? prompt,

    /// The address of the link just made. It is known only to the call that
    /// made it, so it shows until the sheet closes.
    String? madeLink,

    /// The link was copied.
    @Default(false) bool linkCopied,

    /// Why the last step did not happen.
    ShareProblem? problem,

    /// What the last step did.
    ShareDone? done,

    /// Stop sharing (or Leave) was pressed once; the next press does it.
    @Default(false) bool confirmEnd,

    /// The role the next invitation gives.
    @Default(ShareRole.editor) ShareRole inviteRole,

    /// The role and the days of the next link.
    @Default(ShareRole.commenter) ShareRole linkRole,
    @Default(7) int linkDays,
  }) = _ShareNoteState;

  const ShareNoteState._();

  /// Whether this account manages the note's sharing: it is the owner, or
  /// the note is not shared yet.
  bool get manages {
    final shared = info;
    return shared == null || shared.myRole == ShareRole.owner;
  }
}

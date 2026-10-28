// Kanso — undo_notice_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'undo_notice_bloc.dart';

@freezed
abstract class UndoNoticeState with _$UndoNoticeState {
  const factory UndoNoticeState({
    /// The notice showing; null when there is none.
    UndoNotice? notice,

    /// Whether Undo is being carried out.
    @Default(false) bool undoing,

    /// Why the last undo failed, until it is dismissed.
    ApplicationError? error,
  }) = _UndoNoticeState;
}

// Kanso: opened_documents_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'opened_documents_bloc.dart';

@freezed
abstract class OpenedDocumentsState with _$OpenedDocumentsState {
  const factory OpenedDocumentsState({
    /// Documents are being opened.
    @Default(false) bool opening,

    /// The note to show: the last document opened. Set once, then
    /// [OpenedDocumentsEvent.dismissed] clears it.
    String? noteId,

    /// What went wrong with the others, a sentence each.
    @Default(<String>[]) List<String> problems,

    /// Opening failed as a whole: the library could not be written.
    @Default(false) bool failed,
  }) = _OpenedDocumentsState;

  const OpenedDocumentsState._();

  /// There is something to tell the person.
  bool get hasNotice => failed || problems.isNotEmpty;
}

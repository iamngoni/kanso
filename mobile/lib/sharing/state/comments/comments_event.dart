// Kanso — comments_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'comments_bloc.dart';

@freezed
sealed class CommentsEvent with _$CommentsEvent {
  /// The note [noteId] is open: read its comments.
  const factory CommentsEvent.started(String noteId) = CommentsStartedEvent;

  /// Read the comments again: one was made, or a sync brought news.
  const factory CommentsEvent.refreshed() = CommentsRefreshedEvent;

  /// Resolve a comment, or open it again.
  const factory CommentsEvent.resolved({
    required String commentId,
    required bool resolved,
  }) = CommentsResolvedEvent;

  /// Change the words of one of this account's comments.
  const factory CommentsEvent.edited({
    required String commentId,
    required String body,
  }) = CommentsEditedEvent;

  /// Delete one of this account's comments, for everyone.
  const factory CommentsEvent.deleted(String commentId) = CommentsDeletedEvent;

  /// Show comment [commentId]'s words in the editor.
  const factory CommentsEvent.selected(String commentId) =
      CommentsSelectedEvent;

  /// The editor showed them.
  const factory CommentsEvent.selectionHandled() =
      CommentsSelectionHandledEvent;
}

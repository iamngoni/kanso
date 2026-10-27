// Kanso — comments_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'comments_bloc.dart';

@freezed
abstract class CommentsState with _$CommentsState {
  const factory CommentsState({
    /// The note whose comments these are.
    @Default('') String noteId,
    @Default(LoadStatus.loading) LoadStatus status,

    /// This account's role in the note; null when it isn't shared.
    ShareRole? role,

    /// In the order the desktop lists them: on words still in the text, in
    /// text order, then the orphaned ones.
    @Default(<CommentRow>[]) List<CommentRow> comments,

    /// "2 open · 1 resolved", empty with none.
    @Default('') String band,

    /// Why the comments could not be read, or the last change made.
    ApplicationError? error,

    /// A comment whose words the editor is to select, once it is open.
    String? selecting,
  }) = _CommentsState;

  const CommentsState._();

  /// Whether the note is shared, so it has comments at all.
  bool get shared => role != null;

  /// Whether this account may comment, resolve and reopen.
  bool get mayComment => role != null && role != ShareRole.viewer;

  /// How many comments are open: the count on the button.
  int get open => comments.where((comment) => !comment.resolved).length;

  /// The comments on words still in the text.
  List<CommentRow> get anchored => <CommentRow>[
    for (final comment in comments)
      if (comment.placement is CommentPlacement_Anchored) comment,
  ];

  /// The comments whose words were deleted.
  List<CommentRow> get orphaned => <CommentRow>[
    for (final comment in comments)
      if (comment.placement is CommentPlacement_Orphaned) comment,
  ];
}

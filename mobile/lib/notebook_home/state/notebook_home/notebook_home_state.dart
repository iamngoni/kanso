// Kanso — notebook_home_state.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'notebook_home_bloc.dart';

@freezed
abstract class NotebookHomeState with _$NotebookHomeState {
  const factory NotebookHomeState({
    @Default('') String bookId,
    @Default(LoadStatus.loading) LoadStatus status,

    /// The core's own words; null until read.
    HomeWords? words,
    HomePanel? panel,

    /// The map, read after the sections are shown; null until then.
    HomeMap? map,

    /// Section keys being rewritten now.
    @Default(<String>{}) Set<String> working,

    /// Section keys that failed to rewrite, with the model's own words.
    @Default(<String, String>{}) Map<String, String> trouble,

    /// A section was copied into the Home note: the page says so once.
    @Default(false) bool copied,

    /// A new Home note was made: the page opens it once.
    String? madeNoteId,
  }) = _NotebookHomeState;
}

// Kanso — ask_scope.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ask_scope.freezed.dart';

/// What a question searches: all notes, a notebook (and the ones inside it)
/// or a tag. A locked notebook is never offered.
@freezed
abstract class AskScope with _$AskScope {
  const factory AskScope({
    /// `all`, `book` or `tag`.
    @Default(AskScope.allKind) String kind,

    /// The notebook's ID or the tag's name; empty for all notes.
    @Default('') String value,

    /// "All notes", the notebook's path (`Work/Plans`) or `#tag`.
    @Default('') String label,
  }) = _AskScope;

  const AskScope._();

  /// The kind of the scope that searches everything.
  static const String allKind = 'all';

  /// Every note that may be asked about.
  static const AskScope everything = AskScope();

  /// Whether this is narrower than all notes.
  bool get isNarrow => kind != allKind;
}

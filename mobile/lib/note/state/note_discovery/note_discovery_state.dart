// Kanso — note_discovery_state.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'note_discovery_bloc.dart';

@freezed
abstract class NoteDiscoveryState with _$NoteDiscoveryState {
  const factory NoteDiscoveryState({
    @Default('') String noteId,
    @Default(false) bool loading,
    @Default(false) bool relatedEnabled,
    @Default(<RelatedRow>[]) List<RelatedRow> related,
    NoteMeaning? meaning,
  }) = _NoteDiscoveryState;
}

// Kanso — resurface_state.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'resurface_bloc.dart';

@freezed
abstract class ResurfaceState with _$ResurfaceState {
  const factory ResurfaceState({
    @Default('') String noteId,

    /// Related notes are on and the setting says to offer notes.
    @Default(false) bool enabled,

    /// "closer", "default" or "looser".
    @Default('default') String closeness,

    /// Past notes about the paragraph, in the order they came.
    @Default(<ResurfaceRow>[]) List<ResurfaceRow> rows,

    /// Notes written on a daily note's date in earlier years.
    @Default(<OnThisDayRow>[]) List<OnThisDayRow> day,
  }) = _ResurfaceState;
}

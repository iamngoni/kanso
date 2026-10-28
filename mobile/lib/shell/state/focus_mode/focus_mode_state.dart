// Kanso — focus_mode_state.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'focus_mode_bloc.dart';

@freezed
abstract class FocusModeState with _$FocusModeState {
  const factory FocusModeState({
    /// Whether the open note is shown by itself.
    @Default(false) bool active,

    /// Whether the saved choice has been read. A choice made before it was
    /// read is not overwritten by the file.
    @Default(false) bool ready,
  }) = _FocusModeState;
}

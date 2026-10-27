// Kanso — recovery_key_state.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'recovery_key_bloc.dart';

@freezed
abstract class RecoveryKeyState with _$RecoveryKeyState {
  const factory RecoveryKeyState({
    @Default(LoadStatus.loading) LoadStatus status,

    /// The key to write down; null when the library holds none unsaved.
    RecoveryKeyText? key,

    /// Whether the person said they saved it.
    @Default(false) bool confirmed,

    /// Whether Done is being carried out.
    @Default(false) bool saving,

    /// Why the key could not be read or Done failed.
    ApplicationError? error,

    /// Whether Done went through: the screen closes.
    @Default(false) bool done,
  }) = _RecoveryKeyState;
}

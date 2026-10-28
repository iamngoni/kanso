// Kanso — privacy_cover_state.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'privacy_cover_bloc.dart';

@freezed
abstract class PrivacyCoverState with _$PrivacyCoverState {
  const factory PrivacyCoverState({
    /// Whether the cover is on.
    @Default(false) bool secure,
  }) = _PrivacyCoverState;
}

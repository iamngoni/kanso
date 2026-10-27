// Kanso — privacy_cover_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'privacy_cover_bloc.dart';

@freezed
sealed class PrivacyCoverEvent with _$PrivacyCoverEvent {
  /// The cover should be [secure] (on) or off.
  const factory PrivacyCoverEvent.changed({required bool secure}) =
      PrivacyCoverChangedEvent;
}

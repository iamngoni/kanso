// Kanso — ai_key.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ai_key.freezed.dart';

/// An API key on its way from the field to the secret store.
///
/// It is the only form a key takes in the app, and it says nothing of
/// itself when printed, so a log of events or a failing test never shows
/// one. It is never kept in a state.
@freezed
abstract class AiKey with _$AiKey {
  const factory AiKey(String value) = _AiKey;

  const AiKey._();

  @override
  String toString() => 'AiKey(hidden)';
}

// Kanso — assist_action_info.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import 'assist_action.dart';

part 'assist_action_info.freezed.dart';

/// An assistant action as the menu lists it, in the core's words.
@freezed
abstract class AssistActionInfo with _$AssistActionInfo {
  const factory AssistActionInfo({
    required AssistAction action,

    /// The menu label: `Improve Writing`.
    required String label,

    /// What the suggestion is called once it is made: `Improved`.
    required String noun,

    /// Whether accepting replaces the selection, or else the suggestion goes
    /// after it.
    required bool replaces,

    /// Whether the action needs text selected; Continue Writing does not.
    @Default(true) bool needsSelection,
  }) = _AssistActionInfo;
}

// Kanso — ai_test_result.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'ai_test_result.freezed.dart';

/// What the Test button found.
@freezed
sealed class AiTestResult with _$AiTestResult {
  /// The model answered; [said] is what it said.
  const factory AiTestResult.ok({required String said}) = AiTestOk;

  /// The model could not be reached or refused, in [message]'s plain words.
  const factory AiTestResult.failed({required String message}) = AiTestFailed;
}

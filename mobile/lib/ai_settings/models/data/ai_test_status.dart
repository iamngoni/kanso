// Kanso — ai_test_status.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where the Test button is.
enum AiTestStatus {
  /// Not pressed since the settings last changed.
  idle,

  /// A question is on its way to the model.
  running,

  /// The model answered.
  ok,

  /// The model could not be reached or refused.
  failed,
}

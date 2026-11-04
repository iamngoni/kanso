// Kanso — feedback_screenshot_repository.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../data/feedback_draft.dart';

/// The photo picker for a report's screenshot. There is no in-app capture:
/// the screen behind Settings is Settings.
// ignore: one_member_abstracts, a seam with one reader.
abstract interface class FeedbackScreenshotRepository {
  /// The image the person chose, or null if they chose none or it could not
  /// be read.
  Future<FeedbackScreenshot?> pick();
}

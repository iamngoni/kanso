// Kanso — review_moment.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Something good the person just finished, after which the app may ask
/// for a review once they are back on a list.
enum ReviewMoment {
  /// A note was let go after it was written in.
  noteClosedAfterEdit,

  /// The first sync on this phone completed.
  firstSync,

  /// A note or notebook was published.
  published,

  /// A weekly review was saved.
  weeklyReviewFinished,

  /// A task was ticked off.
  taskDone,
}

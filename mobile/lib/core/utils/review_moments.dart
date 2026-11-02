// Kanso — review_moments.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import '../../injection.dart';
import '../models/data/review_moment.dart';
import '../services/review_prompt.dart';

/// Marks [moment] for the review prompt, where something good happens that
/// the prompt does not see itself. Best effort: a missing registration (a
/// test without the app's services) never reaches the action.
void markReviewMoment(ReviewMoment moment) {
  _withPrompt((prompt) => prompt.moment(moment));
}

/// Counts a note let go after it was written in, which is also a good
/// moment for the review prompt.
void markNoteEditedForReview() {
  _withPrompt((prompt) => unawaited(prompt.noteEdited()));
}

void _withPrompt(void Function(ReviewPrompt prompt) action) {
  try {
    action(getIt<ReviewPrompt>());
  } on Object {
    // The review prompt never disturbs the app.
  }
}

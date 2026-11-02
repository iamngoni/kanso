// Kanso — review_prompt_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../data/application_error.dart';
import '../../data/review_prompt_record.dart';

/// What the phone counts to decide whether to ask for a review, kept on
/// this phone only.
abstract interface class ReviewPromptRepository {
  /// The record as saved; the empty record when nothing was saved yet.
  Future<Either<ApplicationError, ReviewPromptRecord>> load();

  /// Applies [change] to the saved record and saves the result, one change
  /// at a time in the order asked. The record as saved.
  Future<Either<ApplicationError, ReviewPromptRecord>> update(
    ReviewPromptRecord Function(ReviewPromptRecord current) change,
  );
}

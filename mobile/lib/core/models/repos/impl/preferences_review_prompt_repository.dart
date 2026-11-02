// Kanso — preferences_review_prompt_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../data/application_error.dart';
import '../../data/review_prompt_record.dart';
import '../abs/preferences_repository.dart';
import '../abs/review_prompt_repository.dart';

/// [ReviewPromptRepository] in `preferences.json`, beside the person's other
/// choices: on this phone, out of backups, and never sent.
@LazySingleton(as: ReviewPromptRepository)
class PreferencesReviewPromptRepository implements ReviewPromptRepository {
  PreferencesReviewPromptRepository(this._preferences);

  final PreferencesRepository _preferences;

  @override
  Future<Either<ApplicationError, ReviewPromptRecord>> load() async {
    final result = await _preferences.load();
    return result.map((preferences) => preferences.reviewPrompt);
  }

  @override
  Future<Either<ApplicationError, ReviewPromptRecord>> update(
    ReviewPromptRecord Function(ReviewPromptRecord current) change,
  ) async {
    final result = await _preferences.update(
      (current) => current.copyWith(reviewPrompt: change(current.reviewPrompt)),
    );
    return result.map((preferences) => preferences.reviewPrompt);
  }
}

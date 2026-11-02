// Kanso — review_prompt_fixtures.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/review_prompt_record.dart';
import 'package:kanso/core/models/repos/abs/review_prompt_repository.dart';
import 'package:kanso/core/models/repos/abs/store_review_repository.dart';

/// A review record in memory; [failing] makes every save fail.
class MemoryReviewPromptRepository implements ReviewPromptRepository {
  MemoryReviewPromptRepository([this.record = const ReviewPromptRecord()]);

  ReviewPromptRecord record;
  bool failing = false;

  @override
  Future<Either<ApplicationError, ReviewPromptRecord>> load() async =>
      Right<ApplicationError, ReviewPromptRecord>(record);

  @override
  Future<Either<ApplicationError, ReviewPromptRecord>> update(
    ReviewPromptRecord Function(ReviewPromptRecord current) change,
  ) async {
    if (failing) {
      return const Left<ApplicationError, ReviewPromptRecord>(
        ApplicationError.storage(message: 'Disk full'),
      );
    }
    record = change(record);
    return Right<ApplicationError, ReviewPromptRecord>(record);
  }
}

/// The system's review request, counted.
class FixtureStoreReviewRepository implements StoreReviewRepository {
  FixtureStoreReviewRepository({this.isAvailable = true});

  bool isAvailable;
  int requests = 0;

  @override
  Future<bool> available() async => isAvailable;

  @override
  Future<void> request() async => requests++;
}

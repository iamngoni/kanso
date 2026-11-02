// Kanso — in_app_store_review_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';
import 'package:in_app_review/in_app_review.dart';
import 'package:injectable/injectable.dart';

import '../../../configs/kanso_stores.dart';
import '../abs/store_review_repository.dart';

/// [StoreReviewRepository] through the `in_app_review` plugin:
/// `AppStore.requestReview(in:)` on iOS and Play's in-app review on Android.
@LazySingleton(as: StoreReviewRepository)
class InAppStoreReviewRepository implements StoreReviewRepository {
  InAppStoreReviewRepository([
    @ignoreParam InAppReview? review,
    @ignoreParam TargetPlatform? platform,
  ]) : _review = review ?? InAppReview.instance,
       _platform = platform;

  final InAppReview _review;
  final TargetPlatform? _platform;

  @override
  Future<bool> available() async {
    // No listing (Android before Play approves it, or a desktop): nothing
    // to rate, so nothing is asked.
    if (KansoStores.writeReviewFor(_platform ?? defaultTargetPlatform) ==
        null) {
      return false;
    }
    try {
      return await _review.isAvailable();
    } on Object {
      return false;
    }
  }

  @override
  Future<void> request() async {
    try {
      await _review.requestReview();
    } on Object {
      // The system said no, or has no store: nothing for the person to do.
    }
  }
}

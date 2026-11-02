// Kanso — kanso_stores.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart';

/// Where Kanso Pocket is sold, the same values as the website's
/// `site/src/lib/stores.ts`: change both together.
abstract final class KansoStores {
  /// Kanso Pocket's id on the App Store (`APP_STORE_ID` on the website).
  static const String appStoreId = '6828301844';

  /// The Android application id, which is the Google Play listing's id.
  static const String playPackage = 'app.kanso.kanso';

  /// Whether the Google Play listing is live (`ANDROID_URL` is set on the
  /// website). Until it is, Android neither asks for a review nor offers
  /// to rate the app: there is no page to rate it on.
  static const bool playListingLive = false;

  /// The App Store's page to write a review of Kanso Pocket, in the App
  /// Store app.
  static final Uri appStoreWriteReview = Uri.parse(
    'itms-apps://apps.apple.com/app/id$appStoreId?action=write-review',
  );

  /// Kanso Pocket's page in the Google Play app, where it is rated.
  static final Uri playListing = Uri.parse(
    'market://details?id=$playPackage',
  );

  /// The page to rate Kanso Pocket on from [platform]; null where there is
  /// none (Android before the listing is live, and the desktops, which are
  /// not sold in a store).
  static Uri? writeReviewFor(TargetPlatform platform) => switch (platform) {
    TargetPlatform.iOS => appStoreWriteReview,
    TargetPlatform.android => playListingLive ? playListing : null,
    TargetPlatform.fuchsia ||
    TargetPlatform.linux ||
    TargetPlatform.macOS ||
    TargetPlatform.windows => null,
  };
}

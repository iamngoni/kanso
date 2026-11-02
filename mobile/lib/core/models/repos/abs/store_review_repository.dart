// Kanso — store_review_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The system's own request for a review of the app (on iOS
/// `SKStoreReviewController`, on Android Play's in-app review). The system
/// decides whether it shows anything, and caps how often; the app never
/// shows its own prompt first.
abstract interface class StoreReviewRepository {
  /// Whether the request can be made on this device: the store is there,
  /// and the app has a listing in it.
  Future<bool> available();

  /// Asks the system to show its review prompt. Whether it was shown is
  /// never known. Never throws.
  Future<void> request();
}

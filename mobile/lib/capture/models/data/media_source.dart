// Kanso — media_source.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where a photo or a document added to the library comes from.
enum MediaSource {
  /// A photo taken with the camera now.
  camera,

  /// Photos chosen from the phone's library.
  photos,

  /// PDFs chosen from the phone's files.
  pdf,
}

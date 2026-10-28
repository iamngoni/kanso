// Kanso — gallery_filter.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How the gallery's themes are ordered.
enum GalleryFilter {
  /// The most installed first. The signed catalog does not count installs
  /// yet, so ties (all of them, for now) put Kanso's own themes first and
  /// then go by name.
  popular,

  /// The most recently updated first.
  recent,
}

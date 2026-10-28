// Kanso — gallery_labels.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../gen/l10n/app_localizations.dart';
import '../models/data/gallery_filter.dart';
import '../models/data/gallery_problem.dart';

/// The words of the theme gallery.
extension GalleryLabels on AppLocalizations {
  /// What an order of the gallery's themes is called.
  String galleryFilterName(GalleryFilter filter) {
    return switch (filter) {
      GalleryFilter.popular => galleryFilterPopular,
      GalleryFilter.recent => galleryFilterRecent,
    };
  }

  /// Why a step on the gallery did not happen.
  String galleryProblemText(GalleryProblem problem) {
    return switch (problem) {
      GalleryProblemWords(:final message) => message,
      GalleryProblemOffline() => galleryProblemOffline,
      GalleryProblemUnexpected() => galleryProblemUnexpected,
    };
  }
}

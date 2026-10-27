// Kanso — gallery_problem_of.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/models/data/application_error.dart';
import '../models/data/gallery_problem.dart';

/// What [error], from a call on the gallery, is to the person.
///
/// The library words what it refuses for people ("Nord needs a newer
/// Kanso."), so `invalid` is shown as it is; a network failure has words of
/// the app's own; the rest is a bug or a broken disk.
GalleryProblem galleryProblemOf(ApplicationError error) {
  return switch (error) {
    InvalidError(:final message) when message.trim().isNotEmpty =>
      GalleryProblem.words(message.trim()),
    NetworkError() => const GalleryProblem.offline(),
    _ => const GalleryProblem.unexpected(),
  };
}

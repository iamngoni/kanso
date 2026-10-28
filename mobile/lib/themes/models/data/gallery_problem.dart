// Kanso — gallery_problem.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'gallery_problem.freezed.dart';

/// Why a step on the gallery did not happen, as the screen says it.
@freezed
sealed class GalleryProblem with _$GalleryProblem {
  /// A sentence from the library, in the desktop's words: "Nord didn't match
  /// the catalog: …".
  const factory GalleryProblem.words(String message) = GalleryProblemWords;

  /// The gallery could not be reached.
  const factory GalleryProblem.offline() = GalleryProblemOffline;

  /// Anything else: nothing to do but try again.
  const factory GalleryProblem.unexpected() = GalleryProblemUnexpected;
}

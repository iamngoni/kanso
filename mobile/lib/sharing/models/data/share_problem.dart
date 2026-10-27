// Kanso — share_problem.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

part 'share_problem.freezed.dart';

/// Why a step of sharing did not happen, as the screen says it.
@freezed
sealed class ShareProblem with _$ShareProblem {
  /// A sentence for the person from the library, in the desktop's words:
  /// "There's no Kanso account called ana that can share yet."
  const factory ShareProblem.words(String message) = ShareProblemWords;

  /// No username was typed.
  const factory ShareProblem.nameMissing() = ShareProblemNameMissing;

  /// No link was typed.
  const factory ShareProblem.linkMissing() = ShareProblemLinkMissing;

  /// The server could not be reached.
  const factory ShareProblem.offline() = ShareProblemOffline;

  /// The account signed this device out.
  const factory ShareProblem.signedOut() = ShareProblemSignedOut;

  /// Anything else: nothing the person can do about it but try again.
  const factory ShareProblem.unexpected() = ShareProblemUnexpected;
}

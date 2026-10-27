// Kanso — share_problem_of.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/models/data/application_error.dart';
import '../models/data/share_problem.dart';

/// What [error], from a call on shared notes, is to the person.
///
/// The library words its refusals for people (`invalid` and `auth` carry
/// them: "Only the note's owner can stop sharing it."), so those are shown
/// as they are; a network failure and a signed-out device have words of the
/// app's own; the rest is a bug or a broken disk, which the person can only
/// try again.
ShareProblem shareProblemOf(ApplicationError error) {
  return switch (error) {
    InvalidError(:final message) when message.trim().isNotEmpty =>
      ShareProblem.words(message.trim()),
    AuthError(:final message) when message.trim().isNotEmpty =>
      ShareProblem.words(message.trim()),
    NetworkError() => const ShareProblem.offline(),
    SignedOutError() => const ShareProblem.signedOut(),
    _ => const ShareProblem.unexpected(),
  };
}

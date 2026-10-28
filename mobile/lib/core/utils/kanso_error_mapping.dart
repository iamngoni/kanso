// Kanso — kanso_error_mapping.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import '../bridge/api/types.dart';
import '../models/data/application_error.dart';

/// The one place the library's errors become the app's.
///
/// [KansoError] is what a call in the Rust library can fail with;
/// [PanicException] is what the bridge raises when a panic got past the
/// library's own net. Anything else is not the library's, and is
/// [ApplicationError.unexpected].
ApplicationError applicationErrorOf(Object error) {
  return switch (error) {
    KansoError_NotFound() => const ApplicationError.notFound(
      message: 'Not found.',
    ),
    KansoError_Locked() => const ApplicationError.locked(
      message: 'The note is locked.',
    ),
    KansoError_Conflict(:final currentRev) => ApplicationError.conflict(
      message: 'The note changed meanwhile.',
      currentRev: currentRev,
    ),
    KansoError_Invalid(:final message) => ApplicationError.invalid(
      message: message,
    ),
    KansoError_Storage(:final message) => ApplicationError.storage(
      message: message,
    ),
    KansoError_Network(:final message) => ApplicationError.network(
      message: message,
    ),
    KansoError_SignedOut() => const ApplicationError.signedOut(
      message: 'Not signed in.',
    ),
    KansoError_Auth(:final message) => ApplicationError.auth(message: message),
    KansoError_Internal(:final message) => ApplicationError.internal(
      message: message,
    ),
    PanicException(:final message) => ApplicationError.internal(
      message: message,
    ),
    _ => ApplicationError.unexpected(message: '$error'),
  };
}

/// Runs [call] and returns what it returned, or the error it failed with as
/// an [ApplicationError].
Future<Either<ApplicationError, T>> attempt<T>(
  Future<T> Function() call,
) async {
  try {
    return Right<ApplicationError, T>(await call());
  } on Object catch (error) {
    return Left<ApplicationError, T>(applicationErrorOf(error));
  }
}

/// [attempt] for a call that is synchronous: the editor's hot path, which
/// runs on the UI isolate in microseconds and is never awaited.
Either<ApplicationError, T> attemptSync<T>(T Function() call) {
  try {
    return Right<ApplicationError, T>(call());
  } on Object catch (error) {
    return Left<ApplicationError, T>(applicationErrorOf(error));
  }
}

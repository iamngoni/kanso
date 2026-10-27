// Kanso — session_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../bridge/api/types.dart';
import '../../data/application_error.dart';

/// The lock session. The lock password's keys stay in the Rust core.
abstract interface class SessionRepository {
  /// Opens the locked notes with [password]; false when it is wrong.
  Future<Either<ApplicationError, bool>> unlock(String password);

  /// Closes the locked notes.
  Future<Either<ApplicationError, Unit>> lock();

  Future<Either<ApplicationError, bool>> isUnlocked();

  /// Whether the note [id] can be read now.
  Future<Either<ApplicationError, bool>> isNoteUnlocked(String id);

  /// Locks the note [id] under [password].
  Future<Either<ApplicationError, Saved>> lockNote(String id, String password);

  /// Closes the locked notes because the app is leaving the foreground.
  Future<Either<ApplicationError, Unit>> lockForBackground();

  /// Whether a lock password exists, the notes are open, and biometrics are
  /// on.
  Future<Either<ApplicationError, LockStatus>> lockStatus();

  /// Opens with [password], tried on the note [id]'s own lock; false when it
  /// is wrong.
  Future<Either<ApplicationError, bool>> unlockNote(
    String id,
    String password,
  );

  /// Opens with [password], tried on the notebook [bookId]'s lock; false
  /// when it is wrong.
  Future<Either<ApplicationError, bool>> unlockNotebook(
    String bookId,
    String password,
  );

  /// Locks the note [id] with the key the open session holds. A locked
  /// session is a [LockedError].
  Future<Either<ApplicationError, Saved>> lockNoteWithSession(String id);

  /// Takes the open note [id]'s lock off, keeping its text as plain text.
  Future<Either<ApplicationError, Saved>> removeNoteLock(String id);

  /// Locks the notebook [bookId], returning how many notes were sealed.
  /// [password] is needed when the notes aren't open.
  Future<Either<ApplicationError, int>> lockNotebook(
    String bookId, {
    String? password,
  });

  /// Takes the open notebook [bookId]'s lock off.
  Future<Either<ApplicationError, Unit>> removeNotebookLock(String bookId);

  /// Whether the notebook [bookId]'s notes can be read now.
  Future<Either<ApplicationError, bool>> notebookIsOpen(String bookId);

  /// Seals the locked notes again under [next]; returns how many [current]
  /// didn't open.
  Future<Either<ApplicationError, int>> changePassword(
    String current,
    String next,
  );

  /// The library's ID, which names the platform's biometric key or item.
  Future<Either<ApplicationError, String>> libraryId();

  Future<Either<ApplicationError, bool>> biometricsOn();

  /// A ticket for the platform's biometric enrolment; the notes must be open.
  Future<Either<ApplicationError, int>> beginBiometricEnrollment();

  /// A ticket for the platform's biometric check, or null when biometrics
  /// are off.
  Future<Either<ApplicationError, int?>> beginBiometricUnlock();

  /// Forgets the keys kept for biometrics.
  Future<Either<ApplicationError, Unit>> turnOffBiometrics();

  /// Each time the locked notes open or close.
  Stream<LockEvent> lockEvents();
}

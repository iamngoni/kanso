// Kanso — rust_session_repository.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/locks.dart';
import '../../../bridge/api/types.dart';
import '../../../utils/cancelable_stream.dart';
import '../../../utils/kanso_error_mapping.dart';
import '../../data/application_error.dart';
import '../abs/session_repository.dart';

/// [SessionRepository] over the Rust core's lock session.
@LazySingleton(as: SessionRepository)
class RustSessionRepository implements SessionRepository {
  RustSessionRepository(this._locks);

  final KansoLocks _locks;

  @override
  Future<Either<ApplicationError, bool>> unlock(String password) =>
      attempt(() => _locks.unlock(password: password));

  @override
  Future<Either<ApplicationError, Unit>> lock() => attempt(() async {
    await _locks.lock();
    return unit;
  });

  @override
  Future<Either<ApplicationError, bool>> isUnlocked() =>
      attempt(_locks.isUnlocked);

  @override
  Future<Either<ApplicationError, bool>> isNoteUnlocked(String id) =>
      attempt(() => _locks.isNoteUnlocked(id: id));

  @override
  Future<Either<ApplicationError, Saved>> lockNote(
    String id,
    String password,
  ) => attempt(() => _locks.lockNote(id: id, password: password));

  @override
  Future<Either<ApplicationError, Unit>> lockForBackground() =>
      attempt(() async {
        await _locks.lockForBackground();
        return unit;
      });

  @override
  Future<Either<ApplicationError, LockStatus>> lockStatus() =>
      attempt(_locks.lockStatus);

  @override
  Future<Either<ApplicationError, bool>> unlockNote(
    String id,
    String password,
  ) => attempt(() => _locks.unlockNote(id: id, password: password));

  @override
  Future<Either<ApplicationError, bool>> unlockNotebook(
    String bookId,
    String password,
  ) => attempt(() => _locks.unlockNotebook(bookId: bookId, password: password));

  @override
  Future<Either<ApplicationError, Saved>> lockNoteWithSession(String id) =>
      attempt(() => _locks.lockNoteWithSession(id: id));

  @override
  Future<Either<ApplicationError, Saved>> removeNoteLock(String id) =>
      attempt(() => _locks.removeNoteLock(id: id));

  @override
  Future<Either<ApplicationError, int>> lockNotebook(
    String bookId, {
    String? password,
  }) => attempt(() => _locks.lockNotebook(bookId: bookId, password: password));

  @override
  Future<Either<ApplicationError, Unit>> removeNotebookLock(String bookId) =>
      attempt(() async {
        await _locks.removeNotebookLock(bookId: bookId);
        return unit;
      });

  @override
  Future<Either<ApplicationError, bool>> notebookIsOpen(String bookId) =>
      attempt(() => _locks.notebookIsOpen(bookId: bookId));

  @override
  Future<Either<ApplicationError, int>> changePassword(
    String current,
    String next,
  ) => attempt(() => _locks.changePassword(old: current, new_: next));

  @override
  Future<Either<ApplicationError, String>> libraryId() =>
      attempt(_locks.libraryId);

  @override
  Future<Either<ApplicationError, bool>> biometricsOn() =>
      attempt(_locks.biometricsOn);

  @override
  Future<Either<ApplicationError, int>> beginBiometricEnrollment() =>
      attempt(_locks.beginBiometricEnrollment);

  @override
  Future<Either<ApplicationError, int?>> beginBiometricUnlock() =>
      attempt(_locks.beginBiometricUnlock);

  @override
  Future<Either<ApplicationError, Unit>> turnOffBiometrics() =>
      attempt(() async {
        await _locks.turnOffBiometrics();
        return unit;
      });

  @override
  Stream<LockEvent> lockEvents() => cancelableStream(_locks.lockEvents());
}

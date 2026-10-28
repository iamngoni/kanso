// Kanso — fixture_session_repository.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/repos/abs/session_repository.dart';

/// A lock session for widget and golden tests: it answers from fields a test
/// can change, and records what it was asked.
class FixtureSessionRepository implements SessionRepository {
  FixtureSessionRepository({
    this.hasLockPassword = true,
    this.unlocked = false,
    this.biometrics = false,
  });

  bool hasLockPassword;
  bool unlocked;

  /// Whether biometrics are on.
  bool biometrics;

  /// The lock password the fixture accepts.
  String password = 'correct horse';

  /// When set, every call that changes a lock fails with it.
  ApplicationError? failure;

  final List<String> calls = <String>[];
  final StreamController<LockEvent> events =
      StreamController<LockEvent>.broadcast();

  Future<void> close() => events.close();

  Either<ApplicationError, T> _answer<T>(String call, T value) {
    calls.add(call);
    final error = failure;
    return error == null
        ? Right<ApplicationError, T>(value)
        : Left<ApplicationError, T>(error);
  }

  @override
  Future<Either<ApplicationError, LockStatus>> lockStatus() async =>
      Right<ApplicationError, LockStatus>(
        LockStatus(
          hasLockPassword: hasLockPassword,
          unlocked: unlocked,
          biometricsOn: biometrics,
        ),
      );

  @override
  Future<Either<ApplicationError, bool>> unlockNote(
    String id,
    String password,
  ) async {
    final ok = password == this.password;
    unlocked = unlocked || ok;
    return _answer('unlockNote:$id', ok);
  }

  @override
  Future<Either<ApplicationError, bool>> unlockNotebook(
    String bookId,
    String password,
  ) async {
    final ok = password == this.password;
    unlocked = unlocked || ok;
    return _answer('unlockNotebook:$bookId', ok);
  }

  @override
  Future<Either<ApplicationError, bool>> unlock(String password) async {
    final ok = password == this.password;
    unlocked = unlocked || ok;
    return _answer('unlock', ok);
  }

  @override
  Future<Either<ApplicationError, Unit>> lock() async {
    unlocked = false;
    return _answer('lock', unit);
  }

  @override
  Future<Either<ApplicationError, Unit>> lockForBackground() async {
    unlocked = false;
    return _answer('lockForBackground', unit);
  }

  @override
  Future<Either<ApplicationError, bool>> isUnlocked() async =>
      Right<ApplicationError, bool>(unlocked);

  @override
  Future<Either<ApplicationError, bool>> isNoteUnlocked(String id) async =>
      Right<ApplicationError, bool>(unlocked);

  @override
  Future<Either<ApplicationError, bool>> notebookIsOpen(String bookId) async =>
      Right<ApplicationError, bool>(unlocked);

  @override
  Future<Either<ApplicationError, Saved>> lockNote(
    String id,
    String password,
  ) async {
    if (hasLockPassword && password != this.password) {
      return const Left<ApplicationError, Saved>(
        ApplicationError.auth(message: "That isn't the lock password."),
      );
    }
    hasLockPassword = true;
    unlocked = true;
    this.password = password;
    return _answer('lockNote:$id', Saved(id: id, rev: 'r2'));
  }

  @override
  Future<Either<ApplicationError, Saved>> lockNoteWithSession(String id) async {
    if (!unlocked) {
      return const Left<ApplicationError, Saved>(
        ApplicationError.locked(message: 'The note is locked.'),
      );
    }
    return _answer('lockNoteWithSession:$id', Saved(id: id, rev: 'r2'));
  }

  @override
  Future<Either<ApplicationError, Saved>> removeNoteLock(String id) async =>
      _answer('removeNoteLock:$id', Saved(id: id, rev: 'r3'));

  @override
  Future<Either<ApplicationError, int>> lockNotebook(
    String bookId, {
    String? password,
  }) async {
    if (password != null) {
      if (hasLockPassword && password != this.password) {
        return const Left<ApplicationError, int>(
          ApplicationError.auth(message: "That isn't the lock password."),
        );
      }
      hasLockPassword = true;
      unlocked = true;
      this.password = password;
    } else if (!unlocked) {
      return const Left<ApplicationError, int>(
        ApplicationError.locked(message: 'The note is locked.'),
      );
    }
    return _answer('lockNotebook:$bookId', 3);
  }

  @override
  Future<Either<ApplicationError, Unit>> removeNotebookLock(
    String bookId,
  ) async => _answer('removeNotebookLock:$bookId', unit);

  /// How many items kept the old password.
  int kept = 0;

  @override
  Future<Either<ApplicationError, int>> changePassword(
    String current,
    String next,
  ) async {
    if (current != password) {
      return const Left<ApplicationError, int>(
        ApplicationError.auth(message: "That isn't the lock password."),
      );
    }
    password = next;
    return _answer('changePassword', kept);
  }

  @override
  Future<Either<ApplicationError, String>> libraryId() async =>
      const Right<ApplicationError, String>('library-1');

  @override
  Future<Either<ApplicationError, bool>> biometricsOn() async =>
      Right<ApplicationError, bool>(biometrics);

  @override
  Future<Either<ApplicationError, int>> beginBiometricEnrollment() async {
    if (!unlocked) {
      return const Left<ApplicationError, int>(
        ApplicationError.locked(message: 'The note is locked.'),
      );
    }
    return _answer('beginBiometricEnrollment', 41);
  }

  @override
  Future<Either<ApplicationError, int?>> beginBiometricUnlock() async =>
      _answer<int?>('beginBiometricUnlock', biometrics ? 42 : null);

  @override
  Future<Either<ApplicationError, Unit>> turnOffBiometrics() async {
    biometrics = false;
    return _answer('turnOffBiometrics', unit);
  }

  @override
  Stream<LockEvent> lockEvents() => events.stream;
}

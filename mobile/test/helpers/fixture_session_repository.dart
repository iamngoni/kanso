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

  Future<void> close() async {
    await events.close();
    await progress.close();
  }

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

  /// When set, locking a note with the session waits for it: a lock that is
  /// still sealing its files.
  Completer<void>? gate;

  @override
  Future<Either<ApplicationError, Saved>> lockNoteWithSession(String id) async {
    await gate?.future;
    if (!unlocked) {
      return const Left<ApplicationError, Saved>(
        ApplicationError.locked(message: 'The note is locked.'),
      );
    }
    return _answer('lockNoteWithSession:$id', Saved(id: id, rev: 'r2'));
  }

  /// Whether Cancel was pressed ([cancelLock]): a Remove Lock that waited
  /// at the [gate] is then stopped, leaving the note locked.
  bool cancelled = false;

  @override
  Future<Either<ApplicationError, Saved>> removeNoteLock(String id) async {
    cancelled = false;
    await gate?.future;
    if (cancelled) {
      return const Left<ApplicationError, Saved>(
        ApplicationError.invalid(message: 'Locking was cancelled.'),
      );
    }
    return _answer('removeNoteLock:$id', Saved(id: id, rev: 'r3'));
  }

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

  /// What locking any note or notebook would seal.
  LockSummary summary = const LockSummary(files: 0, bytes: 0, shared: 0);

  /// The lock that stopped on a note, by default none.
  InterruptedLock? stopped;

  /// What [takeSealedFilesNotice] answers, once.
  bool filesNotice = false;

  /// What [takeCopyNotice] answers, once.
  LockCopyNotice? copyNotice;

  final StreamController<SealProgress> progress =
      StreamController<SealProgress>.broadcast();

  @override
  Future<Either<ApplicationError, LockSummary>> noteLockSummary(
    String id,
  ) async => Right<ApplicationError, LockSummary>(summary);

  @override
  Future<Either<ApplicationError, LockSummary>> notebookLockSummary(
    String bookId,
  ) async => Right<ApplicationError, LockSummary>(summary);

  @override
  Stream<SealProgress> lockProgress() => progress.stream;

  @override
  Future<Either<ApplicationError, Unit>> cancelLock() async {
    cancelled = true;
    gate?.complete();
    return _answer('cancelLock', unit);
  }

  @override
  Future<Either<ApplicationError, InterruptedLock?>> interruptedLock(
    String id,
  ) async => Right<ApplicationError, InterruptedLock?>(stopped);

  @override
  Future<Either<ApplicationError, Saved>> finishLocking(
    String id, {
    String? password,
  }) async {
    await gate?.future;
    if (!unlocked && password == null) {
      return const Left<ApplicationError, Saved>(
        ApplicationError.locked(message: 'The note is locked.'),
      );
    }
    if (password != null && password != this.password) {
      return const Left<ApplicationError, Saved>(
        ApplicationError.auth(message: "That isn't the lock password."),
      );
    }
    stopped = null;
    return _answer('finishLocking:$id', Saved(id: id, rev: 'r2'));
  }

  @override
  Future<Either<ApplicationError, Unit>> undoLocking(
    String id, {
    String? password,
  }) async {
    if (!unlocked && password == null) {
      return const Left<ApplicationError, Unit>(
        ApplicationError.locked(message: 'The note is locked.'),
      );
    }
    if (password != null && password != this.password) {
      return const Left<ApplicationError, Unit>(
        ApplicationError.auth(message: "That isn't the lock password."),
      );
    }
    stopped = null;
    return _answer('undoLocking:$id', unit);
  }

  @override
  Future<Either<ApplicationError, bool>> takeSealedFilesNotice() async {
    final answer = filesNotice;
    filesNotice = false;
    return Right<ApplicationError, bool>(answer);
  }

  @override
  Future<Either<ApplicationError, LockCopyNotice?>> takeCopyNotice() async {
    final answer = copyNotice;
    copyNotice = null;
    return Right<ApplicationError, LockCopyNotice?>(answer);
  /// The locked notes waiting to have their attachments sealed, by default
  /// none.
  FilesMigration migration = const FilesMigration(
    notes: 0,
    stopped: 0,
    postponed: false,
    counted: false,
    files: 0,
    bytes: 0,
    shared: 0,
  );

  /// What a pass seals when it is run.
  MigrationOutcome migrated = const MigrationOutcome(
    notes: 0,
    sealed: 0,
    copied: 0,
    unopened: 0,
    failed: 0,
    stopped: false,
  );

  /// When set, a pass waits for it before it finishes.
  Completer<void>? migrationGate;

  @override
  Future<Either<ApplicationError, FilesMigration>> filesMigration() async =>
      Right<ApplicationError, FilesMigration>(migration);

  @override
  Future<Either<ApplicationError, Unit>> postponeFilesMigration() async {
    migration = FilesMigration(
      notes: migration.notes,
      stopped: migration.stopped,
      postponed: true,
      counted: migration.counted,
      files: migration.files,
      bytes: migration.bytes,
      shared: migration.shared,
    );
    return _answer('postponeFilesMigration', unit);
  }

  ApplicationError? _migrationDenied(String? password) {
    if (!unlocked && password == null) {
      return const ApplicationError.locked(message: 'The notes are closed.');
    }
    if (password != null && password != this.password) {
      return const ApplicationError.auth(
        message: "That isn't the lock password.",
      );
    }
    return null;
  }

  @override
  Future<Either<ApplicationError, MigrationOutcome>> sealLockedAttachments({
    String? password,
  }) async {
    final denied = _migrationDenied(password);
    if (denied != null) {
      return Left<ApplicationError, MigrationOutcome>(denied);
    }
    await migrationGate?.future;
    migration = const FilesMigration(
      notes: 0,
      stopped: 0,
      postponed: false,
      counted: false,
      files: 0,
      bytes: 0,
      shared: 0,
    );
    return _answer('sealLockedAttachments', migrated);
  }

  @override
  Future<Either<ApplicationError, int>> undoFilesMigration({
    String? password,
  }) async {
    final denied = _migrationDenied(password);
    if (denied != null) {
      return Left<ApplicationError, int>(denied);
    }
    migration = FilesMigration(
      notes: migration.notes,
      stopped: 0,
      postponed: migration.postponed,
      counted: migration.counted,
      files: migration.files,
      bytes: migration.bytes,
      shared: migration.shared,
    );
    return _answer('undoFilesMigration', 2);
  }
}

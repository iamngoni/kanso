// Kanso — lock_mode_for.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';
import '../models/data/lock_mode.dart';

/// How locking goes for the lock [status]: a first lock makes the password,
/// closed notes ask for it, open notes only confirm.
LockMode lockModeFor(LockStatus status) {
  if (!status.hasLockPassword) {
    return LockMode.setPassword;
  }
  return status.unlocked ? LockMode.confirm : LockMode.askPassword;
}

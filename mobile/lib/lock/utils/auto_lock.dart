// Kanso — auto_lock.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// How many minutes the locked notes stay open after they were opened or a
/// locked note was last saved. It is the Rust core's `AUTO_LOCK`; the
/// desktop has no setting for it, so neither do the phones.
const int autoLockMinutes = 10;

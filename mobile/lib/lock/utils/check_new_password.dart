// Kanso — check_new_password.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../models/data/password_check.dart';

/// The fewest characters a lock password has.
const int minLockPasswordLength = 8;

/// Whether [password], typed again as [again], can be a new lock password.
/// Two different texts are a mismatch whatever their length; the same text
/// that is too short is too short.
PasswordCheck checkNewPassword(String password, String again) {
  if (password != again) {
    return PasswordCheck.mismatch;
  }
  if (password.runes.length < minLockPasswordLength) {
    return PasswordCheck.tooShort;
  }
  return PasswordCheck.ok;
}

// Kanso — seal_progress_rules.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';

/// Files from which a lock shows its progress sheet.
const int sheetFiles = 4;

/// Bytes from which a lock shows its progress sheet.
const int sheetBytes = 8 * 1024 * 1024;

/// Whether sealing [files] files of [bytes] bytes is long enough to show a
/// sheet that counts them. The desktop draws the same line.
bool wantsProgressSheet({required int files, required int bytes}) =>
    files >= sheetFiles || bytes >= sheetBytes;

/// Whether [progress] is for work long enough to show the sheet.
bool progressWantsSheet(SealProgress progress) =>
    wantsProgressSheet(files: progress.total, bytes: progress.bytesTotal);

/// How far along [progress] is, from 0 to 1: by files, as the desktop's bar.
double sealFraction(SealProgress progress) {
  if (progress.total <= 0) {
    return 0;
  }
  return (progress.done / progress.total).clamp(0.0, 1.0);
}

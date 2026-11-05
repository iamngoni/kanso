// Kanso — copy_notice_words.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';
import '../../gen/l10n/app_localizations.dart';

/// What is said after a lock or Remove Lock copied files that other notes
/// also show (the desktop's `copied_words` and `unsealed_words`); null when
/// it copied none.
String? copyNoticeWords(AppLocalizations l10n, LockCopyNotice? notice) {
  if (notice == null || notice.copied == 0) {
    return null;
  }
  return notice.removed
      ? l10n.lockUnsealedMessage(notice.copied)
      : l10n.lockCopiedMessage(notice.copied);
}

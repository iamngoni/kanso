// Kanso — files_migration_words.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';
import '../../gen/l10n/app_localizations.dart';
import '../../note/views/widgets/attachment_sheet.dart' show byteSize;

/// Whether the first-launch prompt is due: notes wait to be sealed and
/// "Later" was not chosen.
bool wantsMigrationPrompt(FilesMigration info) =>
    info.notes > 0 && !info.postponed;

/// Whether the quiet reminder is due: notes wait and "Later" was chosen.
bool wantsMigrationReminder(FilesMigration info) =>
    info.notes > 0 && info.postponed;

/// Whether a pass stopped part way and waits to be finished or undone.
bool migrationStopped(FilesMigration info) => info.stopped > 0;

/// What the prompt says: the design's sentence, how many locked notes wait,
/// and what sealing will do — counted when the notes are open, else said to
/// be counted once the password opens them.
String migrationPromptMessage(AppLocalizations l10n, FilesMigration info) {
  final parts = <String>[
    l10n.lockMigrationPrompt,
    l10n.lockMigrationNotes(info.notes),
    if (info.counted) ...<String>[
      l10n.lockMigrationCounted(info.files, byteSize(info.bytes)),
      if (info.shared > 0) l10n.lockMigrationShared(info.shared),
    ] else
      l10n.lockMigrationUncounted,
  ];
  return parts.join(' ');
}

/// What is said when a pass is over.
String migrationDoneMessage(AppLocalizations l10n, MigrationOutcome outcome) {
  final parts = <String>[
    if (outcome.stopped)
      l10n.lockMigrationStopped
    else if (outcome.sealed + outcome.copied == 0)
      l10n.lockMigrationNothing(outcome.notes)
    else
      l10n.lockMigrationDone(outcome.sealed + outcome.copied, outcome.notes),
    if (outcome.copied > 0) l10n.lockMigrationCopies(outcome.copied),
    if (outcome.unopened > 0) l10n.lockMigrationUnopened(outcome.unopened),
    if (outcome.failed > 0) l10n.lockMigrationFailedNotes(outcome.failed),
  ];
  return parts.join(' ');
}

/// Whether [outcome] leaves nothing for a later pass.
bool migrationComplete(MigrationOutcome outcome) =>
    !outcome.stopped && outcome.unopened == 0 && outcome.failed == 0;

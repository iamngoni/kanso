// Kanso — files_migration_words_test.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Glados3, Glados2, IntAnys, any, BoolAny;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/gen/l10n/app_localizations_en.dart';
import 'package:kanso/lock/utils/files_migration_words.dart';
import 'package:kanso/note/views/widgets/attachment_sheet.dart' show byteSize;

import '../../helpers/property.dart';

final AppLocalizationsEn _l10n = AppLocalizationsEn();

FilesMigration _info({
  int notes = 3,
  int stopped = 0,
  bool postponed = false,
  bool counted = false,
  int files = 0,
  int bytes = 0,
  int shared = 0,
}) => FilesMigration(
  notes: notes,
  stopped: stopped,
  postponed: postponed,
  counted: counted,
  files: files,
  bytes: bytes,
  shared: shared,
);

void main() {
  group('the prompt', () {
    test('uses the design sentence and counts the notes', () {
      final words = migrationPromptMessage(_l10n, _info());
      expect(
        words,
        startsWith(
          "Attachments of your locked notes aren't sealed yet. "
          'Enter your lock password to seal them now.',
        ),
      );
      expect(words, contains('3 locked notes'));
      expect(words, contains('once the password opens the notes'));
    });

    test('counts the attachments and size when the notes are open', () {
      final words = migrationPromptMessage(
        _l10n,
        _info(counted: true, files: 12, bytes: 84000000, shared: 3),
      );
      expect(words, contains('12 attachments (84 MB) will be sealed.'));
      expect(words, contains('3 attachments are also used by other notes'));
      expect(words, isNot(contains('once the password opens')));
    });

    Glados3<int, int, int>(
      any.intInRange(1, 400),
      any.intInRange(0, 600),
      any.intInRange(0, 1 << 30),
      kansoExploreConfig(),
    ).test(
      'names exactly the notes, files and size it was given',
      (notes, files, bytes) {
        final words = migrationPromptMessage(
          _l10n,
          _info(notes: notes, counted: true, files: files, bytes: bytes),
        );
        expect(words, contains(_l10n.lockMigrationNotes(notes)));
        expect(
          words,
          contains(_l10n.lockMigrationCounted(files, byteSize(bytes))),
        );
      },
    );
  });

  group('when to ask and when to remind', () {
    test('ask until Later, then remind, and nothing once sealed', () {
      expect(wantsMigrationPrompt(_info()), isTrue);
      expect(wantsMigrationReminder(_info()), isFalse);
      expect(wantsMigrationPrompt(_info(postponed: true)), isFalse);
      expect(wantsMigrationReminder(_info(postponed: true)), isTrue);
      expect(wantsMigrationPrompt(_info(notes: 0)), isFalse);
      expect(wantsMigrationReminder(_info(notes: 0, postponed: true)), isFalse);
    });

    Glados2<int, bool>(
      any.intInRange(0, 50),
      any.bool,
      kansoExploreConfig(),
    ).test(
      'a waiting library is asked or reminded, never both, never neither',
      (notes, postponed) {
        final info = _info(notes: notes, postponed: postponed);
        final both = wantsMigrationPrompt(info) && wantsMigrationReminder(info);
        final neither =
            !wantsMigrationPrompt(info) && !wantsMigrationReminder(info);
        expect(both, isFalse);
        expect(neither, notes == 0);
      },
    );
  });

  group('how it went', () {
    test('says what was sealed and copied', () {
      const outcome = MigrationOutcome(
        notes: 3,
        sealed: 4,
        copied: 2,
        unopened: 1,
        failed: 0,
        stopped: false,
      );
      final words = migrationDoneMessage(_l10n, outcome);
      expect(words, startsWith('Sealed 6 attachments in 3 locked notes.'));
      expect(words, contains('2 attachments also used by other notes'));
      expect(words, contains('1 locked note uses another lock password'));
      expect(migrationComplete(outcome), isFalse);
    });

    test('says when nothing needed sealing, or it stopped', () {
      const nothing = MigrationOutcome(
        notes: 1,
        sealed: 0,
        copied: 0,
        unopened: 0,
        failed: 0,
        stopped: false,
      );
      expect(
        migrationDoneMessage(_l10n, nothing),
        'Checked 1 locked note: no attachments needed sealing.',
      );
      expect(migrationComplete(nothing), isTrue);
      const stopped = MigrationOutcome(
        notes: 1,
        sealed: 1,
        copied: 0,
        unopened: 0,
        failed: 0,
        stopped: true,
      );
      expect(
        migrationDoneMessage(_l10n, stopped),
        contains('Sealing stopped.'),
      );
      expect(migrationComplete(stopped), isFalse);
    });

    Glados3<int, int, int>(
      any.intInRange(0, 200),
      any.intInRange(0, 200),
      any.intInRange(0, 200),
      kansoExploreConfig(),
    ).test(
      'the count in the words is sealed plus copied',
      (notes, sealed, copied) {
        final words = migrationDoneMessage(
          _l10n,
          MigrationOutcome(
            notes: notes,
            sealed: sealed,
            copied: copied,
            unopened: 0,
            failed: 0,
            stopped: false,
          ),
        );
        if (sealed + copied > 0) {
          expect(
            words,
            contains(_l10n.lockMigrationDone(sealed + copied, notes)),
          );
        } else {
          expect(words, contains(_l10n.lockMigrationNothing(notes)));
        }
      },
    );
  });
}

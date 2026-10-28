// Kanso — note_window_title_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show BoolAny, Glados2, Glados3, StringAnys, any;
import 'package:kanso/window/models/data/note_window_launch.dart';
import 'package:kanso/window/utils/note_window_title.dart';

import '../../helpers/property.dart';

void main() {
  Glados2<String, bool>(
    any.letterOrDigits,
    any.bool,
    kansoExploreConfig(),
  ).test('a locked note is named "Locked note", whatever its title', (
    title,
    locked,
  ) {
    final name = noteWindowTitle(
      title: title,
      locked: locked,
      untitled: 'Untitled',
      lockedTitle: 'Locked note',
    );
    if (locked) {
      expect(name, 'Locked note');
      // The title is never in it (unless it is the very words).
      if (title.isNotEmpty && !'Locked note'.contains(title)) {
        expect(name, isNot(contains(title)));
      }
    } else if (title.trim().isEmpty) {
      expect(name, 'Untitled');
    } else {
      expect(name, title.trim());
    }
  });

  group('NoteWindowLaunch', () {
    test('reads what the native side starts the entry point with', () {
      final launch = NoteWindowLaunch.fromArgs(<String>[
        'n1',
        'integration',
        '/x',
      ]);
      expect(
        launch,
        const NoteWindowLaunch(
          noteId: 'n1',
          environment: 'integration',
          libraryFolder: '/x',
        ),
      );
      expect(NoteWindowLaunch.fromArgs(launch!.args), launch);
    });

    test('a note alone is the production app on its own library', () {
      expect(
        NoteWindowLaunch.fromArgs(<String>['n1']),
        const NoteWindowLaunch(noteId: 'n1', environment: 'production'),
      );
    });

    test('no note is no window', () {
      expect(NoteWindowLaunch.fromArgs(const <String>[]), isNull);
      expect(NoteWindowLaunch.fromArgs(<String>['']), isNull);
    });

    Glados3<String, String, String>(
      any.nonEmptyLetterOrDigits,
      any.nonEmptyLetterOrDigits,
      any.letterOrDigits,
      kansoExploreConfig(),
    ).test('the arguments read back as they were made', (id, env, folder) {
      final launch = NoteWindowLaunch(
        noteId: id,
        environment: env,
        libraryFolder: folder,
      );
      expect(NoteWindowLaunch.fromArgs(launch.args), launch);
    });
  });
}

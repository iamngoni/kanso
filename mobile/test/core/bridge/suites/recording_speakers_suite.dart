// Kanso — recording_speakers_suite.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/recording_speakers_types.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

const String _body =
    '# Meeting\n\n'
    '[Audio](inkdrop://file:abc123)\n\n'
    '<!-- kanso:transcript -->\n'
    '[00:00](kanso-audio://file:abc123?t=0) You: Morning.\n\n'
    '[00:20](kanso-audio://file:abc123?t=20) Speaker 1: Beta ships.\n\n'
    '[00:40](kanso-audio://file:abc123?t=40) Speaker 1: Agreed.\n'
    '<!-- /kanso:transcript -->\n';

/// Rename Speakers, through the bridge: the names in a transcript listed,
/// renamed in one change, and undone.
void recordingSpeakersSuite(OpenLibrary open) {
  group('rename speakers', () {
    test('names are listed, renamed and the rename undone', () async {
      final library = await open();
      final created = await library.create(
        note: const NewNote(title: 'Meeting', body: _body),
      );
      final speakers = library.recordingSpeakers();
      final names = await speakers.names(id: created.id);
      expect(names.map((n) => n.name), <String>['You', 'Speaker 1']);
      expect(names.last.paragraphs, 2);

      final renamed = await speakers.rename(
        id: created.id,
        changes: const <SpeakerChange>[
          SpeakerChange(from: 'Speaker 1', to: 'Priya'),
        ],
      );
      expect(renamed.undo, isNotNull);
      final after = await library.note(id: created.id);
      expect(after.body, contains('Priya: Beta ships.'));
      expect(after.body, isNot(contains('Speaker 1')));
    });

    test('a note with no transcript has nobody to rename', () async {
      final library = await open();
      final plain = await library.create(
        note: const NewNote(title: 'Plain', body: 'text'),
      );
      expect(await library.recordingSpeakers().names(id: plain.id), isEmpty);
    });
  });
}

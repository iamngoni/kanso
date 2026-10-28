// Kanso — split_voice_note_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Any, AnyUtils, Generator, GeneratorUtils, Glados, ListAnys, any;
import 'package:kanso/dictation/utils/split_voice_note.dart';

import '../../helpers/property.dart';

extension _Inputs on Any {
  /// Lines of a few words, with the marks that make a title or a list.
  Generator<String> get transcript => listWithLengthInRange<String>(
    0,
    12,
    choose(<String>[
      'buy',
      'milk',
      ' ',
      '\n',
      '#',
      '- ',
      '>',
      'xxxxxxxxxxxxxxxxxxxxxxxxxxxxxx',
    ]),
  ).map((parts) => parts.join());
}

void main() {
  group('splitVoiceNote', () {
    test('is nothing for text of spaces', () {
      expect(splitVoiceNote(''), isNull);
      expect(splitVoiceNote(' \n  '), isNull);
    });

    test('takes a short first line as the title and the rest as the body', () {
      final note = splitVoiceNote('Groceries\n\nMilk and bread.')!;
      expect(note.title, 'Groceries');
      expect(note.body, 'Milk and bread.');
    });

    test('a single short line is a title with no body', () {
      final note = splitVoiceNote('  Call Ana tomorrow.  ')!;
      expect(note.title, 'Call Ana tomorrow.');
      expect(note.body, isEmpty);
    });

    test('takes the text of a heading as the title', () {
      final note = splitVoiceNote('## Plan\nFirst, the roof.')!;
      expect(note.title, 'Plan');
      expect(note.body, 'First, the roof.');
    });

    test('cuts a long first line at a space and keeps all of the text', () {
      final line = List<String>.filled(30, 'milk').join(' ');
      final note = splitVoiceNote(line)!;
      expect(note.title, endsWith('…'));
      expect(
        note.title.runes.length,
        lessThanOrEqualTo(voiceNoteTitleLength + 1),
      );
      expect(note.body, line);
    });

    test('a first line that starts a list is cut and the body keeps it', () {
      final note = splitVoiceNote('- milk\n- bread')!;
      expect(note.title, 'milk');
      expect(note.body, '- milk\n- bread');
    });
  });

  group('properties', () {
    Glados<String>(
      any.transcript,
      kansoExploreConfig(),
    ).test('nothing said is lost, and the title is one line', (text) {
      final note = splitVoiceNote(text);
      if (text.trim().isEmpty) {
        expect(note, isNull);
        return;
      }
      expect(note, isNotNull);
      expect(note!.title, isNotEmpty);
      expect(note.title.contains('\n'), isFalse);
      expect(
        note.title.runes.length,
        lessThanOrEqualTo(voiceNoteTitleLength + 1),
      );
      // Every word is in the title or the body.
      final kept = '${note.title} ${note.body}';
      for (final word
          in text.split(RegExp(r'[\s#>-]+')).where((w) => w.isNotEmpty)) {
        final head = word.length > 20 ? word.substring(0, 20) : word;
        expect(kept, contains(head));
      }
    });
  });
}

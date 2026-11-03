// Kanso — home_note_text_test.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/notebook_home/views/widgets/home_note_text.dart';

import '../../helpers/pump.dart';

void main() {
  testWidgets('a Home note reads as preview does, without its marks', (
    tester,
  ) async {
    await pumpKanso(
      tester,
      const SizedBox(
        width: 360,
        child: HomeNoteText(
          '# Research\n\nLiterature for the **thesis**, see [[Chapter plan]].\n'
          '- Drafts in `Drafts`\n- [ ] Email Dr Banda\n- [x] Pick a dataset',
        ),
      ),
    );
    String all() => tester
        .widgetList<RichText>(find.byType(RichText))
        .map((r) => r.text.toPlainText())
        .join('\n');
    final text = all();
    for (final shown in <String>[
      'Research',
      'Literature for the thesis, see Chapter plan.',
      'Drafts in Drafts',
      '☐',
      'Email Dr Banda',
      '☑',
      '•',
    ]) {
      expect(text, contains(shown));
    }
    for (final mark in <String>['#', '**', '[[', '`', '[ ]', '[x]']) {
      expect(text, isNot(contains(mark)), reason: mark);
    }
  });
}

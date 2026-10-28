// Kanso — dictation_editor_extras_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/dictation/models/data/dictation_signal.dart';
import 'package:kanso/dictation/models/repos/impl/plain_dictation_text_rules.dart';
import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';
import 'package:kanso/dictation/utils/dictation_editor_extras.dart';
import 'package:mocktail/mocktail.dart';

import '../helpers/fake_text_target.dart';
import '../helpers/mock_dictation_repository.dart';

void main() {
  late SpokenRepository spoken;
  late DictationBloc bloc;
  late DictationEditorExtras extras;
  late FakeTextTarget note;

  setUp(() {
    spoken = SpokenRepository();
    bloc = DictationBloc(spoken.repository, PlainDictationTextRules());
    extras = DictationEditorExtras(
      bloc: bloc,
      rules: PlainDictationTextRules(),
    );
    note = FakeTextTarget('Notes:');
  });

  tearDown(() async {
    await extras.dispose();
    await bloc.close();
    spoken.close();
  });

  Future<void> hear(DictationSignal signal) async {
    spoken.signals.add(signal);
    await pumpEventQueue();
  }

  Future<void> start() async {
    extras.toggle(note);
    await pumpEventQueue();
    await hear(const DictationSignal.listening());
  }

  test('offers one key, after the Markdown keys, and no selection actions', () {
    final keys = extras.accessoryKeys(note);
    expect(keys, hasLength(1));
    expect(keys.single.id, 'dictation');
    expect(keys.single.icon, KansoIcons.mic);
    expect(keys.single.onTap, isNotNull);
    expect(extras.selectionActions(note), isEmpty);
  });

  test('the key is lit while the microphone is on', () async {
    final active = extras.accessoryKeys(note).single.active!;
    expect(active.value, isFalse);
    await start();
    expect(active.value, isTrue);
    await hear(const DictationSignal.ended());
    expect(active.value, isFalse);
  });

  test('words being spoken show in the preview and not in the note', () async {
    await start();
    await hear(const DictationSignal.partial('buy milk'));
    expect(extras.preview.value, ' buy milk');
    expect(note.text, 'Notes:');
    expect(note.undoSteps, 0);
  });

  test('each finished phrase is one undo step in the note', () async {
    await start();
    await hear(const DictationSignal.partial('buy milk'));
    await hear(const DictationSignal.finalText('Buy milk.'));
    expect(note.text, 'Notes: Buy milk.');
    await hear(const DictationSignal.partial('and eggs'));
    expect(note.text, 'Notes: Buy milk.');
    await hear(const DictationSignal.finalText('And eggs.'));
    expect(note.text, 'Notes: Buy milk. And eggs.');
    expect(note.undoSteps, 2);
  });

  test(
    'a queued batch still makes one edit for each finished phrase',
    () async {
      extras.toggle(note);
      await extras.handle(
        note,
        const DictationState(
          segments: <String>['Buy milk.', 'And eggs.'],
        ),
        extras.generation,
      );
      expect(note.text, 'Notes: Buy milk. And eggs.');
      expect(note.undoSteps, 2);
    },
  );

  test('what was heard but not finished is written when it stops', () async {
    await start();
    await hear(const DictationSignal.partial('last words'));
    extras.toggle(note);
    await pumpEventQueue();
    verify(spoken.repository.stop).called(1);
    await hear(const DictationSignal.ended());
    expect(note.text, 'Notes: last words');
    expect(extras.preview.value, isEmpty);
  });

  test('writes nothing more once the note is released', () async {
    await start();
    await hear(const DictationSignal.finalText('One.'));
    extras.release(note);
    await pumpEventQueue();
    verify(spoken.repository.cancel).called(1);
    await hear(const DictationSignal.finalText('Two.'));
    expect(note.text, 'Notes: One.');
  });

  test('a note the key was not pressed in is not released', () async {
    await start();
    extras.release(FakeTextTarget(''));
    await pumpEventQueue();
    verifyNever(spoken.repository.cancel);
  });

  testWidgets('pressing the key starts dictation in the note', (tester) async {
    late BuildContext context;
    await tester.pumpWidget(
      Builder(
        builder: (inner) {
          context = inner;
          return const SizedBox.shrink();
        },
      ),
    );
    await tester.runAsync(() async {
      extras.accessoryKeys(note).single.onTap!(context, note);
      await pumpEventQueue();
    });
    verify(
      () => spoken.repository.start(localeTag: any(named: 'localeTag')),
    ).called(1);
  });
}

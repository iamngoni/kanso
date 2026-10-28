// Kanso — rust_ai_repository_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/ai.dart' as bridge;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/ai_note_ref.dart';
import 'package:kanso/core/models/data/ai_settings_change.dart';
import 'package:kanso/core/models/data/ai_test_result.dart';
import 'package:kanso/core/models/data/ask_update.dart';
import 'package:kanso/core/models/data/assist_action.dart';
import 'package:kanso/core/models/data/assist_update.dart';
import 'package:kanso/core/models/data/layout_class.dart';
import 'package:kanso/core/models/repos/impl/rust_ai_repository.dart';
import 'package:kanso/core/router/layout_class_notifier.dart';
import 'package:mocktail/mocktail.dart';

class MockKansoAi extends Mock implements bridge.KansoAi {}

class MockConversation extends Mock implements bridge.KansoAskConversation {}

class MockRun extends Mock implements bridge.KansoAiRun {}

const _view = bridge.AiSettingsView(
  enabled: true,
  provider: 'anthropic',
  baseUrl: '',
  model: '',
  region: '',
  hasKey: true,
  host: 'api.anthropic.com',
  isRemote: true,
);

void main() {
  late MockKansoAi ai;
  late MockConversation conversation;
  late MockRun run;

  RustAiRepository build({bool tablet = false}) => RustAiRepository(
    ai,
    LayoutClassNotifier.starting(LayoutClass.single, tabletDevice: tablet),
  );

  setUpAll(() {
    registerFallbackValue(
      const bridge.AiSettingsInput(
        enabled: false,
        provider: '',
        baseUrl: '',
        model: '',
        region: '',
      ),
    );
  });

  setUp(() {
    ai = MockKansoAi();
    conversation = MockConversation();
    run = MockRun();
    when(() => ai.conversation()).thenReturn(conversation);
    when(() => ai.newRun()).thenReturn(run);
    when(() => ai.settings()).thenAnswer((_) async => _view);
    when(
      () => ai.privacyLine(device: any(named: 'device')),
    ).thenAnswer((i) async => 'Goes from ${i.namedArguments[#device]}.');
  });

  group('RustAiRepository', () {
    test(
      'settings carry the privacy line in the words of the device',
      () async {
        final phone = (await build().settings()).toOption().toNullable()!;
        expect(phone.privacy, 'Goes from this phone.');
        expect(phone.hasKey, isTrue);
        expect(phone.host, 'api.anthropic.com');
        final tablet = (await build(
          tablet: true,
        ).settings()).toOption().toNullable()!;
        expect(tablet.privacy, 'Goes from this tablet.');
      },
    );

    test('saving settings passes them on and announces the result', () async {
      when(() => ai.setSettings(input: any(named: 'input'))).thenAnswer(
        (_) async => _view,
      );
      final repo = build();
      final heard = repo.settingsChanges().first;
      await repo.saveSettings(
        const AiSettingsChange(
          enabled: true,
          provider: 'anthropic',
          model: 'm',
        ),
      );
      expect((await heard).enabled, isTrue);
      final input =
          verify(
                () => ai.setSettings(input: captureAny(named: 'input')),
              ).captured.single
              as bridge.AiSettingsInput;
      expect(input.model, 'm');
    });

    test('a key goes to the core and comes back only as hasKey', () async {
      when(
        () => ai.setKey(
          provider: any(named: 'provider'),
          key: any(named: 'key'),
        ),
      ).thenAnswer((_) async {});
      final result = await build().setKey(provider: 'anthropic', key: 'sk-x');
      verify(() => ai.setKey(provider: 'anthropic', key: 'sk-x')).called(1);
      expect(result.toOption().toNullable()!.hasKey, isTrue);
      expect(result.toString(), isNot(contains('sk-x')));
    });

    test('Test says what the model said, or why not', () async {
      when(() => ai.test()).thenAnswer((_) async => 'OK');
      expect(
        (await build().test()).toOption().toNullable(),
        const AiTestResult.ok(said: 'OK'),
      );
      when(() => ai.test()).thenThrow(
        const KansoError.network(message: 'No route to host.'),
      );
      expect(
        (await build().test()).toOption().toNullable(),
        const AiTestResult.failed(message: 'No route to host.'),
      );
    });

    test('lists the providers and the actions', () async {
      when(() => ai.providers()).thenReturn(const <bridge.AiProviderInfo>[
        bridge.AiProviderInfo(
          key: 'bedrock',
          label: 'Bedrock',
          description: 'd',
          defaultBaseUrl: 'u',
          defaultModel: 'm',
          defaultRegion: 'us-east-1',
          needsKey: true,
          acceptsKey: true,
        ),
      ]);
      final providers = (await build().providers()).toOption().toNullable()!;
      expect(providers.single.usesRegion, isTrue);
      when(() => ai.actions()).thenReturn(const <bridge.AiAction>[
        bridge.AiAction(
          key: 'continue',
          label: 'Continue Writing',
          noun: 'Continued',
          replaces: false,
        ),
        bridge.AiAction(
          key: 'improve',
          label: 'Improve Writing',
          noun: 'Improved',
          replaces: true,
        ),
        bridge.AiAction(key: 'other', label: 'x', noun: 'x', replaces: false),
      ]);
      final actions = build().assistActions();
      expect(actions.map((a) => a.action), <AssistAction>[
        AssistAction.continueWriting,
        AssistAction.improve,
      ]);
      expect(actions.first.needsSelection, isFalse);
      expect(actions.last.replaces, isTrue);
    });

    test('an answer is searched, written and done, with its notes', () async {
      when(() => conversation.ask(question: 'q')).thenAnswer(
        (_) => Stream<bridge.AskEvent>.fromIterable(const <bridge.AskEvent>[
          bridge.AskEvent.searching(),
          bridge.AskEvent.status(line: 'Thinking…'),
          bridge.AskEvent.used(
            notes: <bridge.AskNote>[
              bridge.AskNote(id: 'a:1', title: 'A'),
            ],
          ),
          bridge.AskEvent.answer(markdown: 'Par'),
          bridge.AskEvent.done(
            markdown: 'Part [A](kanso://note/a:1)',
            notes: <bridge.AskNote>[
              bridge.AskNote(id: 'a:1', title: 'A'),
            ],
          ),
        ]),
      );
      const a = AiNoteRef(id: 'a:1', title: 'A');
      expect(await build().ask('q').toList(), const <AskUpdate>[
        AskUpdate.searching(),
        AskUpdate.thinking(),
        AskUpdate.usedNotes(notes: <AiNoteRef>[a]),
        AskUpdate.answer(text: 'Par'),
        AskUpdate.done(
          answer: 'Part [A](kanso://note/a:1)',
          used: <AiNoteRef>[a],
        ),
      ]);
    });

    test('nothing matching, a cut off and a failure are updates', () async {
      when(() => conversation.ask(question: 'q')).thenAnswer(
        (_) => Stream<bridge.AskEvent>.fromIterable(const <bridge.AskEvent>[
          bridge.AskEvent.nothingMatches(message: 'n'),
          bridge.AskEvent.cutOff(message: 'c'),
          bridge.AskEvent.failed(message: 'AI is turned off.'),
          bridge.AskEvent.stopped(),
          bridge.AskEvent.searching(),
        ]),
      );
      // Nothing is heard after the core says it stopped.
      expect(await build().ask('q').toList(), const <AskUpdate>[
        AskUpdate.nothingMatches(),
        AskUpdate.cutOff(),
        AskUpdate.error(message: 'AI is turned off.'),
      ]);
    });

    test('stop and clear reach the conversation', () async {
      final repo = build();
      await repo.stop();
      await repo.clear();
      verify(() => conversation.stop()).called(1);
      verify(() => conversation.clear()).called(1);
    });

    test('a suggestion streams, and a kept out note says so', () async {
      when(
        () => ai.assist(
          id: 'n1',
          action: 'continue',
          before: 'b',
          selection: 's',
          run: run,
        ),
      ).thenAnswer(
        (_) => Stream<bridge.AssistEvent>.fromIterable(
          const <bridge.AssistEvent>[
            bridge.AssistEvent.status(line: 's'),
            bridge.AssistEvent.text(text: 'We'),
            bridge.AssistEvent.done(text: 'We ship.'),
          ],
        ),
      );
      final updates = await build()
          .assist(
            noteId: 'n1',
            action: AssistAction.continueWriting,
            before: 'b',
            selection: 's',
          )
          .toList();
      expect(updates, const <AssistUpdate>[
        AssistUpdate.text(text: 'We'),
        AssistUpdate.done(suggestion: 'We ship.'),
      ]);
      verify(
        () => ai.assist(
          id: 'n1',
          action: 'continue',
          before: 'b',
          selection: 's',
          run: run,
        ),
      ).called(1);

      when(() => ai.summarize(id: 'n1', run: run)).thenAnswer(
        (_) => Stream<bridge.AssistEvent>.fromIterable(
          const <bridge.AssistEvent>[
            bridge.AssistEvent.failed(message: keptOutMessage),
          ],
        ),
      );
      expect(await build().summarize('n1').toList(), const <AssistUpdate>[
        AssistUpdate.keptOut(),
      ]);
    });

    test('a suggestion nobody waits for any more stops its run', () async {
      final controller = StreamController<bridge.AssistEvent>();
      addTearDown(controller.close);
      when(() => ai.summarize(id: 'n1', run: run)).thenAnswer(
        (_) => controller.stream,
      );
      final sub = build().summarize('n1').listen((_) {});
      controller.add(const bridge.AssistEvent.text(text: 'x'));
      await Future<void>.delayed(Duration.zero);
      await sub.cancel();
      verify(() => run.stop()).called(1);
    });

    test('eligibility is the core’s word', () async {
      when(() => ai.noteAllowed(id: 'n1')).thenAnswer((_) async => false);
      expect((await build().isEligible('n1')).toOption().toNullable(), isFalse);
    });
  });
}

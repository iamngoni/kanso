// Kanso — ai_suite.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/ai.dart';

import 'open_library.dart';

/// What the model on the loopback server says.
const String modelAnswer = 'The yearly plan costs 40 dollars, see [[Pricing]].';

/// A model on this device: an OpenAI-compatible server on an ephemeral port
/// of the loopback address, answering every question with [answer]. The
/// deciding turn of a question gets it whole; a streamed request (the
/// assistant, the summary, the test) gets it in two pieces.
class LoopbackModel {
  LoopbackModel._(this._server, this.answer);

  final HttpServer _server;
  final String answer;

  /// The request bodies the server was sent.
  final List<String> received = <String>[];

  static Future<LoopbackModel> start({String answer = modelAnswer}) async {
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
    final model = LoopbackModel._(server, answer);
    server.listen(model._serve);
    return model;
  }

  /// The address to give the Custom provider.
  String get baseUrl => 'http://127.0.0.1:${_server.port}/v1';

  Future<void> close() => _server.close(force: true);

  Future<void> _serve(HttpRequest request) async {
    final body = await utf8.decoder.bind(request).join();
    received.add(body);
    final streamed =
        (jsonDecode(body) as Map<String, dynamic>)['stream'] == true;
    final response = request.response;
    if (streamed) {
      response.headers.contentType = ContentType('text', 'event-stream');
      final cut = answer.length ~/ 2;
      for (final piece in <String>[
        answer.substring(0, cut),
        answer.substring(cut),
      ]) {
        response.write(
          'data: ${jsonEncode({
            'choices': [
              {
                'index': 0,
                'delta': {'content': piece},
              },
            ],
          })}\n\n',
        );
      }
      response.write('data: [DONE]\n\n');
    } else {
      response.headers.contentType = ContentType.json;
      response.write(
        jsonEncode({
          'choices': [
            {
              'message': {'content': answer},
            },
          ],
        }),
      );
    }
    await response.close();
  }
}

AiSettingsInput _settings(String baseUrl, {bool enabled = true}) =>
    AiSettingsInput(
      enabled: enabled,
      provider: 'custom',
      baseUrl: baseUrl,
      model: 'test-model',
      region: '',
    );

/// The AI features, through the bridge: every call of `KansoAi`,
/// `KansoAskConversation` and `KansoAiRun`, against a model on the loopback
/// address.
void aiSuite(OpenLibrary open) {
  group('KansoAi', () {
    test('settings, providers, actions and the key', () async {
      final ai = (await open()).ai();
      final first = await ai.settings();
      expect(first.enabled, isFalse, reason: 'AI is off until turned on');
      expect(first.hasKey, isFalse);

      final providers = ai.providers();
      expect(providers, hasLength(13));
      expect(providers.first.key, 'local');
      expect(providers.first.description, contains('LM Studio'));
      expect(ai.actions().map((a) => a.label), contains('Improve Writing'));

      final saved = await ai.setSettings(
        input: _settings('http://192.168.1.9:1234/v1'),
      );
      expect(saved.provider, 'custom');
      expect(saved.isRemote, isTrue);
      expect(saved.host, '192.168.1.9:1234');
      expect(await ai.settings(), saved);
      expect(
        await ai.privacyLine(device: 'this tablet'),
        contains('from this tablet to the model at 192.168.1.9:1234'),
      );

      await ai.setKey(provider: 'custom', key: 'sk-test-value');
      expect((await ai.settings()).hasKey, isTrue);
      await ai.setKey(provider: 'custom', key: '');
      expect((await ai.settings()).hasKey, isFalse);
    });

    test('nothing is asked while AI is off', () async {
      final library = await open();
      final ai = library.ai();
      final id = await makeNote(library, 'Plan', 'We ship on friday.');
      final asked = await ai
          .conversation()
          .ask(question: 'when do we ship?')
          .toList();
      expect(
        asked.single,
        isA<AskEvent_Failed>().having(
          (e) => e.message,
          'message',
          'AI is turned off. Turn it on in Settings.',
        ),
      );
      final summary = await ai.summarize(id: id, run: ai.newRun()).toList();
      expect(summary.single, isA<AssistEvent_Failed>());
    });

    test('a streamed answer cites a note by its id', () async {
      final model = await LoopbackModel.start();
      addTearDown(model.close);
      final library = await open();
      final ai = library.ai();
      await ai.setSettings(input: _settings(model.baseUrl));
      final id = await makeNote(
        library,
        'Pricing',
        'We decided the yearly plan costs 40 dollars.',
      );

      final conversation = ai.conversation();
      final events = await conversation
          .ask(question: 'What did we decide about the yearly plan pricing?')
          .toList();
      expect(events.first, isA<AskEvent_Searching>());
      expect(events.any((e) => e is AskEvent_Answer), isTrue);
      final done = events.last as AskEvent_Done;
      expect(done.markdown, contains('[Pricing](kanso://note/$id)'));
      expect(done.notes.single.id, id);
      expect(done.notes.single.title, 'Pricing');
      expect(conversation.turnCount(), 1);
      // The id the link carries opens the note.
      expect((await library.note(id: id)).title, 'Pricing');

      conversation.clear();
      expect(conversation.turnCount(), 0);
      conversation.stop();
    });

    test('a question nothing matches asks no model', () async {
      final model = await LoopbackModel.start();
      addTearDown(model.close);
      final library = await open();
      final ai = library.ai();
      await ai.setSettings(input: _settings(model.baseUrl));
      await makeNote(library, 'Pricing', 'The yearly plan costs 40 dollars.');
      final events = await ai
          .conversation()
          .ask(question: 'quantum chromodynamics lattice')
          .toList();
      expect(
        events.last,
        isA<AskEvent_NothingMatches>().having(
          (e) => e.message,
          'message',
          'Nothing in your notes matches this question.',
        ),
      );
      expect(model.received, isEmpty);
    });

    test(
      'the assistant and the summary stream, and refuse a locked note',
      () async {
        final model = await LoopbackModel.start(answer: 'We ship on Friday.');
        addTearDown(model.close);
        final library = await open();
        final ai = library.ai();
        await ai.setSettings(input: _settings(model.baseUrl));
        final id = await makeNote(library, 'Plan', 'We ship on friday.');
        expect(await ai.noteAllowed(id: id), isTrue);

        final run = ai.newRun();
        final rewrite = await ai
            .assist(
              id: id,
              action: 'improve',
              before: '',
              selection: 'We ship on friday.',
              run: run,
            )
            .toList();
        expect(rewrite.last, isA<AssistEvent_Done>());
        expect((rewrite.last as AssistEvent_Done).text, 'We ship on Friday.');
        run.stop();

        final summary = await ai.summarize(id: id, run: ai.newRun()).toList();
        expect((summary.last as AssistEvent_Done).text, 'We ship on Friday.');

        // Locked, with the lock open: still kept out, and nothing is sent.
        final locks = library.locks();
        await locks.lockNote(id: id, password: lockPassword);
        expect(await locks.isNoteUnlocked(id: id), isTrue);
        expect(await ai.noteAllowed(id: id), isFalse);
        final before = model.received.length;
        for (final events in <List<AssistEvent>>[
          await ai.summarize(id: id, run: ai.newRun()).toList(),
          await ai
              .assist(
                id: id,
                action: 'fix',
                before: '',
                selection: 'text',
                run: ai.newRun(),
              )
              .toList(),
        ]) {
          expect(
            events.single,
            isA<AssistEvent_Failed>().having(
              (e) => e.message,
              'message',
              'This note is kept out of AI.',
            ),
          );
        }
        expect(model.received, hasLength(before));
      },
    );

    test('the test button asks the server', () async {
      final model = await LoopbackModel.start(answer: 'OK');
      addTearDown(model.close);
      final ai = (await open()).ai();
      await ai.setSettings(input: _settings(model.baseUrl));
      expect(await ai.test(), 'OK');
    });
  });
}

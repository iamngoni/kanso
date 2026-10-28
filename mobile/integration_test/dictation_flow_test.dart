// Kanso — dictation_flow_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/models/repos/abs/library_repository.dart';
import 'package:kanso/dictation/models/data/dictation_routes.dart';
import 'package:kanso/dictation/models/repos/abs/dictation_repository.dart';
import 'package:kanso/dictation/models/repos/abs/dictation_text_rules.dart';
import 'package:kanso/dictation/models/repos/impl/platform_dictation_repository.dart';
import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';
import 'package:kanso/dictation/utils/dictation_editor_extras.dart';
import 'package:kanso/dictation/views/pages/voice_note_page.dart';
import 'package:kanso/dictation/views/widgets/dictation_scope.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';
import 'package:kanso/injection.dart';
import 'package:kanso_dictation/kanso_dictation.dart' as plugin;

import '../test/dictation/helpers/fake_dictation_host.dart';
import '../test/dictation/helpers/fake_text_target.dart';
import 'helpers/flow_driver.dart';

/// Dictation on a simulator or emulator, with the speech recognizer
/// replaced by a fake native side: speech cannot be driven there, so the
/// platform's events are sent by the test, through the real plugin, the real
/// repository and BLoC, and the real library.
///
/// Run it with
/// `flutter test integration_test/dictation_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  late FakeHost host;
  late plugin.KansoDictation dictation;

  /// The app's own repository, over a fake native side.
  Future<void> installFakeRecognizer(WidgetTester tester) async {
    await startApp(tester);
    host = FakeHost();
    dictation = plugin.KansoDictation(api: host);
    getIt
      ..allowReassignment = true
      ..registerSingleton<DictationRepository>(
        PlatformDictationRepository(dictation),
      );
  }

  Future<void> say(WidgetTester tester, plugin.DictationEvent event) async {
    dictation.receiver.onEvent(event);
    await tester.pump(const Duration(milliseconds: 50));
    await tester.pump(const Duration(milliseconds: 50));
  }

  /// Dictation into a note, a phrase at a time, through the editor's key.
  Future<void> dictateIntoNote(WidgetTester tester) async {
    final bloc = getIt<DictationBloc>();
    final extras = DictationEditorExtras(
      bloc: bloc,
      rules: getIt<DictationTextRules>(),
    );
    addTearDown(() async {
      await extras.dispose();
      await bloc.close();
    });
    final note = FakeTextTarget('Shopping:');

    extras.toggle(note);
    await tester.pump(const Duration(milliseconds: 100));
    await say(
      tester,
      plugin.DictationEvent(kind: plugin.DictationEventKind.listening),
    );
    await say(
      tester,
      plugin.DictationEvent(
        kind: plugin.DictationEventKind.partial,
        text: 'milk',
      ),
    );
    expect(note.text, 'Shopping:');
    expect(extras.preview.value, ' milk');
    await say(
      tester,
      plugin.DictationEvent(
        kind: plugin.DictationEventKind.finalResult,
        text: 'Milk.',
      ),
    );
    await say(
      tester,
      plugin.DictationEvent(
        kind: plugin.DictationEventKind.finalResult,
        text: 'Eggs.',
      ),
    );
    expect(note.text, 'Shopping: Milk. Eggs.');
    expect(note.undoSteps, 2);
    extras.toggle(note);
    await say(
      tester,
      plugin.DictationEvent(kind: plugin.DictationEventKind.ended),
    );
    expect(extras.accessoryKeys(note).single.active!.value, isFalse);
  }

  // The app starts once per file: both flows share it.
  testWidgets(
    'a voice note is dictated and saved, and a note is dictated into',
    (
      tester,
    ) async {
      await installFakeRecognizer(tester);
      final router = GoRouter(
        initialLocation: DictationRoutes.voiceNote,
        routes: <RouteBase>[
          GoRoute(
            path: DictationRoutes.voiceNote,
            builder: (context, state) => const VoiceNotePage(),
          ),
          GoRoute(
            path: '/note/:id',
            builder: (context, state) =>
                Scaffold(body: Text('Opened ${state.pathParameters['id']}')),
          ),
          GoRoute(path: '/notes', builder: (_, _) => const SizedBox.shrink()),
        ],
      );
      addTearDown(router.dispose);
      await tester.pumpWidget(
        MaterialApp.router(
          theme: KansoTheme.light,
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          routerConfig: router,
          builder: (context, child) => DictationScope(child: child!),
        ),
      );
      await pumpUntil(tester, find.text('Tap the mic and start talking.'));

      await tester.tap(find.bySemanticsLabel('Start listening'));
      await pumpUntil(
        tester,
        find.bySemanticsLabel('Stop listening'),
        timeout: const Duration(seconds: 5),
      ).catchError((Object _) {});
      expect(host.calls, contains('start null'));
      await say(
        tester,
        plugin.DictationEvent(kind: plugin.DictationEventKind.listening),
      );
      await say(
        tester,
        plugin.DictationEvent(
          kind: plugin.DictationEventKind.partial,
          text: 'Garden beds',
        ),
      );
      expect(
        find.textContaining('Garden beds', findRichText: true),
        findsOneWidget,
      );
      await say(
        tester,
        plugin.DictationEvent(
          kind: plugin.DictationEventKind.finalResult,
          text: 'Garden beds',
        ),
      );
      await say(
        tester,
        plugin.DictationEvent(
          kind: plugin.DictationEventKind.finalResult,
          text: 'Three raised beds along the fence.',
        ),
      );

      await tester.tap(find.text('Done'));
      await tester.pump();
      expect(host.calls, contains('stop'));
      await say(
        tester,
        plugin.DictationEvent(kind: plugin.DictationEventKind.ended),
      );
      await pumpUntil(tester, find.textContaining('Opened '));

      // The note is in the real library: what was said is one short line, so
      // it is the title, as the desktop's quick capture makes it.
      final library = getIt<LibraryRepository>();
      final found = await library.search('raised beds');
      final hits = found.getOrElse(() => <Hit>[]);
      expect(hits, hasLength(1));
      expect(
        hits.single.row.title,
        'Garden beds Three raised beds along the fence.',
      );
      await dictateIntoNote(tester);
    },
  );
}

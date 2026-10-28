// Kanso — voice_note_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:go_router/go_router.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/views/widgets/kanso_button.dart';
import 'package:kanso/dictation/models/data/dictation_capability.dart';
import 'package:kanso/dictation/models/data/dictation_permission.dart';
import 'package:kanso/dictation/models/data/dictation_signal.dart';
import 'package:kanso/dictation/models/data/dictation_unavailable_reason.dart';
import 'package:kanso/dictation/state/dictation/dictation_bloc.dart';
import 'package:kanso/dictation/state/voice_note/voice_note_bloc.dart';
import 'package:kanso/dictation/views/widgets/voice_note_pane.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/pump_routed.dart';
import '../../helpers/fake_dictation_text_rules.dart';
import '../../helpers/mock_dictation_repository.dart';

class FakeNewNote extends Fake implements NewNote {}

void main() {
  late SpokenRepository spoken;
  late MockLibraryRepository library;

  setUpAll(() => registerFallbackValue(FakeNewNote()));

  setUp(() {
    spoken = SpokenRepository();
    library = MockLibraryRepository();
    when(() => library.create(any())).thenAnswer(
      (_) async => const Right<ApplicationError, Saved>(
        Saved(id: 'note:new', rev: 'r1'),
      ),
    );
  });

  tearDown(() => spoken.close());

  Future<GoRouterHandle> pump(WidgetTester tester) async {
    final dictation = DictationBloc(spoken.repository, PassThroughRules());
    final voice = VoiceNoteBloc(library);
    closeBlocLater(dictation);
    closeBlocLater(voice);
    final router = await pumpRouted(
      tester,
      MultiBlocProvider(
        providers: <BlocProvider<dynamic>>[
          BlocProvider<DictationBloc>.value(value: dictation),
          BlocProvider<VoiceNoteBloc>.value(value: voice),
        ],
        child: const VoiceNotePane(),
      ),
    );
    return GoRouterHandle(router, dictation);
  }

  Future<void> hear(WidgetTester tester, DictationSignal signal) async {
    spoken.signals.add(signal);
    await tester.pump();
    await tester.pump();
  }

  testWidgets('says how to begin, and Done waits for words', (tester) async {
    await pump(tester);
    expect(find.text('Voice note'), findsOneWidget);
    expect(find.text('Tap the mic and start talking.'), findsOneWidget);
    expect(find.text('What you say will appear here.'), findsOneWidget);
    final done = find.widgetWithText(KansoButton, 'Done');
    expect(tester.widget<KansoButton>(done).onPressed, isNull);
  });

  testWidgets('the mic starts listening and shows what is heard', (
    tester,
  ) async {
    await pump(tester);
    await tester.tap(find.bySemanticsLabel('Start listening'));
    await tester.pump();
    await tester.pump();
    await hear(tester, const DictationSignal.listening());
    expect(find.bySemanticsLabel('Stop listening'), findsOneWidget);
    await hear(tester, const DictationSignal.partial('buy milk'));
    expect(find.textContaining('buy milk', findRichText: true), findsOneWidget);
    verify(
      () => spoken.repository.start(localeTag: any(named: 'localeTag')),
    ).called(1);
  });

  testWidgets('Done stops, saves the words as a note and opens it', (
    tester,
  ) async {
    final handle = await pump(tester);
    await tester.tap(find.bySemanticsLabel('Start listening'));
    await tester.pump();
    await tester.pump();
    await hear(tester, const DictationSignal.listening());
    await hear(tester, const DictationSignal.partial('Groceries'));
    await tester.tap(find.text('Done'));
    await tester.pump();
    verify(spoken.repository.stop).called(1);
    await hear(tester, const DictationSignal.finalText('Groceries'));
    await hear(tester, const DictationSignal.ended());
    await tester.pumpAndSettle();
    final note =
        verify(() => library.create(captureAny())).captured.single as NewNote;
    expect(note.title, 'Groceries');
    expect(note.body, isEmpty);
    expect(whereIs(handle.router), '/note/note%3Anew');
  });

  testWidgets('Cancel drops the dictation', (tester) async {
    await pump(tester);
    await tester.tap(find.text('Cancel'));
    await tester.pumpAndSettle();
    verifyNever(() => library.create(any()));
  });

  testWidgets('a phone that cannot listen says so and offers a way out', (
    tester,
  ) async {
    spoken = SpokenRepository(
      capability: const DictationCapability(
        available: false,
        onDevice: false,
        permission: DictationPermission.granted,
        reason: DictationUnavailableReason.notOnDevice,
      ),
    );
    final handle = await pump(tester);
    handle.dictation.add(const DictationEvent.checked());
    await tester.pumpAndSettle();
    expect(find.text("Dictation isn't available"), findsOneWidget);
    expect(find.textContaining('never sends your voice'), findsOneWidget);
  });

  testWidgets('a refused microphone offers Settings', (tester) async {
    spoken = SpokenRepository(
      capability: const DictationCapability(
        available: true,
        onDevice: true,
        permission: DictationPermission.denied,
      ),
    );
    final handle = await pump(tester);
    handle.dictation.add(const DictationEvent.checked());
    await tester.pumpAndSettle();
    expect(find.text("Kanso can't hear you"), findsOneWidget);
    await tester.tap(find.text('Open Settings'));
    await tester.pump();
    verify(spoken.repository.openSettings).called(1);
  });

  testWidgets('a note that could not be saved keeps the words', (tester) async {
    when(() => library.create(any())).thenAnswer(
      (_) async => const Left<ApplicationError, Saved>(
        ApplicationError.storage(message: 'Disk full.'),
      ),
    );
    await pump(tester);
    await tester.tap(find.bySemanticsLabel('Start listening'));
    await tester.pump();
    await tester.pump();
    await hear(tester, const DictationSignal.listening());
    await hear(tester, const DictationSignal.finalText('Keep this'));
    await tester.tap(find.text('Done'));
    await tester.pump();
    await hear(tester, const DictationSignal.ended());
    await tester.pumpAndSettle();
    expect(find.text("The note wasn't saved"), findsOneWidget);
    expect(find.textContaining('Disk full'), findsNothing);
    await tester.tap(find.text('Try again'));
    await tester.pumpAndSettle();
    verify(() => library.create(any())).called(2);
  });
}

class GoRouterHandle {
  GoRouterHandle(this.router, this.dictation);

  final GoRouter router;
  final DictationBloc dictation;
}

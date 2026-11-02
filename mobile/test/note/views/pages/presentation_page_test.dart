// Kanso — presentation_page_test.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc_test/bloc_test.dart';
import 'package:dartz/dartz.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart' hide LockState;
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart' hide Table, TableRow;
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/lock/models/data/note_menu_action.dart';
import 'package:kanso/lock/state/lock_status/lock_status_bloc.dart';
import 'package:kanso/lock/utils/note_menu_actions.dart';
import 'package:kanso/note/state/preview/preview_bloc.dart';
import 'package:kanso/note/views/pages/presentation_page.dart';
import 'package:kanso/note/views/widgets/presentation_deck.dart';
import 'package:mocktail/mocktail.dart';

import '../../../helpers/kanso_test_app.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/mock_repositories.dart';
import '../../../helpers/seeded_appearance_bloc.dart';
import '../../../helpers/theme_tokens.dart';

class _MockLockStatusBloc extends MockBloc<LockStatusEvent, LockStatusState>
    implements LockStatusBloc {}

const PaletteDto _palette = PaletteDto(
  dark: false,
  background: 0xFAFAFA,
  text: 0x111111,
  muted: 0x777777,
  accent: 0x1E66F5,
  border: 0xCCCCCC,
  surface: 0xEEEEEE,
  heading: 0x8839EF,
  quote: 0x40A02B,
  taskDone: 0x999999,
  inlineCodeBackground: 0xE0E0E0,
);

PreviewBlock _words(String text) => PreviewBlock.paragraph(
  inlines: <PreviewInline>[
    PreviewInline.text(
      text: text,
      bold: false,
      italic: false,
      strike: false,
      code: false,
    ),
  ],
);

PreviewBlock _heading(int level, String text) => PreviewBlock.heading(
  level: level,
  id: text,
  inlines: <PreviewInline>[
    PreviewInline.text(
      text: text,
      bold: false,
      italic: false,
      strike: false,
      code: false,
    ),
  ],
);

const PreviewBlock _rule = PreviewBlock.rule();

Widget _deck(List<List<PreviewBlock>> slides, {VoidCallback? onClose}) =>
    KansoTestApp(
      home: PresentationDeck(
        slides: slides,
        palette: _palette,
        onClose: onClose ?? () {},
      ),
    );

void main() {
  group('slidesOf', () {
    test('a rule starts a slide and is on neither', () {
      final one = _words('One');
      final two = _words('Two');
      expect(slidesOf(<PreviewBlock>[one, _rule, two]), <List<PreviewBlock>>[
        <PreviewBlock>[one],
        <PreviewBlock>[two],
      ]);
    });

    test('with no rule, the main headings start slides', () {
      final intro = _words('Intro');
      final first = _heading(1, 'One');
      final second = _heading(2, 'Two');
      final kept = _heading(3, 'Kept');
      expect(
        slidesOf(<PreviewBlock>[intro, first, second, kept]),
        <List<PreviewBlock>>[
          <PreviewBlock>[intro],
          <PreviewBlock>[first],
          <PreviewBlock>[second, kept],
        ],
      );
    });

    test('rules win over headings', () {
      final first = _heading(1, 'One');
      final second = _heading(1, 'Two');
      expect(
        slidesOf(<PreviewBlock>[first, second, _rule]),
        <List<PreviewBlock>>[
          <PreviewBlock>[first, second],
        ],
      );
    });

    test('empty slides are left out', () {
      expect(slidesOf(const <PreviewBlock>[_rule, _rule]), isEmpty);
      expect(slidesOf(const <PreviewBlock>[]), isEmpty);
    });
  });

  test("text is the note's size on a phone and larger on a tablet", () {
    expect(slideTextScale(const Size(390, 844)), 1);
    expect(slideTextScale(const Size(844, 390)), 1);
    expect(slideTextScale(const Size(1180, 820)), greaterThan(1.5));
    expect(slideTextScale(const Size(6000, 6000)), 2.5);
  });

  test('a note that can be read can be presented, a closed one cannot', () {
    expect(
      noteMenuActions(lock: const LockState.plain(), unlocked: false),
      contains(NoteMenuAction.present),
    );
    expect(
      noteMenuActions(lock: const LockState.closed(), unlocked: false),
      isNot(contains(NoteMenuAction.present)),
    );
  });

  group('PresentationDeck', () {
    final slides = <List<PreviewBlock>>[
      <PreviewBlock>[_words('First')],
      <PreviewBlock>[_words('Second')],
      <PreviewBlock>[_words('Third')],
    ];

    testWidgets('a tap on the right goes on and on the left goes back', (
      tester,
    ) async {
      await tester.pumpWidget(_deck(slides));
      await tester.pumpAndSettle();
      expect(find.text('1 / 3'), findsOneWidget);
      final size = tester.getSize(find.byType(PresentationDeck));

      await tester.tapAt(Offset(size.width * 0.9, size.height / 2));
      await tester.pumpAndSettle();
      expect(find.text('2 / 3'), findsOneWidget);
      expect(find.text('Second', findRichText: true), findsOneWidget);

      await tester.tapAt(Offset(size.width * 0.1, size.height / 2));
      await tester.pumpAndSettle();
      expect(find.text('1 / 3'), findsOneWidget);
    });

    testWidgets('a tap stops at either end', (tester) async {
      await tester.pumpWidget(_deck(slides));
      await tester.pumpAndSettle();
      final size = tester.getSize(find.byType(PresentationDeck));
      await tester.tapAt(Offset(size.width * 0.1, size.height / 2));
      await tester.pumpAndSettle();
      expect(find.text('1 / 3'), findsOneWidget);
    });

    testWidgets('a keyboard moves and Esc asks to close', (tester) async {
      var closed = 0;
      await tester.pumpWidget(_deck(slides, onClose: () => closed++));
      await tester.pumpAndSettle();

      await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
      await tester.pumpAndSettle();
      expect(find.text('2 / 3'), findsOneWidget);
      await tester.sendKeyEvent(LogicalKeyboardKey.end);
      await tester.pumpAndSettle();
      expect(find.text('3 / 3'), findsOneWidget);
      await tester.sendKeyEvent(LogicalKeyboardKey.home);
      await tester.pumpAndSettle();
      expect(find.text('1 / 3'), findsOneWidget);

      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      expect(closed, 1);
    });

    testWidgets('fewer slides than the one on show leaves a slide on show', (
      tester,
    ) async {
      await tester.pumpWidget(_deck(slides));
      await tester.pumpAndSettle();
      await tester.sendKeyEvent(LogicalKeyboardKey.end);
      await tester.pumpAndSettle();
      await tester.pumpWidget(_deck(slides.sublist(0, 1)));
      await tester.pumpAndSettle();
      expect(find.text('1 / 1'), findsOneWidget);
    });

    testWidgets('a note with no slides says so', (tester) async {
      await tester.pumpWidget(_deck(const <List<PreviewBlock>>[]));
      await tester.pumpAndSettle();
      expect(find.text('Nothing to present yet'), findsOneWidget);
    });
  });

  group('PresentationPage', () {
    late MockLibraryRepository library;
    late MockRendererRepository renderer;
    late MockThemesRepository themes;
    late _MockLockStatusBloc status;
    late StreamController<LockStatusState> lockChanges;
    late Either<ApplicationError, List<PreviewBlock>> drawn;

    setUpAll(() {
      registerFallbackValue(_palette);
      registerFallbackValue(
        const PreviewOptions(size: 16, utcOffsetSecs: 0, code: []),
      );
    });

    setUp(() {
      library = MockLibraryRepository();
      renderer = MockRendererRepository();
      themes = MockThemesRepository();
      status = _MockLockStatusBloc();
      lockChanges = StreamController<LockStatusState>.broadcast();
      addTearDown(lockChanges.close);
      whenListen(
        status,
        lockChanges.stream,
        initialState: const LockStatusState(unlocked: true),
      );
      drawn = Right(<PreviewBlock>[
        _words('First'),
        _rule,
        _words('Second'),
      ]);
      when(
        () => library.changes(),
      ).thenAnswer((_) => const Stream<List<String>>.empty());
      when(() => library.note(any())).thenAnswer(
        (_) async => Right<ApplicationError, NoteView>(mockOpenNote),
      );
      when(() => themes.fit(any(), any(), any())).thenAnswer(
        (_) async => Right<ApplicationError, ThemeTokens>(
          tokensFor(fakePalette('paper')),
        ),
      );
      when(
        () => renderer.previewBlocks(any(), any(), any()),
      ).thenAnswer((_) async => drawn);
      getIt.registerFactory<PreviewBloc>(
        () => PreviewBloc(
          library,
          renderer,
          themes,
          MockWebLinkRepository(),
          MockUndoNoticeBloc(),
          MockHapticsRepository(),
        ),
      );
      addTearDown(() => getIt.unregister<PreviewBloc>());
    });

    Future<void> open(WidgetTester tester) async {
      final appearance = SeededAppearanceBloc(const AppearanceState());
      addTearDown(appearance.close);
      await tester.pumpWidget(
        KansoTestApp(
          home: MultiBlocProvider(
            providers: [
              BlocProvider<AppearanceBloc>.value(value: appearance),
              BlocProvider<LockStatusBloc>.value(value: status),
            ],
            child: Builder(
              builder: (context) => GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTap: () => PresentationPage.show(context, noteId: 'note:1'),
                child: const SizedBox.expand(),
              ),
            ),
          ),
        ),
      );
      await tester.tapAt(const Offset(40, 40));
      await tester.pumpAndSettle();
    }

    testWidgets("shows the note's slides from its preview blocks", (
      tester,
    ) async {
      await open(tester);
      expect(find.text('1 / 2'), findsOneWidget);
      expect(find.text('First', findRichText: true), findsOneWidget);
    });

    testWidgets('closes when a redraw after locking fails', (tester) async {
      await open(tester);
      drawn = const Left(LockedError(message: 'Locked'));
      lockChanges.add(const LockStatusState());
      await tester.pumpAndSettle();
      expect(find.byType(PresentationDeck), findsNothing);
    });

    testWidgets('a tap on a slide writes nothing to the note', (tester) async {
      await open(tester);
      final size = tester.getSize(find.byType(PresentationDeck));
      await tester.tapAt(Offset(size.width * 0.9, size.height / 2));
      await tester.pumpAndSettle();
      expect(find.text('2 / 2'), findsOneWidget);
      verifyNever(
        () => library.toggleTaskAt(
          noteId: any(named: 'noteId'),
          index: any(named: 'index'),
          baseRev: any(named: 'baseRev'),
        ),
      );
    });
  });
}

// Kanso — embargo_test.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/embargo_types.dart';
import 'package:kanso/core/bridge/api/publish.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/embargo/models/data/embargo_gate.dart';
import 'package:kanso/embargo/models/repos/abs/embargo_repository.dart';
import 'package:kanso/embargo/views/widgets/embargo_sheet.dart';
import 'package:kanso/embargo/views/widgets/embargo_strip.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/note/views/widgets/note_html_export.dart';
import 'package:kanso/note/views/widgets/note_pdf_export.dart';
import 'package:kanso/publish/models/repos/abs/publish_repository.dart';
import 'package:kanso/publish/state/publish/publish_bloc.dart';
import 'package:kanso/publish/views/widgets/publish_sheet.dart';
import 'package:kanso/sharing/state/share_note/share_note_bloc.dart';
import 'package:kanso/sharing/views/widgets/share_note_view.dart';

import '../helpers/close_bloc_later.dart';
import '../helpers/fixture_shared_notes_repository.dart';
import '../helpers/fixture_system_share_repository.dart';
import '../helpers/golden.dart';
import '../helpers/kanso_test_app.dart';
import '../helpers/kanso_viewport.dart';
import '../helpers/pump_routed.dart';
import '../helpers/seeded_appearance_bloc.dart';
import '../helpers/sharing_fixtures.dart';

const EmbargoView _held = EmbargoView(
  lifts: 'Wed 18 Nov, 09:00 New York',
  liftsIn: 'in 5 days',
  liftsAtMs: 1795010400000,
  zone: 'America/New_York',
);

/// Says [view] for every note, and remembers which were asked about.
class _Embargoes implements EmbargoRepository {
  _Embargoes([this.view = _held]);

  EmbargoView? view;
  final List<String> asked = <String>[];

  @override
  Future<Either<ApplicationError, EmbargoView?>> embargo(String noteId) async {
    asked.add(noteId);
    return right(view);
  }
}

/// Publishes at once, and counts what it publishes.
class _Publisher implements PublishRepository {
  int published = 0;

  @override
  Future<Either<ApplicationError, List<PublishExpiryOption>>>
  expiryOptions() async => right(<PublishExpiryOption>[
    const PublishExpiryOption(expiry: PublishExpiry.never, label: 'Never'),
  ]);

  @override
  Future<Either<ApplicationError, PublishNoteInfo>> note(String id) async =>
      right(
        PublishNoteInfo(
          id: id,
          title: 'Pixel 11 review',
          body: 'The Pixel 11 is fast.',
          rev: '2-a',
          needsUpdate: false,
        ),
      );

  @override
  Future<Either<ApplicationError, PublishNotebookInfo>> notebook(
    String id,
  ) async => right(
    PublishNotebookInfo(
      id: id,
      name: 'Studio',
      needsUpdate: false,
      contents: '1 note.',
      locked: '',
      noteCount: 1,
      fileCount: 0,
    ),
  );

  @override
  Future<Either<ApplicationError, PublishLink>> publishNote({
    required String id,
    required PublishExpiry expiry,
    required PaletteDto palette,
    required PreviewOptions options,
  }) async {
    published++;
    return right(
      const PublishLink(
        url: 'https://kanso.example/p/pixel#key',
        expiry: PublishExpiry.never,
        publishedAtMs: 1794582000000,
      ),
    );
  }

  @override
  Future<Either<ApplicationError, PublishLink>> publishNotebook({
    required String id,
    required PublishExpiry expiry,
    required PaletteDto palette,
    required PreviewOptions options,
  }) => publishNote(id: id, expiry: expiry, palette: palette, options: options);

  @override
  Future<Either<ApplicationError, Unit>> unpublishNote(String id) async =>
      right(unit);

  @override
  Future<Either<ApplicationError, Unit>> unpublishNotebook(String id) async =>
      right(unit);
}

const PaletteDto _palette = PaletteDto(
  dark: false,
  background: 0xfcfaf4,
  text: 0x26241f,
  muted: 0x857c6e,
  accent: 0x4d6684,
  border: 0xddd6c9,
  surface: 0xf4f1ea,
  quote: 0x655f55,
  taskDone: 0x655f55,
  inlineCodeBackground: 0xf4f1ea,
);

const PreviewOptions _options = PreviewOptions(
  size: 16,
  utcOffsetSecs: 0,
  code: <SyntaxStyleRow>[],
);

/// Puts [embargoes] where the gate finds it, for the test.
void _use(_Embargoes embargoes) {
  getIt.registerSingleton<EmbargoRepository>(embargoes);
  addTearDown(() => getIt.unregister<EmbargoRepository>());
}

Future<void> _settle(WidgetTester tester) async {
  for (var i = 0; i < 10; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

/// A page with one button that runs [run] and keeps what it answered.
Widget _button(Future<void> Function(BuildContext context) run) => Scaffold(
  body: Center(
    child: Builder(
      builder: (context) => TextButton(
        onPressed: () => run(context),
        child: const Text('Go'),
      ),
    ),
  ),
);

void main() {
  group('the banner', () {
    testWidgets('a note under embargo says until when, and how long', (
      tester,
    ) async {
      await pumpRouted(
        tester,
        Scaffold(
          body: EmbargoStrip(noteId: 'note:pixel', repository: _Embargoes()),
        ),
      );
      await tester.pumpAndSettle();
      expect(
        find.text('Embargoed until Wed 18 Nov, 09:00 New York'),
        findsOneWidget,
      );
      expect(find.text('Lifts in 5 days'), findsOneWidget);
    });

    testWidgets('a note without one, or whose embargo lifted, shows nothing', (
      tester,
    ) async {
      final embargoes = _Embargoes(null);
      await pumpRouted(
        tester,
        Scaffold(
          body: EmbargoStrip(
            noteId: 'note:pixel',
            revision: '1-a',
            repository: embargoes,
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.byKey(const ValueKey<String>('embargo-strip')), findsNothing);

      // A new revision that sets one shows it.
      embargoes.view = _held;
      await pumpRouted(
        tester,
        Scaffold(
          body: EmbargoStrip(
            noteId: 'note:pixel',
            revision: '2-b',
            repository: embargoes,
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(
        find.byKey(const ValueKey<String>('embargo-strip')),
        findsOneWidget,
      );
      expect(embargoes.asked, <String>['note:pixel', 'note:pixel']);
    });

    testWidgets('banner golden', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        Scaffold(
          body: Column(
            children: <Widget>[
              EmbargoStrip(noteId: 'note:pixel', repository: _Embargoes()),
            ],
          ),
        ),
        name: 'embargo_banner_phone',
        brightness: Brightness.light,
        viewport: KansoViewport.phone,
        palette: KansoPalette.kansoLight,
      );
    });
  });

  group('asking first', () {
    testWidgets('no embargo goes ahead without asking', (tester) async {
      bool? answer;
      await pumpRouted(
        tester,
        _button((context) async {
          answer = await confirmEmbargo(
            context,
            noteId: 'note:pixel',
            gate: EmbargoGate.publish,
            repository: _Embargoes(null),
          );
        }),
      );
      await tester.tap(find.text('Go'));
      await _settle(tester);
      expect(answer, isTrue);
      expect(find.text('This note is under embargo'), findsNothing);
    });

    for (final (gate, question, confirm) in <(EmbargoGate, String, String)>[
      (
        EmbargoGate.publish,
        'Embargo lifts Wed 18 Nov, 09:00 New York (in 5 days). Publish anyway?',
        'Publish anyway',
      ),
      (
        EmbargoGate.shareLink,
        'Embargo lifts Wed 18 Nov, 09:00 New York (in 5 days). '
            'Make a share link anyway?',
        'Make link anyway',
      ),
      (
        EmbargoGate.export,
        'Embargo lifts Wed 18 Nov, 09:00 New York (in 5 days). '
            'Export anyway?',
        'Export anyway',
      ),
    ]) {
      testWidgets('${gate.name} asks, and goes ahead only when told', (
        tester,
      ) async {
        bool? answer;
        await pumpRouted(
          tester,
          _button((context) async {
            answer = await confirmEmbargo(
              context,
              noteId: 'note:pixel',
              gate: gate,
              repository: _Embargoes(),
            );
          }),
        );
        await tester.tap(find.text('Go'));
        await _settle(tester);
        expect(find.text('This note is under embargo'), findsOneWidget);
        expect(find.text(question), findsOneWidget);
        await tester.tap(find.text('Cancel'));
        await _settle(tester);
        expect(answer, isFalse);

        await tester.tap(find.text('Go'));
        await _settle(tester);
        await tester.tap(find.text(confirm));
        await _settle(tester);
        expect(answer, isTrue);
      });
    }

    testWidgets('sheet golden', (tester) async {
      await expectGoldenOnBothPlatforms(
        tester,
        const Scaffold(
          body: Align(
            alignment: Alignment.bottomCenter,
            child: EmbargoSheet(embargo: _held, gate: EmbargoGate.publish),
          ),
        ),
        name: 'embargo_sheet_phone',
        brightness: Brightness.light,
        viewport: KansoViewport.phone,
        palette: KansoPalette.kansoLight,
      );
    });
  });

  group('publish', () {
    Future<_Publisher> pumpSheet(WidgetTester tester) async {
      final publisher = _Publisher();
      await tester.pumpWidget(
        KansoTestApp(
          home: Scaffold(
            body: BlocProvider<PublishBloc>(
              create: (_) => PublishBloc(publisher)
                ..add(
                  const PublishEvent.openNote(
                    id: 'note:pixel',
                    palette: _palette,
                    options: _options,
                  ),
                ),
              child: const PublishSheet(),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      return publisher;
    }

    testWidgets('a note under embargo asks before it is published', (
      tester,
    ) async {
      final embargoes = _Embargoes();
      _use(embargoes);
      final publisher = await pumpSheet(tester);
      await tester.tap(find.text('Publish'));
      await _settle(tester);
      expect(embargoes.asked, <String>['note:pixel']);
      expect(find.text('This note is under embargo'), findsOneWidget);
      await tester.tap(find.text('Cancel'));
      await _settle(tester);
      expect(publisher.published, 0, reason: 'Cancel publishes nothing');

      await tester.tap(find.text('Publish'));
      await _settle(tester);
      await tester.tap(find.text('Publish anyway'));
      await _settle(tester);
      expect(publisher.published, 1);
    });

    testWidgets('a lifted embargo publishes at once', (tester) async {
      _use(_Embargoes(null));
      final publisher = await pumpSheet(tester);
      await tester.tap(find.text('Publish'));
      await _settle(tester);
      expect(find.text('This note is under embargo'), findsNothing);
      expect(publisher.published, 1);
    });
  });

  group('share link', () {
    testWidgets('making a link to a note under embargo asks first', (
      tester,
    ) async {
      final embargoes = _Embargoes();
      _use(embargoes);
      final shared = FixtureSharedNotesRepository();
      shared.shares['n1'] = shareInfo(
        'n1',
        members: <MemberRow>[
          memberRow('ngoni', role: ShareRole.owner, me: true, verified: true),
        ],
      );
      final bloc = ShareNoteBloc(shared, FixtureSystemShareRepository())
        ..add(const ShareNoteEvent.started('n1'));
      closeBlocLater(bloc);
      await pumpRouted(
        tester,
        BlocProvider<ShareNoteBloc>.value(
          value: bloc,
          child: const SizedBox(height: 820, child: ShareNoteView()),
        ),
      );
      await _settle(tester);
      final make = find.text('Make link').first;
      await tester.ensureVisible(make);
      await tester.pump();
      await tester.tap(make);
      await _settle(tester);
      expect(embargoes.asked, <String>['n1']);
      expect(find.text('This note is under embargo'), findsOneWidget);
      await tester.tap(find.text('Cancel'));
      await _settle(tester);
      expect(
        shared.calls.where((call) => call.startsWith('createLink')),
        isEmpty,
      );

      await tester.tap(find.text('Make link').first);
      await _settle(tester);
      await tester.tap(find.text('Make link anyway'));
      await _settle(tester);
      expect(
        shared.calls.where((call) => call.startsWith('createLink')),
        hasLength(1),
      );
    });
  });

  group('export', () {
    for (final (kind, export)
        in <(String, Future<void> Function(BuildContext))>[
          (
            'web page',
            (context) => exportNoteHtml(context, noteId: 'note:pixel'),
          ),
          ('PDF', (context) => exportNotePdf(context, noteId: 'note:pixel')),
        ]) {
      testWidgets('a $kind of a note under embargo asks first', (
        tester,
      ) async {
        final embargoes = _Embargoes();
        _use(embargoes);
        final appearance = SeededAppearanceBloc(const AppearanceState());
        addTearDown(appearance.close);
        await pumpRouted(
          tester,
          BlocProvider<AppearanceBloc>.value(
            value: appearance,
            child: _button(export),
          ),
        );
        await tester.tap(find.text('Go'));
        await _settle(tester);
        expect(find.text('This note is under embargo'), findsOneWidget);
        expect(
          find.text(
            'Embargo lifts Wed 18 Nov, 09:00 New York (in 5 days). '
            'Export anyway?',
          ),
          findsOneWidget,
        );
        await tester.tap(find.text('Cancel'));
        await _settle(tester);
        // Kept back: nothing was written, so nothing failed either.
        expect(find.text('The note could not be exported.'), findsNothing);
        expect(find.text('This note is under embargo'), findsNothing);
        expect(embargoes.asked, <String>['note:pixel']);
      });
    }
  });
}

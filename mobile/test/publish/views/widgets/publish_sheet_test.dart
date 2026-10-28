// Kanso — publish_sheet_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/publish.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_palette.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';
import 'package:kanso/publish/models/repos/abs/publish_repository.dart';
import 'package:kanso/publish/state/publish/publish_bloc.dart';
import 'package:kanso/publish/views/widgets/publish_sheet.dart';

import '../../../helpers/golden.dart';
import '../../../helpers/kanso_viewport.dart';

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

const PublishLink _link = PublishLink(
  url: 'https://kanso.example/p/research#secret-key',
  expiry: PublishExpiry.never,
  publishedAtMs: 1_798_000_000_000,
);

const PublishLink _expiredLink = PublishLink(
  url: 'https://kanso.example/p/research#old-key',
  expiresAtMs: 1,
  expiry: PublishExpiry.day,
  publishedAtMs: 0,
);

class _PublishRepository implements PublishRepository {
  _PublishRepository({
    this.published = false,
    this.expired = false,
    this.refusal,
    this.failure,
    this.pending = false,
  });

  final bool published;
  final bool expired;
  final String? refusal;
  final ApplicationError? failure;
  final bool pending;
  final Completer<Either<ApplicationError, PublishLink>> _waiting =
      Completer<Either<ApplicationError, PublishLink>>();

  @override
  Future<Either<ApplicationError, List<PublishExpiryOption>>>
  expiryOptions() async => right(<PublishExpiryOption>[
    const PublishExpiryOption(expiry: PublishExpiry.never, label: 'Never'),
    const PublishExpiryOption(expiry: PublishExpiry.day, label: '1 day'),
    const PublishExpiryOption(expiry: PublishExpiry.week, label: '7 days'),
    const PublishExpiryOption(expiry: PublishExpiry.month, label: '30 days'),
  ]);

  @override
  Future<Either<ApplicationError, PublishNoteInfo>> note(String id) async =>
      right(
        PublishNoteInfo(
          id: id,
          title: 'Field notes',
          body: 'A real note with an outline, a passage, and useful detail.',
          rev: '2-a',
          published: published ? (expired ? _expiredLink : _link) : null,
          needsUpdate: published,
          refusal: refusal,
        ),
      );

  @override
  Future<Either<ApplicationError, PublishNotebookInfo>> notebook(
    String id,
  ) async => right(
    PublishNotebookInfo(
      id: id,
      name: 'Research',
      published: published ? (expired ? _expiredLink : _link) : null,
      needsUpdate: published,
      contents: '2 notes and 1 image.',
      locked: 'Left out because it is locked: Private journal.',
      refusal: refusal,
      noteCount: 2,
      fileCount: 1,
    ),
  );

  @override
  Future<Either<ApplicationError, PublishLink>> publishNote({
    required String id,
    required PublishExpiry expiry,
    required PaletteDto palette,
    required PreviewOptions options,
  }) => _result();

  @override
  Future<Either<ApplicationError, PublishLink>> publishNotebook({
    required String id,
    required PublishExpiry expiry,
    required PaletteDto palette,
    required PreviewOptions options,
  }) => _result();

  Future<Either<ApplicationError, PublishLink>> _result() {
    if (pending) {
      return _waiting.future;
    }
    return Future<Either<ApplicationError, PublishLink>>.value(
      failure == null ? right(_link) : left(failure!),
    );
  }

  @override
  Future<Either<ApplicationError, Unit>> unpublishNote(String id) async =>
      right(unit);

  @override
  Future<Either<ApplicationError, Unit>> unpublishNotebook(String id) async =>
      right(unit);
}

Widget _sheet({
  required _PublishRepository repository,
  required bool notebook,
  bool startPublishing = false,
}) => BlocProvider<PublishBloc>(
  create: (context) {
    final bloc = PublishBloc(repository)
      ..add(
        notebook
            ? const PublishEvent.openNotebook(
                id: 'book:research',
                palette: _palette,
                options: _options,
              )
            : const PublishEvent.openNote(
                id: 'note:field',
                palette: _palette,
                options: _options,
              ),
      );
    if (startPublishing) {
      Future<void>.delayed(const Duration(milliseconds: 20), () {
        if (!bloc.isClosed) {
          bloc.add(const PublishEvent.publish(PublishExpiry.never));
        }
      });
    }
    return bloc;
  },
  child: const Scaffold(body: PublishSheet()),
);

void main() {
  testWidgets('note sheet describes content and exposes expiry choices', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: KansoTheme.light,
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: _sheet(repository: _PublishRepository(), notebook: false),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Publish note'), findsOneWidget);
    expect(find.text('Field notes'), findsOneWidget);
    expect(
      find.text('A real note with an outline, a passage, and useful detail.'),
      findsOneWidget,
    );
    expect(find.text('Never'), findsOneWidget);
    expect(find.text('Publish'), findsOneWidget);
  });

  testWidgets('notebook sheet shows site contents and locked exclusions', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: KansoTheme.light,
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: _sheet(
          repository: _PublishRepository(),
          notebook: true,
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Publish notebook as a site'), findsOneWidget);
    expect(find.text('Research'), findsOneWidget);
    expect(find.text('2 notes and 1 image.'), findsOneWidget);
    expect(
      find.text('Left out because it is locked: Private journal.'),
      findsOneWidget,
    );
    expect(find.text('Publish site'), findsOneWidget);
  });

  testWidgets('locked note displays the engine refusal and blocks publish', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: KansoTheme.light,
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: _sheet(
          repository: _PublishRepository(
            refusal: "Locked notes can't be published. Remove the lock first.",
          ),
          notebook: false,
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text("Locked notes can't be published. Remove the lock first."),
      findsOneWidget,
    );
    final button = tester.widget<FilledButton>(
      find.ancestor(
        of: find.text('Publish'),
        matching: find.byType(FilledButton),
      ),
    );
    expect(button.onPressed, isNull);
  });

  testWidgets('expired link shows why it needs an update', (tester) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: KansoTheme.light,
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: _sheet(
          repository: _PublishRepository(published: true, expired: true),
          notebook: false,
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(
      find.text('This link has expired. Update to create a new link.'),
      findsOneWidget,
    );
    expect(find.text('Update published note'), findsOneWidget);
  });

  for (final state in <String>[
    'unpublished',
    'publishing',
    'published',
    'failed',
  ]) {
    testWidgets('publish sheet state golden', (tester) async {
      final repo = _PublishRepository(
        published: state == 'published',
        failure: state == 'failed'
            ? const ApplicationError.unexpected(
                message: 'This note is too large to publish.',
              )
            : null,
        pending: state == 'publishing',
      );
      await expectGoldenOnBothPlatforms(
        tester,
        _sheet(
          repository: repo,
          notebook: false,
          startPublishing: state == 'publishing' || state == 'failed',
        ),
        name: 'publish_${state}_phone',
        brightness: Brightness.light,
        viewport: KansoViewport.phone,
        palette: KansoPalette.kansoLight,
      );
    });
  }
}

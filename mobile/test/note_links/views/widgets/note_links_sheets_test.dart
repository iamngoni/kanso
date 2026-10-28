// Kanso — note_links_sheets_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/linking_types.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';
import 'package:kanso/note_links/models/repos/abs/note_links_repository.dart';
import 'package:kanso/note_links/views/widgets/backlinks_sheet.dart';
import 'package:kanso/note_links/views/widgets/outline_sheet.dart';

const _headings = <OutlineHeading>[
  OutlineHeading(level: 1, text: 'Morning', line: 3),
  OutlineHeading(level: 2, text: 'Coffee', line: 9),
];

/// Answers with what it was given.
class _FakeLinks implements NoteLinksRepository {
  _FakeLinks({
    this.outlined = const <OutlineHeading>[],
    this.links = const <LinkRef>[],
    this.mentions = const <LinkRef>[],
    this.outlineError,
    this.backlinksError,
  });

  final List<OutlineHeading> outlined;
  final List<LinkRef> links;
  final List<LinkRef> mentions;
  final ApplicationError? outlineError;
  final ApplicationError? backlinksError;

  @override
  Either<ApplicationError, List<OutlineHeading>> outline(String text) {
    final error = outlineError;
    return error == null ? Right(outlined) : Left(error);
  }

  @override
  Future<Either<ApplicationError, List<LinkRef>>> backlinks(String id) async {
    final error = backlinksError;
    return error == null ? Right(links) : Left(error);
  }

  @override
  Future<Either<ApplicationError, List<LinkRef>>> unlinkedMentions(
    String id,
  ) async => Right(mentions);
}

Widget _app(Widget child) => MaterialApp(
  theme: KansoTheme.light,
  localizationsDelegates: AppLocalizations.localizationsDelegates,
  supportedLocales: AppLocalizations.supportedLocales,
  home: Scaffold(body: child),
);

const _link = LinkRef(
  id: 'note-1',
  title: 'Weekly review',
  line: 4,
  context: 'as said in the plan',
);

void main() {
  testWidgets('the outline lists the headings in order', (tester) async {
    await tester.pumpWidget(
      _app(
        OutlineSheet(
          noteId: 'note-0',
          text: '# Morning\n\n## Coffee',
          links: _FakeLinks(outlined: _headings),
        ),
      ),
    );

    expect(find.text('Morning'), findsOneWidget);
    expect(find.text('Coffee'), findsOneWidget);
    final first = tester.getTopLeft(find.text('Morning')).dy;
    final second = tester.getTopLeft(find.text('Coffee')).dy;
    expect(second, greaterThan(first));
  });

  testWidgets('the outline says when a note has no headings', (tester) async {
    await tester.pumpWidget(
      _app(
        OutlineSheet(
          noteId: 'note-0',
          text: 'no headings',
          links: _FakeLinks(),
        ),
      ),
    );

    expect(find.text('No headings'), findsOneWidget);
  });

  testWidgets('the outline says why it could not read the text', (
    tester,
  ) async {
    await tester.pumpWidget(
      _app(
        OutlineSheet(
          noteId: 'note-0',
          text: 'text',
          links: _FakeLinks(
            outlineError: const ApplicationError.notFound(
              message: 'That note is gone.',
            ),
          ),
        ),
      ),
    );

    expect(find.text('That note is gone.'), findsOneWidget);
  });

  testWidgets('the backlinks list the linked notes and the mentions', (
    tester,
  ) async {
    await tester.pumpWidget(
      _app(
        BacklinksSheet(
          noteId: 'note-0',
          links: _FakeLinks(
            links: const <LinkRef>[_link],
            mentions: const <LinkRef>[_link],
          ),
        ),
      ),
    );
    await tester.pump();

    expect(find.text('Linked from'), findsOneWidget);
    expect(find.text('Unlinked mentions'), findsOneWidget);
    expect(find.text('Weekly review'), findsNWidgets(2));
  });

  testWidgets('the backlinks say when nothing names the note', (tester) async {
    await tester.pumpWidget(
      _app(BacklinksSheet(noteId: 'note-0', links: _FakeLinks())),
    );
    await tester.pump();

    expect(find.text('Nothing links here'), findsOneWidget);
  });

  testWidgets('the backlinks say why they could not be read', (tester) async {
    await tester.pumpWidget(
      _app(
        BacklinksSheet(
          noteId: 'note-0',
          links: _FakeLinks(
            backlinksError: const ApplicationError.notFound(
              message: 'That note is gone.',
            ),
          ),
        ),
      ),
    );
    await tester.pump();

    expect(find.text('That note is gone.'), findsOneWidget);
  });
}

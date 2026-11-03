// Kanso — notebook_home_card_test.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/notebook_home_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/note_list/views/pages/note_list_page.dart';
import 'package:kanso/notebook_home/views/widgets/notebook_home_card.dart';
import 'package:kanso/notebook_home/views/widgets/parent_only_home.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/fake_notebook_home_repository.dart';
import '../../helpers/library_fixtures.dart';
import '../../helpers/register_page_blocs.dart';
import 'home_harness.dart';

const String _book = MockIds.work;

HomeCard _card({
  bool show = true,
  String about = 'Plans and notes for the job.',
  bool opensDirectly = false,
}) => HomeCard(
  show_: show,
  name: 'Work',
  aboutLine: about,
  hasHomeNote: about.isNotEmpty,
  opensDirectly: opensDirectly,
);

// no-prop-test: widget layout and one tap per behaviour.
void main() {
  late FakeNotebookHomeRepository repo;
  late BrowseBlocs blocs;

  setUp(() async {
    repo = FakeNotebookHomeRepository();
    useHomeRepository(repo);
    blocs = await BrowseBlocs.start();
  });

  group('NotebookHomeCard', () {
    testWidgets('shows the name, the About line and Open home', (tester) async {
      repo.cardAnswer = _card();
      final visits = <Uri>[];
      await pumpHome(
        tester,
        blocs,
        () => const NotebookHomeCard(bookId: _book),
        visits: visits,
      );
      expect(find.text('Work'), findsOneWidget);
      expect(find.text('Plans and notes for the job.'), findsOneWidget);
      expect(find.text('Add a Home note'), findsNothing);
      await tester.tap(find.text('Open home ›'));
      await tester.pumpAndSettle();
      expect(visits.single.path, endsWith('/home'));
      expect(visits.single.path, startsWith('/notebooks/'));
    });

    testWidgets('offers "Add a Home note" when there is no About line', (
      tester,
    ) async {
      repo
        ..cardAnswer = _card(about: '')
        ..newHomeId = 'n-home';
      final visits = <Uri>[];
      await pumpHome(
        tester,
        blocs,
        () => const NotebookHomeCard(bookId: _book),
        visits: visits,
      );
      expect(find.text('Add a Home note'), findsOneWidget);
      await tester.tap(find.text('Add a Home note'));
      await tester.pumpAndSettle();
      // The same choice the menu of the page has.
      expect(find.text('Choose Home note…'), findsOneWidget);
      await tester.tap(find.text('New Home note'));
      await tester.pumpAndSettle();
      expect(repo.calls, contains('newHome'));
      expect(visits.single.path, '/note/n-home');
    });

    testWidgets('draws nothing, and takes no room, when there is no card', (
      tester,
    ) async {
      repo.cardAnswer = _card(show: false);
      await pumpHome(
        tester,
        blocs,
        () => const Column(children: <Widget>[NotebookHomeCard(bookId: _book)]),
      );
      expect(find.text('Work'), findsNothing);
      expect(find.text('Open home ›'), findsNothing);
      expect(tester.getSize(find.byType(NotebookHomeCard)), Size.zero);
    });

    testWidgets("sits above the first note of a notebook's list", (
      tester,
    ) async {
      repo.cardAnswer = _card();
      registerPageBlocs(blocs.library);
      await atMockNow(
        () => pumpHome(
          tester,
          blocs,
          () => const NoteListPage(place: ListPlace.notebook(id: _book)),
        ),
      );
      expect(find.text('Open home ›'), findsOneWidget);
      expect(find.text('Weekly review'), findsOneWidget);
      expect(
        tester.getTopLeft(find.text('Open home ›')).dy,
        lessThan(tester.getTopLeft(find.text('Weekly review')).dy),
      );
    });

    testWidgets('is not on All Notes, and the core is not even asked', (
      tester,
    ) async {
      repo.cardAnswer = _card();
      registerPageBlocs(blocs.library);
      await atMockNow(
        () => pumpHome(
          tester,
          blocs,
          () => const NoteListPage(place: ListPlace.all()),
        ),
      );
      expect(find.text('Open home ›'), findsNothing);
      expect(repo.calls, isNot(contains('card')));
    });
  });

  group('ParentOnlyHome', () {
    Widget parent({bool beside = false}) => ParentOnlyHome(
      bookId: _book,
      beside: beside,
      empty: const Text('The usual empty list'),
    );

    testWidgets('opens the homepage when the notebook has only notebooks', (
      tester,
    ) async {
      repo.cardAnswer = _card(opensDirectly: true);
      final visits = <Uri>[];
      await pumpHome(tester, blocs, parent, visits: visits);
      expect(visits.single.path, endsWith('/home'));
      expect(find.text('The usual empty list'), findsNothing);
    });

    testWidgets('opens it in the note pane on a wide window', (tester) async {
      repo.cardAnswer = _card(opensDirectly: true);
      final visits = <Uri>[];
      await pumpHome(
        tester,
        blocs,
        () => parent(beside: true),
        visits: visits,
      );
      expect(visits.single.path, endsWith('/home'));
    });

    testWidgets('keeps the empty list when homepages are off', (tester) async {
      repo.cardAnswer = _card(show: false, opensDirectly: true);
      final visits = <Uri>[];
      await pumpHome(tester, blocs, parent, visits: visits);
      expect(visits, isEmpty);
      expect(find.text('The usual empty list'), findsOneWidget);
    });

    testWidgets('keeps the empty list for a notebook that is just empty', (
      tester,
    ) async {
      repo.cardAnswer = _card();
      final visits = <Uri>[];
      await pumpHome(tester, blocs, parent, visits: visits);
      expect(visits, isEmpty);
      expect(find.text('The usual empty list'), findsOneWidget);
    });
  });
}

// Kanso — notebook_home_goldens_test.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/notebook_home_types.dart';
import 'package:kanso/core/bridge/api/people_types.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/note_list/views/pages/note_list_page.dart';
import 'package:kanso/notebook_home/views/pages/notebook_home_page.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/fake_notebook_home_repository.dart';
import '../../helpers/golden.dart';
import '../../helpers/library_fixtures.dart';
import '../../helpers/register_page_blocs.dart';
import 'home_harness.dart';

const String _book = MockIds.work;

HomePanel _panel() => homePanel(
  <PageSection>[
    homeSection(
      'about',
      'What this is about',
      ai: true,
      items: <PageItem>[
        homeItem(
          'Work holds the plans for the year, the weekly reviews and the '
          'sync design.',
          sources: const <PageSource>[
            PageSource(
              noteId: 'n1',
              title: 'Weekly review',
              updated: '28 Oct',
              start: 0,
              end: 2,
              quote: '',
              label: 'Weekly review',
            ),
          ],
        ),
      ],
    ),
    homeSection(
      'key_notes',
      'Key notes',
      items: <PageItem>[
        homeItem('Weekly review', target: 'n1', meta: '3 links in'),
        homeItem('Sync design', target: 'n2', meta: '2 links in'),
      ],
    ),
    homeSection(
      'open_tasks',
      'Open tasks',
      items: <PageItem>[
        homeItem(
          'Send the contract',
          target: 'n1',
          task: const PageTask(
            noteId: 'n1',
            line: 4,
            text: 'Send the contract',
            checked: false,
          ),
        ),
      ],
      total: 7,
    ),
    homeSection('map', 'Map'),
  ],
  aiOn: true,
  footer: 'Updated just now',
);

const HomeMap _map = HomeMap(
  width: 320,
  height: 200,
  nodes: <HomeMapNode>[
    HomeMapNode(
      id: 'a',
      title: 'A',
      x: 60,
      y: 70,
      radius: 6,
      degree: 2,
      stubs: 2,
    ),
    HomeMapNode(
      id: 'b',
      title: 'B',
      x: 170,
      y: 50,
      radius: 9,
      degree: 3,
      stubs: 0,
    ),
    HomeMapNode(
      id: 'c',
      title: 'C',
      x: 250,
      y: 130,
      radius: 5,
      degree: 1,
      stubs: 3,
    ),
    HomeMapNode(
      id: 'd',
      title: 'D',
      x: 110,
      y: 150,
      radius: 4,
      degree: 1,
      stubs: 0,
    ),
  ],
  edges: <HomeMapEdge>[
    HomeMapEdge(a: 0, b: 1),
    HomeMapEdge(a: 1, b: 2),
    HomeMapEdge(a: 0, b: 3),
  ],
);

// no-prop-test: pictures.
void main() {
  for (final brightness in Brightness.values) {
    final name = brightness.name;

    group('the notebook homepage pictures, $name', () {
      testWidgets('the homepage', (tester) async {
        final repo = FakeNotebookHomeRepository(
          panelAnswer: _panel(),
          mapAnswer: _map,
        );
        useHomeRepository(repo);
        final blocs = await BrowseBlocs.start();
        await expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(const NotebookHomePage(bookId: _book)),
          name: 'phone_notebook_home_$name',
          size: phoneSize,
          brightness: brightness,
        );
      });
    });
  }

  group('the notebook list with its card', () {
    testWidgets('the card above the notes', (tester) async {
      final repo = FakeNotebookHomeRepository(
        cardAnswer: const HomeCard(
          show_: true,
          name: 'Work',
          aboutLine: 'The plans for the year and the weekly reviews.',
          hasHomeNote: true,
          opensDirectly: false,
        ),
      );
      useHomeRepository(repo);
      final blocs = await BrowseBlocs.start();
      registerPageBlocs(blocs.library);
      await atMockNow(
        () => expectGoldenOnBothPlatforms(
          tester,
          blocs.provide(
            const NoteListPage(place: ListPlace.notebook(id: _book)),
          ),
          name: 'phone_notebook_list_card_light',
          size: phoneSize,
          brightness: Brightness.light,
        ),
      );
    });
  });
}

// Kanso — notebook_home_page_test.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/notebook_home_types.dart';
import 'package:kanso/core/bridge/api/people_types.dart';
import 'package:kanso/core/views/widgets/kanso_checkbox.dart';
import 'package:kanso/core/views/widgets/kanso_icon_button.dart';
import 'package:kanso/core/views/widgets/kanso_switch.dart';
import 'package:kanso/core/views/widgets/kanso_top_bar.dart';
import 'package:kanso/notebook_home/views/pages/notebook_home_page.dart';
import 'package:kanso/notebook_home/views/widgets/home_map_card.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/fake_notebook_home_repository.dart';
import '../../helpers/library_fixtures.dart';
import 'home_harness.dart';

const String _book = MockIds.work;

PageTask _task(bool checked) => PageTask(
  noteId: 'n1',
  line: 4,
  text: 'Send the contract',
  checked: checked,
);

HomePanel _computed() => homePanel(<PageSection>[
  homeSection(
    'key_notes',
    'Key notes of the week',
    items: <PageItem>[homeItem('Weekly review', target: 'n1')],
  ),
  homeSection(
    'recent',
    'Lately',
    items: <PageItem>[
      homeItem('Sync design', target: 'n2', group: 'Today'),
      homeItem('Plans', target: 'n3', group: 'Yesterday'),
    ],
  ),
  homeSection(
    'open_tasks',
    'Things to do',
    items: <PageItem>[
      homeItem('Send the contract', target: 'n1', task: _task(false)),
    ],
    total: 12,
  ),
  homeSection(
    'inside',
    'Notebooks within',
    items: <PageItem>[
      homeItem('Plans', target: 'book:plans', meta: '8 notes'),
      homeItem('Private', target: 'book:private', tag: 'locked'),
    ],
  ),
], footer: 'Updated just now');

// no-prop-test: widget layout and one tap per behaviour.
void main() {
  late FakeNotebookHomeRepository repo;
  late BrowseBlocs blocs;

  setUp(() async {
    repo = FakeNotebookHomeRepository(panelAnswer: _computed());
    useHomeRepository(repo);
    blocs = await BrowseBlocs.start();
  });

  Future<void> pump(WidgetTester tester, {bool inPane = false}) async {
    await pumpHome(
      tester,
      blocs,
      () => NotebookHomePage(bookId: _book, inPane: inPane),
    );
  }

  testWidgets('draws the computed sections the core sent, in its order', (
    tester,
  ) async {
    await pump(tester);
    // The notebook's name is the bar's title, and the header line is the
    // core's.
    expect(
      find.descendant(
        of: find.byType(KansoTopBar),
        matching: find.text('Work'),
      ),
      findsOneWidget,
    );
    expect(find.text('4 notes'), findsOneWidget);
    final titles = <String>[
      'Key notes of the week',
      'Lately',
      'Things to do',
      'Notebooks within',
    ];
    final tops = <double>[
      for (final t in titles) tester.getTopLeft(find.text(t)).dy,
    ];
    expect(tops, orderedEquals(<double>[...tops]..sort()));
    // The recent notes are grouped by day.
    expect(find.text('Today'), findsOneWidget);
    expect(find.text('Yesterday'), findsOneWidget);
    // The footer is one muted line at the end.
    expect(find.text('Updated just now'), findsOneWidget);
    expect(
      tester.getTopLeft(find.text('Updated just now')).dy,
      greaterThan(tester.getTopLeft(find.text('Notebooks within')).dy),
    );
  });

  testWidgets('a locked notebook inside shows a lock and no count', (
    tester,
  ) async {
    await pump(tester);
    expect(find.text('Private'), findsOneWidget);
    expect(find.text('8 notes'), findsOneWidget);
    // Only the open one has a count line: the locked row has no meta.
    expect(find.text('Plans'), findsNWidgets(2));
  });

  testWidgets('"Show all" asks the core for every task', (tester) async {
    await pump(tester);
    await tester.tap(find.text('Show all 12'));
    await tester.pumpAndSettle();
    expect(repo.calls, contains('showAll:open_tasks'));
  });

  testWidgets('ticking a task goes through the page and reads it again', (
    tester,
  ) async {
    await pump(tester);
    final opens = repo.calls.where((c) => c == 'open').length;
    await tester.tap(find.byType(KansoCheckbox));
    await tester.pumpAndSettle();
    expect(repo.ticked?.text, 'Send the contract');
    expect(repo.tickedTo, isTrue);
    expect(repo.calls.where((c) => c == 'open').length, opens + 1);
  });

  testWidgets("no open tasks says so in the core's words", (tester) async {
    repo.panelAnswer = homePanel(<PageSection>[
      homeSection('open_tasks', 'Things to do'),
    ]);
    await pump(tester);
    expect(find.text('No open tasks'), findsOneWidget);
  });

  testWidgets("an unwritten AI section offers the core's first-visit button", (
    tester,
  ) async {
    repo.panelAnswer = homePanel(<PageSection>[
      homeSection(
        'about',
        'What this is about',
        ai: true,
        state: PageSectionState.unwritten,
      ),
    ], aiOn: true);
    await pump(tester);
    expect(find.text('Write this summary'), findsOneWidget);
    await tester.tap(find.text('Write this summary'));
    await tester.pumpAndSettle();
    expect(repo.calls, contains('refresh:about'));
  });

  testWidgets('with AI off there is no button to write a section', (
    tester,
  ) async {
    repo.panelAnswer = homePanel(<PageSection>[
      homeSection(
        'about',
        'What this is about',
        ai: true,
        state: PageSectionState.unwritten,
      ),
    ]);
    await pump(tester);
    expect(find.text('Write this summary'), findsNothing);
    expect(find.text('Ask about this notebook'), findsNothing);
  });

  testWidgets('a stale section says so, and the bar has Ask and Refresh', (
    tester,
  ) async {
    repo.panelAnswer = homePanel(<PageSection>[
      homeSection(
        'about',
        'What this is about',
        ai: true,
        state: PageSectionState.stale,
        staleLine: '3 notes changed since this was written',
        items: <PageItem>[homeItem('A notebook of plans.')],
      ),
    ], aiOn: true);
    final visits = <Uri>[];
    await pumpHome(
      tester,
      blocs,
      () => const NotebookHomePage(bookId: _book),
      visits: visits,
    );
    expect(find.text('3 notes changed since this was written'), findsOneWidget);
    expect(find.text('A notebook of plans.'), findsOneWidget);
    // Ask about this notebook opens Ask limited to it.
    await tester.tap(find.bySemanticsLabel('Ask about this notebook'));
    await tester.pumpAndSettle();
    expect(visits.single.path, '/ask');
    expect(visits.single.queryParameters['book'], _book);
    expect(visits.single.queryParameters['label'], 'Work');
  });

  testWidgets('a locked, closed notebook draws no sections and no counts', (
    tester,
  ) async {
    repo.panelAnswer = homePanel(
      <PageSection>[],
      name: '',
      line: '',
      locked: true,
    );
    await pump(tester);
    expect(find.text('This notebook is locked'), findsOneWidget);
    expect(find.text('Key notes of the week'), findsNothing);
    expect(find.textContaining('notes'), findsNothing);
    // No menu to make changes behind the lock either.
    expect(
      find.descendant(
        of: find.byType(KansoTopBar),
        matching: find.byType(KansoIconButton),
      ),
      findsOneWidget, // the back button only
    );
  });

  testWidgets('a locked notebook that is open draws its computed sections', (
    tester,
  ) async {
    repo.panelAnswer = homePanel(
      <PageSection>[
        homeSection(
          'recent',
          'Lately',
          items: <PageItem>[homeItem('Secret plans', target: 'n9')],
        ),
      ],
      lockedOpen: true,
      footer: 'Only on this device while it is open',
    );
    await pump(tester);
    expect(find.text('Lately'), findsOneWidget);
    expect(find.text('Secret plans'), findsOneWidget);
    expect(find.text('Only on this device while it is open'), findsOneWidget);
    expect(find.text('This notebook is locked'), findsNothing);
  });

  testWidgets('the map is read after the sections and draws as a card', (
    tester,
  ) async {
    repo
      ..panelAnswer = homePanel(<PageSection>[homeSection('map', 'Map')])
      ..mapAnswer = const HomeMap(
        width: 320,
        height: 200,
        nodes: <HomeMapNode>[
          HomeMapNode(
            id: 'a',
            title: 'A',
            x: 60,
            y: 60,
            radius: 5,
            degree: 1,
            stubs: 2,
          ),
          HomeMapNode(
            id: 'b',
            title: 'B',
            x: 200,
            y: 120,
            radius: 8,
            degree: 1,
            stubs: 0,
          ),
        ],
        edges: <HomeMapEdge>[HomeMapEdge(a: 0, b: 1)],
      );
    final visits = <Uri>[];
    await pumpHome(
      tester,
      blocs,
      () => const NotebookHomePage(bookId: _book),
      visits: visits,
    );
    expect(repo.calls.indexOf('map'), greaterThan(repo.calls.indexOf('open')));
    expect(
      find.byKey(const ValueKey<String>('home-map-canvas')),
      findsOneWidget,
    );
    final card = tester.getSize(find.byType(HomeMapCard));
    expect(card.width / card.height, closeTo(16 / 10, 0.01));
    // A tap opens the graph narrowed to this notebook.
    await tester.tap(find.byType(HomeMapCard));
    await tester.pumpAndSettle();
    expect(visits.single.path, '/graph');
    expect(visits.single.queryParameters['book'], _book);
  });

  testWidgets('a section folds away when its title is tapped', (tester) async {
    await pump(tester);
    expect(find.text('Weekly review'), findsOneWidget);
    await tester.tap(find.text('Key notes of the week'));
    await tester.pumpAndSettle();
    expect(find.text('Weekly review'), findsNothing);
  });

  testWidgets('hiding a section goes through the core', (tester) async {
    await pump(tester);
    await tester.tap(find.bySemanticsLabel('Key notes of the week').last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Hide section'));
    await tester.pumpAndSettle();
    expect(repo.calls, contains('hide:key_notes:true'));
  });

  testWidgets("the section titles and words are the core's, not ours", (
    tester,
  ) async {
    // The titles are not any English this app could have: whatever the core
    // names a section is what is drawn, and its own words replace ours.
    repo
      ..words_ = const HomeWords(
        aiOff: '1',
        locked: '2',
        firstVisitButton: 'Schreib das',
        noOpenTasks: 'Nichts offen',
        homeRow: '4',
        askButton: 'Frag das Notizbuch',
        openHome: '5',
        addHomeNote: '6',
        homeMoved: '7',
        sectionKeys: <String>[],
      )
      ..panelAnswer = homePanel(<PageSection>[
        homeSection('key_notes', 'Wichtige Notizen'),
        homeSection('recent', 'Zuletzt'),
        homeSection('open_tasks', 'Aufgaben'),
        homeSection(
          'about',
          'Worum es geht',
          ai: true,
          state: PageSectionState.unwritten,
        ),
      ], aiOn: true);
    await pump(tester);
    for (final title in <String>[
      'Wichtige Notizen',
      'Zuletzt',
      'Aufgaben',
      'Worum es geht',
    ]) {
      expect(find.text(title), findsOneWidget);
    }
    for (final english in <String>[
      'Key notes',
      'Recent',
      'Open tasks',
      'About',
      'No open tasks',
      'Write this summary',
    ]) {
      expect(find.text(english), findsNothing, reason: english);
    }
    expect(find.text('Schreib das'), findsOneWidget);
    expect(find.bySemanticsLabel('Frag das Notizbuch'), findsOneWidget);
  });

  testWidgets('the menu chooses a Home note and customises the layout', (
    tester,
  ) async {
    repo
      ..candidatesAnswer = const <HomeCandidate>[
        HomeCandidate(id: 'n7', title: 'About Work', current: false),
      ]
      ..sectionsAnswer = const <HomeSectionSetting>[
        HomeSectionSetting(key: 'key_notes', title: 'Key notes', hidden: false),
        HomeSectionSetting(key: 'recent', title: 'Recent', hidden: false),
        HomeSectionSetting(key: 'map', title: 'Map', hidden: false),
      ];
    await pump(tester);
    final menu = find.descendant(
      of: find.byType(KansoTopBar),
      matching: find.byType(KansoIconButton),
    );

    await tester.tap(menu.last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Choose Home note…'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('About Work'));
    await tester.pumpAndSettle();
    expect(repo.calls, contains('chooseHome:n7'));

    await tester.tap(menu.last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Customise home…'));
    await tester.pumpAndSettle();
    // Hide Recent, drag Map to the top, and press Done.
    await tester.tap(find.byType(KansoSwitch).at(1));
    await tester.pumpAndSettle();
    await tester.drag(
      find.byType(ReorderableDragStartListener).at(2),
      const Offset(0, -220),
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Done'));
    await tester.pumpAndSettle();
    expect(repo.customisedHidden, <String>['recent']);
    expect(repo.customisedOrder, <String>['map', 'key_notes', 'recent']);
  });

  testWidgets('Reset layout goes back to the default', (tester) async {
    repo.sectionsAnswer = const <HomeSectionSetting>[
      HomeSectionSetting(key: 'key_notes', title: 'Key notes', hidden: true),
    ];
    await pump(tester);
    await tester.tap(
      find
          .descendant(
            of: find.byType(KansoTopBar),
            matching: find.byType(KansoIconButton),
          )
          .last,
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('Customise home…'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Reset layout'));
    await tester.pumpAndSettle();
    expect(repo.calls, contains('resetLayout'));
  });

  testWidgets('New Home note makes the note and opens it', (tester) async {
    repo.newHomeId = 'n-new';
    final visits = <Uri>[];
    await pumpHome(
      tester,
      blocs,
      () => const NotebookHomePage(bookId: _book),
      visits: visits,
    );
    await tester.tap(
      find
          .descendant(
            of: find.byType(KansoTopBar),
            matching: find.byType(KansoIconButton),
          )
          .last,
    );
    await tester.pumpAndSettle();
    await tester.tap(find.text('New Home note'));
    await tester.pumpAndSettle();
    expect(repo.calls, contains('newHome'));
    expect(visits.single.path, '/note/n-new');
  });
}

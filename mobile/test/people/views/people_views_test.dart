// Kanso — people_views_test.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/people_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/people/models/repos/abs/people_repository.dart';
import 'package:kanso/people/views/pages/people_list_page.dart';
import 'package:kanso/people/views/widgets/page_panel.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/kanso_test_app.dart';

class _MockPeople extends Mock implements PeopleRepository {}

PageSection _section(
  String key,
  String title, {
  bool ai = false,
  List<PageItem> items = const <PageItem>[],
  PageSectionState state = PageSectionState.fresh,
  String emptyLine = '',
}) => PageSection(
  key: key,
  title: title,
  ai: ai,
  items: items,
  state: state,
  changed: 0,
  writtenMs: 0,
  total: items.length,
  note: '',
  emptyLine: emptyLine,
  staleLine: '',
);

PageItem _item(String text) => PageItem(
  text: text,
  who: '',
  sources: const <PageSource>[],
  meta: '28 Oct',
  group: '',
  target: '',
  tag: '',
  shared: false,
);

PagePanel _panel(List<PageSection> sections) => PagePanel(
  header: const PageHeaderInfo(
    title: 'Sam Moyo',
    kind: PeopleKind.person,
    aliases: <String>['Sam'],
    notes: 2,
    lastMentioned: '28 Oct',
    line: 'Also known as Sam · 2 notes',
  ),
  sections: sections,
  hidden: const <PageHidden>[],
  aiOn: false,
  due: const <String>[],
);

// no-prop-test: widget layout and one tap per behaviour.
void main() {
  late _MockPeople people;

  setUpAll(() {
    registerFallbackValue(PeopleKind.person);
  });

  setUp(() {
    people = _MockPeople();
    if (getIt.isRegistered<PeopleRepository>()) {
      getIt.unregister<PeopleRepository>();
    }
    getIt.registerSingleton<PeopleRepository>(people);
  });

  tearDown(() => getIt.unregister<PeopleRepository>());

  testWidgets('a note that is not a page shows nothing', (tester) async {
    when(() => people.page(any())).thenAnswer((_) async => right(null));
    await tester.pumpWidget(
      const KansoTestApp(home: PeoplePagePanel(noteId: 'n1')),
    );
    await tester.pumpAndSettle();
    expect(find.text('Also known as Sam · 2 notes'), findsNothing);
  });

  testWidgets('a page shows its header and sections, and hides one', (
    tester,
  ) async {
    final panel = _panel(<PageSection>[
      _section('timeline', 'Timeline', items: <PageItem>[_item('Met Sam')]),
      _section(
        'promises',
        'Promises',
        ai: true,
        state: PageSectionState.off,
        emptyLine: 'Turn on AI to write this.',
      ),
    ]);
    when(() => people.page('n1')).thenAnswer((_) async => right(panel));
    when(
      () => people.hide('n1', 'timeline', hidden: true),
    ).thenAnswer((_) async => right(_panel(const <PageSection>[])));
    await tester.pumpWidget(
      const KansoTestApp(
        home: SingleChildScrollView(child: PeoplePagePanel(noteId: 'n1')),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.text('Also known as Sam · 2 notes'), findsOneWidget);
    expect(find.text('Timeline'), findsOneWidget);
    expect(find.text('Met Sam'), findsOneWidget);
    expect(find.text('Turn on AI to write this.'), findsOneWidget);

    // A section folds away when its title is tapped.
    await tester.tap(find.text('Timeline'));
    await tester.pumpAndSettle();
    expect(find.text('Met Sam'), findsNothing);
  });

  testWidgets('the list offers a suggestion and creates it only on a tap', (
    tester,
  ) async {
    when(() => people.detect(force: false)).thenAnswer((_) async => right(1));
    when(() => people.pages()).thenAnswer((_) async => right(<PeoplePage>[]));
    when(() => people.suggestions()).thenAnswer(
      (_) async => right(<PeopleSuggestion>[
        const PeopleSuggestion(
          key: 'tendai chirwa',
          name: 'Tendai Chirwa',
          kind: PeopleKind.person,
          detail: 'person · mentioned in 3 notes',
          alsoLine: '',
          aliasOfId: '',
          aliasOfTitle: '',
        ),
      ]),
    );
    when(() => people.status()).thenAnswer(
      (_) async => right(
        const PeopleStatus(
          pages: 0,
          suggestions: 1,
          newLabel: '1 new',
          suggestOn: true,
          aiOn: false,
        ),
      ),
    );
    await tester.pumpWidget(const KansoTestApp(home: PeopleListPage()));
    await tester.pumpAndSettle();
    expect(find.text('Tendai Chirwa'), findsOneWidget);
    expect(find.text('Create page'), findsOneWidget);
    expect(find.text('Not a person'), findsOneWidget);
    expect(find.text('Not now'), findsOneWidget);
    verifyNever(() => people.create(any(), any(), any()));
  });

  testWidgets('a failure is told, not thrown', (tester) async {
    when(() => people.detect(force: false)).thenAnswer((_) async => right(0));
    when(() => people.pages()).thenAnswer(
      (_) async => left(const ApplicationError.unexpected(message: 'boom')),
    );
    when(
      () => people.suggestions(),
    ).thenAnswer((_) async => right(<PeopleSuggestion>[]));
    when(() => people.status()).thenAnswer(
      (_) async => left(const ApplicationError.unexpected(message: 'boom')),
    );
    await tester.pumpWidget(const KansoTestApp(home: PeopleListPage()));
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
  });
}

// Kanso — people_settings_homes_test.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/people_types.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/people/models/repos/abs/people_repository.dart';
import 'package:kanso/people/views/pages/people_settings_page.dart';
import 'package:mocktail/mocktail.dart';

import '../../helpers/fake_notebook_home_repository.dart';
import '../../helpers/kanso_test_app.dart';
import 'home_harness.dart';

class _MockPeople extends Mock implements PeopleRepository {}

// no-prop-test: widget layout and one tap per behaviour.
void main() {
  late FakeNotebookHomeRepository homes;
  late _MockPeople people;

  setUp(() {
    homes = FakeNotebookHomeRepository();
    useHomeRepository(homes);
    people = _MockPeople();
    if (getIt.isRegistered<PeopleRepository>()) {
      getIt.unregister<PeopleRepository>();
    }
    getIt.registerSingleton<PeopleRepository>(people);
    addTearDown(() => getIt.unregister<PeopleRepository>());
    when(() => people.settings()).thenAnswer(
      (_) async => right(
        const PeopleSettings(
          suggest: true,
          keepIn: '',
          writeWithAi: true,
          refresh: 'open',
          detectWithAi: false,
        ),
      ),
    );
    when(() => people.dismissed()).thenAnswer((_) async => right(<String>[]));
  });

  testWidgets('the homepage settings are saved with the shared refresh '
      'choice', (tester) async {
    // Tall enough for the whole list to be built.
    tester.view
      ..physicalSize = const Size(393, 2400)
      ..devicePixelRatio = 1;
    addTearDown(tester.view.reset);
    await tester.pumpWidget(const KansoTestApp(home: PeopleSettingsPage()));
    await tester.pumpAndSettle();
    expect(find.text('Show notebook homepages'), findsOneWidget);
    expect(find.text('Write notebook summaries with AI'), findsOneWidget);
    // The refresh choice is People's one: there is only one row for it.
    expect(find.text('Refresh AI sections'), findsOneWidget);

    await tester.tap(find.text('Show notebook homepages'));
    await tester.pumpAndSettle();
    expect(homes.settingsAnswer.showHomepages, isFalse);
    expect(homes.settingsAnswer.summariesWithAi, isTrue);
    // The shared refresh setting travels with it, as People has it.
    expect(homes.settingsAnswer.refresh, 'open');
  });
}

// Kanso — kanso_list_scrollbar_test.dart
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_list_scrollbar.dart';

import '../../../helpers/seeded_appearance_bloc.dart';

void main() {
  testWidgets('leaves the scrollbar suppressed in When scrolling mode', (
    tester,
  ) async {
    final appearance = SeededAppearanceBloc(const AppearanceState());
    addTearDown(appearance.close);

    await tester.pumpWidget(
      BlocProvider<AppearanceBloc>.value(
        value: appearance,
        child: MaterialApp(
          home: Scaffold(
            body: KansoListScrollbar(
              child: ListView(
                children: const <Widget>[
                  SizedBox(height: 1600, child: Text('row')),
                ],
              ),
            ),
          ),
        ),
      ),
    );

    expect(find.byType(Scrollbar), findsNothing);
  });

  testWidgets('keeps the scrollbar visible in light and dark themes', (
    tester,
  ) async {
    final appearance = SeededAppearanceBloc(
      const AppearanceState(alwaysShowScrollbars: true),
    );
    addTearDown(appearance.close);

    for (final brightness in Brightness.values) {
      await tester.pumpWidget(
        BlocProvider<AppearanceBloc>.value(
          value: appearance,
          child: MaterialApp(
            theme: ThemeData(brightness: brightness),
            home: Scaffold(
              body: KansoListScrollbar(
                child: ListView(
                  children: List<Widget>.generate(
                    30,
                    (index) => SizedBox(height: 48, child: Text('$index')),
                  ),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pump();
      final scrollbar = tester.widget<Scrollbar>(find.byType(Scrollbar));
      expect(scrollbar.thumbVisibility, isTrue);
    }
  });
}

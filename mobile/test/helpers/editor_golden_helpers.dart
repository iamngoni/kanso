// Kanso — editor_golden_helpers.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/editor/state/editor/editor_bloc.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';

import 'editor_fixture.dart';

/// The height the keyboard is drawn at on a phone.
const double phoneKeyboard = 336;

/// [page] with a block where the keyboard would be, as the mocks have.
Widget withKeyboard(Widget page, [double height = phoneKeyboard]) {
  return Stack(
    children: <Widget>[
      page,
      Positioned(
        left: 0,
        right: 0,
        bottom: 0,
        height: height,
        child: Builder(
          builder: (context) => ColoredBox(color: context.colors.selected),
        ),
      ),
    ],
  );
}

/// Puts the keyboard up: the page makes room for it.
Future<void> keyboardUp(
  WidgetTester tester, [
  double height = phoneKeyboard,
]) async {
  tester.view.viewInsets = FakeViewPadding(
    bottom: height * tester.view.devicePixelRatio,
  );
  await tester.pump();
  await tester.pump();
}

EditorBloc editorBloc(WidgetTester tester) =>
    tester.element(find.byType(KansoEditableText)).read<EditorBloc>();

/// The note with [last] at the end of the text.
String bodyEndingWith(String last) =>
    '$goldenFront## Open questions\n'
    '- How long a device may stay offline\n'
    '- What a conflict looks like to a person\n'
    '\n'
    '$last';

/// A spell checker that marks every `teh` as a misspelling of `the`.
class TehSpellCheckService implements SpellCheckService {
  @override
  Future<List<SuggestionSpan>?> fetchSpellCheckSuggestions(
    Locale locale,
    String text,
  ) async => <SuggestionSpan>[
    for (final match in RegExp('teh').allMatches(text))
      SuggestionSpan(
        TextRange(start: match.start, end: match.end),
        const <String>['the', 'tech', 'ten'],
      ),
  ];
}

// Kanso — note_page_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/note/views/pages/note_page.dart';
import 'package:kanso/note/views/widgets/note_pane.dart';

import '../../../helpers/browse_blocs.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/register_page_blocs.dart';
import '../../../helpers/seeded_appearance_bloc.dart';

void main() {
  group('NotePage', () {
    testWidgets('reads the note it was opened with, and shows it', (
      tester,
    ) async {
      final blocs = await BrowseBlocs.start();
      registerPageBlocs(blocs.library, renderer: blocs.renderer);
      final appearance = SeededAppearanceBloc(const AppearanceState());
      addTearDown(appearance.close);
      await atMockNow(
        () => pumpRouted(
          tester,
          BlocProvider<AppearanceBloc>.value(
            value: appearance,
            child: blocs.provide(
              const NotePage(noteId: MockIds.syncDesign),
            ),
          ),
        ),
      );
      expect(find.byType(NotePane), findsOneWidget);
      expect(find.text('Sync design notes'), findsOneWidget);
      expect(blocs.renderer.themes, <String>['kanso-light']);
    });

    testWidgets('is a visit, however the note was opened', (tester) async {
      final blocs = await BrowseBlocs.start();
      registerPageBlocs(blocs.library, renderer: blocs.renderer);
      final appearance = SeededAppearanceBloc(const AppearanceState());
      addTearDown(appearance.close);
      await atMockNow(
        () => pumpRouted(
          tester,
          BlocProvider<AppearanceBloc>.value(
            value: appearance,
            child: blocs.provide(const NotePage(noteId: MockIds.syncDesign)),
          ),
        ),
      );
      await tester.pump();
      expect(blocs.history.state.history.current, MockIds.syncDesign);
    });

    testWidgets('highlights in the theme the screen is in, and again when it '
        'changes', (tester) async {
      final blocs = await BrowseBlocs.start();
      registerPageBlocs(blocs.library, renderer: blocs.renderer);
      final appearance = SeededAppearanceBloc(const AppearanceState());
      addTearDown(appearance.close);
      final brightness = ValueNotifier<Brightness>(Brightness.light);
      addTearDown(brightness.dispose);
      await atMockNow(
        () => pumpRouted(
          tester,
          BlocProvider<AppearanceBloc>.value(
            value: appearance,
            child: blocs.provide(
              ValueListenableBuilder<Brightness>(
                valueListenable: brightness,
                builder: (context, value, _) => Theme(
                  data: value == Brightness.dark
                      ? KansoTheme.dark
                      : KansoTheme.light,
                  child: const NotePage(noteId: MockIds.syncDesign),
                ),
              ),
            ),
          ),
        ),
      );
      expect(blocs.renderer.themes, <String>['kanso-light']);
      brightness.value = Brightness.dark;
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(blocs.renderer.themes, <String>['kanso-light', 'kanso-dark']);
      // The note was not read again for it.
      expect(find.text('Sync design notes'), findsOneWidget);
    });

    testWidgets('is a new note for a new id', (tester) async {
      final blocs = await BrowseBlocs.start();
      registerPageBlocs(blocs.library, renderer: blocs.renderer);
      final appearance = SeededAppearanceBloc(const AppearanceState());
      addTearDown(appearance.close);
      final id = ValueNotifier<String>(MockIds.syncDesign);
      addTearDown(id.dispose);
      await atMockNow(
        () => pumpRouted(
          tester,
          BlocProvider<AppearanceBloc>.value(
            value: appearance,
            child: blocs.provide(
              ValueListenableBuilder<String>(
                valueListenable: id,
                builder: (context, value, _) => NotePage(noteId: value),
              ),
            ),
          ),
        ),
      );
      id.value = MockIds.privateJournal;
      await tester.pump();
      await tester.pump();
      await tester.pump();
      expect(find.text('Private journal'), findsOneWidget);
      expect(find.text('This note is locked'), findsOneWidget);
    });
  });
}

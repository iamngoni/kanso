// Kanso — appearance_flow_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:kanso/core/bridge/api/types.dart' show DiagramRenderer;
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/kanso_text_size.dart';
import 'package:kanso/core/models/repos/abs/file_picker_repository.dart';
import 'package:kanso/core/state/appearance/appearance_bloc.dart';
import 'package:kanso/injection.dart';
import 'package:kanso/themes/views/widgets/installed_theme_card.dart';
import 'package:kanso/themes/views/widgets/theme_picker_pane.dart';

import 'helpers/flow_driver.dart';
import 'helpers/seed_library.dart';

/// The file picker of the flow: the system's sheet cannot be driven from a
/// test, so it chooses the file the flow wrote.
class _ChosenFile implements FilePickerRepository {
  _ChosenFile(this.path);

  final String path;

  @override
  Future<Either<ApplicationError, String?>> pickFile() async =>
      Right<ApplicationError, String?>(path);
}

/// Changing how the app looks as a person does: in Settings, the Themes page
/// (a syntax theme, a preview theme, a theme imported from a file), the
/// preview text size and the diagram renderer.
///
/// Run it with
/// `flutter test integration_test/appearance_flow_test.dart -d <device>`.
void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  Finder card(String kind, String id) =>
      find.byKey(ValueKey<String>('$kind-$id'));

  Future<void> bringIntoView(WidgetTester tester, Finder finder) async {
    await tester.ensureVisible(finder);
    await tester.pumpAndSettle();
  }

  testWidgets('pick a syntax and a preview theme, import a theme, and set '
      'the preview text size and the diagrams', (tester) async {
    final semantics = tester.ensureSemantics();
    await startApp(tester);
    await seedLibrary();
    final themeFile =
        File(
          '${Directory.systemTemp.createTempSync('kanso-theme').path}/flow.json',
        )..writeAsStringSync(
          jsonEncode(<String, Object?>{
            'name': 'Flow',
            'author': 'Ana',
            'themes': <Object?>[
              <String, Object?>{
                'name': 'Flow Dark',
                'appearance': 'dark',
                'style': <String, Object?>{
                  'editor.background': '#181825ff',
                  'text': '#cdd6f4ff',
                },
              },
            ],
          }),
        );
    // The system's file picker is replaced by one that chooses the file.
    await getIt.unregister<FilePickerRepository>();
    getIt.registerSingleton<FilePickerRepository>(_ChosenFile(themeFile.path));
    final appearance = getIt<AppearanceBloc>();

    // Settings, from the library.
    await pumpUntil(tester, inList('Weekly review'));
    await openLibrary(tester);
    await tester.tap(find.bySemanticsLabel('Settings'));
    await tester.pumpAndSettle();
    expect(find.text('Appearance'), findsOneWidget);

    // Themes: the three sections and Theme files.
    await tester.tap(find.text('Theme'));
    await pumpUntil(tester, find.byType(InstalledThemeCard));
    for (final heading in <String>['Theme', 'Editor syntax', 'Preview']) {
      expect(find.text(heading), findsWidgets, reason: heading);
    }

    // A syntax theme of its own: the interface keeps its theme.
    final interface = appearance.state.selection.ui;
    await bringIntoView(tester, card('syntax', 'nord'));
    await tester.tap(card('syntax', 'nord'));
    await tester.pumpAndSettle();
    expect(appearance.state.selection.syntax, 'nord');
    expect(appearance.state.selection.ui, interface);

    // A preview theme of its own.
    await bringIntoView(tester, card('preview', 'dracula'));
    await tester.tap(card('preview', 'dracula'));
    await tester.pumpAndSettle();
    expect(appearance.state.selection.preview, 'dracula');
    expect(appearance.state.selection.syntax, 'nord');

    // Same as theme takes the syntax theme back.
    final same = find.byKey(const ValueKey<String>('same-syntax'));
    await bringIntoView(tester, same);
    await tester.tap(same);
    await tester.pumpAndSettle();
    expect(appearance.state.selection.syntax, isNull);

    // Import a theme from a file: it is listed in every section and the page
    // says so.
    final page = find.descendant(
      of: find.byType(ThemePickerPane),
      matching: find.byType(Scrollable),
    );
    await tester.drag(page.first, const Offset(0, -4000));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Import Theme…'));
    await pumpUntil(tester, find.textContaining('Imported'));
    expect(card('ui', 'flow-dark'), findsOneWidget);

    // Back to Settings: the preview text size and the diagrams.
    await tester.tap(find.bySemanticsLabel('Back').first);
    await tester.pumpAndSettle();
    expect(find.text('Preview text size'), findsOneWidget);
    await tester.tap(find.text('Preview text size'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Larger').last);
    await tester.pumpAndSettle();
    expect(appearance.state.previewTextSize, KansoTextSize.larger);
    expect(appearance.state.textSize, KansoTextSize.standard);
    await tester.tap(find.bySemanticsLabel('Back').first);
    await tester.pumpAndSettle();
    expect(find.text('Larger'), findsOneWidget);

    expect(appearance.state.diagramRenderer, DiagramRenderer.pele);
    await tester.tap(find.text('Kanso'));
    await tester.pumpAndSettle();
    expect(appearance.state.diagramRenderer, DiagramRenderer.kanso);
    await tester.tap(find.text('Pele'));
    await tester.pumpAndSettle();
    expect(appearance.state.diagramRenderer, DiagramRenderer.pele);

    // Icon themes: the page is there, with its import row.
    await tester.tap(find.text('Icon themes'));
    await tester.pumpAndSettle();
    expect(find.text('Import icon theme'), findsOneWidget);
    semantics.dispose();
  });
}

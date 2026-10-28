// Kanso — themes_suite.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';

import 'open_library.dart';

/// A small theme file used to check importing through the shared facade.
const String _themeJson = '''
{
  "schema_version": 1,
  "name": "Suite Import",
  "appearance": "light",
  "ui": {"editor": "#FCFAF4", "text": "#26241F", "accent": "#5E7895"}
}
''';

/// The themes, through the bridge: listing, notices, fitting, highlighting,
/// importing colour themes and importing icon themes.
void themesSuite(OpenLibrary open) {
  group('KansoThemes', () {
    test('lists the desktop themes and reports no unreadable files', () async {
      final themes = (await open()).themes();
      final rows = await themes.list();
      expect(rows.length, greaterThan(20));
      expect(rows.take(2).map((theme) => theme.id), <String>[
        'kanso-light',
        'kanso-dark',
      ]);
      expect(rows.map((theme) => theme.id).toSet(), hasLength(rows.length));
      expect(await themes.notices(), isEmpty);
    });

    test(
      'fits the interface, editor syntax and rendered preview together',
      () async {
        final themes = (await open()).themes();
        final fitted = await themes.fit(
          ui: 'kanso-light',
          syntax: 'nord',
          preview: 'kanso-dark',
        );
        expect(fitted.name, isNotEmpty);
        expect(fitted.dark, isFalse);
        expect(fitted.palette.background, 0xFCFAF4);
        final syntaxOnly = await themes.fit(ui: 'kanso-light', syntax: 'nord');
        final previewOnly = await themes.fit(
          ui: 'kanso-light',
          preview: 'kanso-dark',
        );
        expect(
          fitted.syntax.firstWhere((style) => style.token == 'keyword').color,
          syntaxOnly.syntax
              .firstWhere((style) => style.token == 'keyword')
              .color,
        );
        expect(fitted.palette.surface, previewOnly.palette.surface);
        await expectLater(
          themes.fit(ui: 'no-such-theme'),
          throwsA(isA<KansoError_NotFound>()),
        );
      },
    );

    test('styles source runs with the fitted syntax theme', () async {
      final themes = (await open()).themes();
      const text = '# Title\n\nSome `code` here.\n';
      final runs = await themes.styledRuns(
        text: text,
        start: 0,
        end: text.length,
        ui: 'kanso-light',
        syntax: 'nord',
      );
      expect(runs, isNotEmpty);
      for (final run in runs) {
        expect(run.range.start, lessThan(run.range.end));
        expect(run.range.end, lessThanOrEqualTo(text.length));
        expect(
          run.color != null || run.italic != null || run.weight != null,
          isTrue,
        );
      }
    });

    test('imports a theme and finds it in the list', () async {
      final themes = (await open()).themes();
      final folder = await Directory.systemTemp.createTemp('kanso_theme_');
      addTearDown(() => folder.delete(recursive: true));
      final path = '${folder.path}/suite-import.json';
      await File(path).writeAsString(_themeJson);

      final imported = await themes.import_(path: path);

      expect(imported.ids, <String>['suite-import']);
      expect(imported.names, <String>['Suite Import']);
      expect(
        (await themes.list()).any((theme) => theme.id == 'suite-import'),
        isTrue,
      );
    });

    test('lists icon themes and reports an unreadable import', () async {
      final themes = (await open()).themes();
      expect(await themes.iconThemes(), isNotEmpty);
      final folder = await Directory.systemTemp.createTemp('kanso_icons_');
      addTearDown(() => folder.delete(recursive: true));
      await expectLater(
        themes.importIconTheme(path: '${folder.path}/missing.zip'),
        throwsA(anyOf(isA<KansoError_Invalid>(), isA<KansoError_NotFound>())),
      );
    });
  });
}

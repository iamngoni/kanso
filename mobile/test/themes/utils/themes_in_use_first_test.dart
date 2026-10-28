// Kanso — themes_in_use_first_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados2, IntAnys, ListAnys, any;
import 'package:kanso/core/models/data/theme_selection.dart';
import 'package:kanso/themes/models/data/theme_preview.dart';
import 'package:kanso/themes/utils/themes_in_use_first.dart';

import '../../helpers/property.dart';
import '../../helpers/theme_tokens.dart';

ThemePreview _theme(String id) {
  const color = Color(0xFF123456);
  return ThemePreview(
    id: id,
    name: id,
    dark: false,
    editor: color,
    text: color,
    accent: color,
    heading: color,
    marker: color,
    quote: color,
    link: color,
    url: color,
    code: color,
    strong: color,
    emphasis: color,
    palette: fakePalette(id),
  );
}

void main() {
  group('themesInUseFirst', () {
    test('puts the theme in use first and keeps the order of the rest', () {
      final themes = <ThemePreview>[
        for (final id in <String>[
          'kanso-light',
          'kanso-dark',
          'nord',
          'dracula',
        ])
          _theme(id),
      ];
      final ordered = themesInUseFirst(
        themes,
        const ThemeSelection(followSystem: false, ui: 'nord'),
      );
      expect(ordered.map((theme) => theme.id), <String>[
        'nord',
        'kanso-light',
        'kanso-dark',
        'dracula',
      ]);
    });

    test('puts both themes in use first while the system is followed', () {
      final themes = <ThemePreview>[
        for (final id in <String>['a', 'kanso-dark', 'b', 'kanso-light', 'c'])
          _theme(id),
      ];
      final ordered = themesInUseFirst(themes, const ThemeSelection());
      expect(ordered.map((theme) => theme.id), <String>[
        'kanso-dark',
        'kanso-light',
        'a',
        'b',
        'c',
      ]);
    });

    Glados2<List<int>, ThemeSelection>(
      any.list(any.intInRange(0, 6)),
      any.themeSelection,
      kansoExploreConfig(),
    ).test('is the same themes, those in use first, each group in order', (
      numbers,
      selection,
    ) {
      final ids = <String>{
        for (final n in numbers)
          <String>['kanso-light', 'kanso-dark', 'nord', 'x', 'y', 'z'][n],
      }.toList();
      final themes = ids.map(_theme).toList();
      final ordered = themesInUseFirst(themes, selection);
      expect(ordered.map((t) => t.id).toList()..sort(), [...ids]..sort());
      final inUse = ordered.where((t) => selection.isSelected(t.id)).length;
      expect(
        ordered.take(inUse).every((t) => selection.isSelected(t.id)),
        isTrue,
      );
      expect(
        ordered.skip(inUse).any((t) => selection.isSelected(t.id)),
        isFalse,
      );
      List<String> order(Iterable<ThemePreview> of) =>
          of.map((t) => t.id).toList();
      expect(
        order(ordered.take(inUse)),
        order(themes.where((t) => selection.isSelected(t.id))),
      );
      expect(
        order(ordered.skip(inUse)),
        order(themes.where((t) => !selection.isSelected(t.id))),
      );
    });
  });
}

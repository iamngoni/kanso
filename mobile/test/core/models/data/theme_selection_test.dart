// Kanso — theme_selection_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show BoolAny, Glados, Glados2, Glados3, any;
import 'package:kanso/core/models/data/theme_selection.dart';

import '../../../helpers/property.dart';

void main() {
  group('ThemeSelection', () {
    test('starts as the desktop does: following the system, Kanso themes', () {
      const selection = ThemeSelection();
      expect(selection.followSystem, isTrue);
      expect(selection.light, 'kanso-light');
      expect(selection.dark, 'kanso-dark');
      expect(selection.ui, 'kanso-dark');
    });

    test('picking while following fills the slot of its appearance', () {
      const selection = ThemeSelection();
      final light = selection.picked('github-light', dark: false);
      expect(light.light, 'github-light');
      expect(light.dark, 'kanso-dark');
      expect(light.ui, 'github-light');
      final dark = selection.picked('nord', dark: true);
      expect(dark.dark, 'nord');
      expect(dark.light, 'kanso-light');
      expect(dark.ui, 'nord');
    });

    test('picking while not following sets the fixed theme only', () {
      const selection = ThemeSelection(followSystem: false);
      final picked = selection.picked('nord', dark: true);
      expect(picked, selection.copyWith(ui: 'nord'));
    });

    test('lists the themes to draw', () {
      expect(const ThemeSelection().ids, <String>['kanso-light', 'kanso-dark']);
      expect(
        const ThemeSelection(followSystem: false, ui: 'nord').ids,
        <String>['nord'],
      );
      expect(
        const ThemeSelection(light: 'nord', dark: 'nord').ids,
        <String>['nord'],
      );
    });

    Glados3<ThemeSelection, String, bool>(
      any.themeSelection,
      any.themeId,
      any.bool,
      kansoExploreConfig(),
    ).test('picking a theme selects it and fills the right slot', (
      selection,
      id,
      dark,
    ) {
      final next = selection.picked(id, dark: dark);
      expect(next.isSelected(id), isTrue);
      expect(next.followSystem, selection.followSystem);
      expect(next.ui, id);
      if (selection.followSystem) {
        expect(dark ? next.dark : next.light, id);
        // The other slot is untouched.
        expect(
          dark ? next.light : next.dark,
          dark ? selection.light : selection.dark,
        );
      } else {
        expect(next.light, selection.light);
        expect(next.dark, selection.dark);
      }
    });

    Glados3<ThemeSelection, String, bool>(
      any.themeSelection,
      any.themeId,
      any.bool,
      kansoExploreConfig(),
    ).test('picking twice is picking once', (selection, id, dark) {
      final once = selection.picked(id, dark: dark);
      expect(once.picked(id, dark: dark), once);
    });

    Glados2<ThemeSelection, bool>(
      any.themeSelection,
      any.bool,
      kansoExploreConfig(),
    ).test('uiFor follows the system only while it is followed', (
      selection,
      systemDark,
    ) {
      final shown = selection.uiFor(systemDark: systemDark);
      if (selection.followSystem) {
        expect(shown, systemDark ? selection.dark : selection.light);
      } else {
        expect(shown, selection.ui);
        expect(selection.uiFor(systemDark: !systemDark), shown);
      }
      expect(selection.ids, contains(shown));
    });

    Glados<ThemeSelection>(any.themeSelection, kansoExploreConfig()).test(
      'survives JSON',
      (selection) {
        expect(ThemeSelection.fromJson(selection.toJson()), selection);
      },
    );

    test('without a theme puts Kanso’s own in each slot that held it', () {
      const selection = ThemeSelection(
        followSystem: false,
        ui: 'dusk',
        light: 'dusk',
        dark: 'nord',
      );
      expect(
        selection.without('dusk'),
        const ThemeSelection(followSystem: false, dark: 'nord'),
      );
      expect(selection.without('other'), selection);
    });

    Glados2<ThemeSelection, String>(
      any.themeSelection,
      any.themeId,
      kansoExploreConfig(),
    ).test('without a theme leaves nothing that names it, unless it is '
        'Kanso’s own', (selection, id) {
      final without = selection.without(id);
      final names = <String>[without.ui, without.light, without.dark];
      if (id == ThemeSelection.kansoLight || id == ThemeSelection.kansoDark) {
        return;
      }
      expect(names, isNot(contains(id)));
    });

    Glados2<ThemeSelection, String>(
      any.themeSelection,
      any.themeId,
      kansoExploreConfig(),
    ).test('without a theme keeps the slots that did not hold it', (
      selection,
      id,
    ) {
      final without = selection.without(id);
      expect(without.followSystem, selection.followSystem);
      for (final (before, after) in <(String, String)>[
        (selection.ui, without.ui),
        (selection.light, without.light),
        (selection.dark, without.dark),
      ]) {
        if (before != id) {
          expect(after, before);
        }
      }
      expect(without.without(id), without);
    });

    test('reads a file that lacks keys as defaults', () {
      expect(
        ThemeSelection.fromJson(<String, dynamic>{'light': 'nord'}),
        const ThemeSelection(light: 'nord'),
      );
    });
  });
}

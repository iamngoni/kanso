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

    group('the syntax and preview themes', () {
      test('are the interface theme’s own until chosen', () {
        const selection = ThemeSelection();
        expect(selection.syntax, isNull);
        expect(selection.preview, isNull);
        expect(selection.syntaxFor(systemDark: false), 'kanso-light');
        expect(selection.syntaxFor(systemDark: true), 'kanso-dark');
        expect(selection.previewFor(systemDark: false), 'kanso-light');
        expect(selection.previewFor(systemDark: true), 'kanso-dark');
      });

      test('are the same in both modes once chosen', () {
        const selection = ThemeSelection(syntax: 'nord', preview: 'dracula');
        expect(selection.syntaxFor(systemDark: false), 'nord');
        expect(selection.syntaxFor(systemDark: true), 'nord');
        expect(selection.previewFor(systemDark: false), 'dracula');
        expect(selection.previewFor(systemDark: true), 'dracula');
      });

      test('follow the fixed interface theme while the system is not '
          'followed', () {
        const selection = ThemeSelection(followSystem: false, ui: 'nord');
        expect(selection.syntaxFor(systemDark: false), 'nord');
        expect(selection.previewFor(systemDark: true), 'nord');
      });

      test('choosing none again goes back to the interface theme', () {
        const chosen = ThemeSelection(syntax: 'nord', preview: 'dracula');
        final cleared = chosen.copyWith(syntax: null, preview: null);
        expect(cleared, const ThemeSelection());
      });

      test('picking an interface theme leaves them alone', () {
        const selection = ThemeSelection(syntax: 'nord', preview: 'dracula');
        final picked = selection.picked('github-light', dark: false);
        expect(picked.syntax, 'nord');
        expect(picked.preview, 'dracula');
      });

      test('are read from the saved file when it has them', () {
        final read = ThemeSelection.fromJson(<String, dynamic>{
          'syntax': 'nord',
          'preview': null,
        });
        expect(read, const ThemeSelection(syntax: 'nord'));
      });

      test('are none when an old saved file lacks them', () {
        // What a version without separate syntax and preview themes wrote.
        final read = ThemeSelection.fromJson(<String, dynamic>{
          'followSystem': false,
          'ui': 'nord',
          'light': 'github-light',
          'dark': 'nord',
        });
        expect(
          read,
          const ThemeSelection(
            followSystem: false,
            ui: 'nord',
            light: 'github-light',
            dark: 'nord',
          ),
        );
        expect(read.syntax, isNull);
        expect(read.preview, isNull);
      });

      Glados2<ThemeSelection, bool>(
        any.themeSelection,
        any.bool,
        kansoExploreConfig(),
      ).test('syntaxFor and previewFor are the chosen theme or the '
          'interface’s', (selection, systemDark) {
        final ui = selection.uiFor(systemDark: systemDark);
        expect(
          selection.syntaxFor(systemDark: systemDark),
          selection.syntax ?? ui,
        );
        expect(
          selection.previewFor(systemDark: systemDark),
          selection.preview ?? ui,
        );
      });

      Glados<ThemeSelection>(any.themeSelection, kansoExploreConfig()).test(
        'allIds holds every theme to read, once, and the ones drawn',
        (selection) {
          final all = selection.allIds;
          expect(all, containsAll(selection.ids));
          for (final dark in <bool>[false, true]) {
            expect(all, contains(selection.syntaxFor(systemDark: dark)));
            expect(all, contains(selection.previewFor(systemDark: dark)));
          }
          final syntax = selection.syntax;
          if (syntax != null) {
            expect(all, contains(syntax));
          }
        },
      );

      Glados2<ThemeSelection, String>(
        any.themeSelection,
        any.themeId,
        kansoExploreConfig(),
      ).test('choosing a syntax theme changes only that', (selection, id) {
        final next = selection.copyWith(syntax: id);
        expect(next.preview, selection.preview);
        expect(next.ui, selection.ui);
        expect(next.followSystem, selection.followSystem);
        expect(next.syntaxFor(systemDark: true), id);
        expect(next.syntaxFor(systemDark: false), id);
      });
    });
  });
}

// Kanso — shortcut_bindings_test.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/shortcuts.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/shell/utils/kanso_shortcut_bindings.dart';
import 'package:kanso/shell/utils/shortcut_intents.dart';
import 'package:kanso/shell/utils/shortcut_keys.dart';

import '../../helpers/core_library.dart';

void main() {
  setUpAll(initCoreForTests);

  test('every mobile catalog entry is bound, intentional, or documented', () {
    final catalog = shortcutCatalog();
    expect(catalog, isNotEmpty);
    // The bridge returns shortcuts::on_mobile(), already filtered by on_mobile.
    expect(
      catalog.any((entry) => entry.place == ShortcutPlace.capture),
      isFalse,
    );

    final bound = shortcutBindingsFor(
      const <ShortcutContext>[ShortcutContext.global, ShortcutContext.noteList],
      TargetPlatform.iOS,
    );
    for (final entry in catalog) {
      final intent = intentForShortcut(entry);
      if (unboundOnMobile.contains(entry.id)) {
        expect(intent, isNull, reason: '${entry.id} is explicitly unbound');
        expect(
          bound.values.contains(intent),
          isFalse,
          reason: '${entry.id} cannot be both bound and unbound',
        );
        continue;
      }
      expect(intent, isNotNull, reason: '${entry.id} needs an explicit action');
      if (entry.place == ShortcutPlace.global ||
          entry.place == ShortcutPlace.noteList) {
        final keys = shortcutKeysFor(entry, TargetPlatform.iOS);
        final expected = activatorFor(keys, TargetPlatform.iOS);
        if (expected != null) {
          expect(
            bound.keys.any(
              (activator) =>
                  activator is SingleActivator &&
                  activator.trigger == expected.trigger &&
                  activator.control == expected.control &&
                  activator.alt == expected.alt &&
                  activator.shift == expected.shift &&
                  activator.meta == expected.meta,
            ),
            isTrue,
            reason: '${entry.id} ($keys) must be in the global/list map',
          );
        }
      }
    }
  });

  test('secondary N resolves to the platform modifier', () {
    final entry = shortcutCatalog().firstWhere((item) => item.id == 'new_note');
    for (final (platform, isApple) in <(TargetPlatform, bool)>[
      (TargetPlatform.iOS, true),
      (TargetPlatform.android, false),
    ]) {
      final activator = activatorFor(
        shortcutKeysFor(entry, platform),
        platform,
      )!;
      expect(activator.trigger, LogicalKeyboardKey.keyN);
      expect(activator.meta, isApple);
      expect(activator.control, !isApple);
    }
  });

  test('ctrl-tab stays Control+Tab on Apple and Android', () {
    for (final platform in <TargetPlatform>[
      TargetPlatform.iOS,
      TargetPlatform.android,
    ]) {
      final activator = activatorFor('ctrl-tab', platform)!;
      expect(activator.trigger, LogicalKeyboardKey.tab);
      expect(activator.control, isTrue);
      expect(activator.meta, isFalse);
      final bound = shortcutBindingsFor(
        const <ShortcutContext>[ShortcutContext.global],
        platform,
      );
      expect(
        bound.keys.any(
          (key) =>
              key is SingleActivator &&
              key.trigger == LogicalKeyboardKey.tab &&
              key.control,
        ),
        isTrue,
      );
    }
  });
}

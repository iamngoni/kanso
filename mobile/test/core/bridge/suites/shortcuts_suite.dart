// Kanso — shortcuts_suite.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/foundation.dart' show TargetPlatform;
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/shortcuts.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/shell/utils/shortcut_keys.dart';

import 'open_library.dart';

/// The desktop's keyboard shortcuts, through the bridge: `shortcutCatalog`,
/// and the keys of every entry as a phone reads them.
void shortcutsSuite(OpenLibrary open) {
  group('shortcutCatalog', () {
    late List<ShortcutEntry> catalog;

    setUpAll(() async {
      // The catalog is static data, but a phone asks for it with a library
      // open, so this reads it with one.
      await open();
      catalog = shortcutCatalog();
    });

    test('is the desktop menu, less what a phone cannot do', () {
      expect(catalog, isNotEmpty);
      // Quit is the system's on a phone, and Quick Capture is a window.
      expect(catalog.where((entry) => entry.id == 'quit'), isEmpty);
      expect(
        catalog.where((entry) => entry.place == ShortcutPlace.capture),
        isEmpty,
      );
      // The formatting keys are the accessory bar's plus the headings.
      expect(
        catalog.where((entry) => entry.section == ShortcutSection.format),
        hasLength(16),
      );
    });

    test('every entry has keys on both systems, save the text keys', () {
      for (final entry in catalog) {
        final mac = shortcutKeysFor(entry, TargetPlatform.iOS);
        expect(
          activatorFor(mac, TargetPlatform.iOS),
          isNotNull,
          reason: '${entry.id} $mac',
        );
        expect(keyLabel(mac, TargetPlatform.iOS), isNotEmpty, reason: entry.id);
        final other = shortcutKeysFor(entry, TargetPlatform.android);
        if (other.isEmpty) {
          // Only the desktop's macOS-only text bindings have none.
          expect(entry.place, ShortcutPlace.editor, reason: entry.id);
          expect(entry.id, anyOf('go_back', 'go_forward'));
          continue;
        }
        expect(
          activatorFor(other, TargetPlatform.android),
          isNotNull,
          reason: '${entry.id} $other',
        );
        expect(
          keyLabel(other, TargetPlatform.android),
          isNotEmpty,
          reason: entry.id,
        );
      }
    });

    test('no two entries share a keystroke in one place', () {
      for (final platform in <TargetPlatform>[
        TargetPlatform.iOS,
        TargetPlatform.android,
      ]) {
        final seen = <(ShortcutPlace, String)>{};
        for (final entry in catalog) {
          final keys = shortcutKeysFor(entry, platform);
          if (keys.isEmpty) {
            continue;
          }
          final canonical = keystrokeOf(activatorFor(keys, platform)!);
          expect(
            seen.add((entry.place, canonical!)),
            isTrue,
            reason: '${entry.id} $keys on $platform',
          );
        }
      }
    });
  });
}

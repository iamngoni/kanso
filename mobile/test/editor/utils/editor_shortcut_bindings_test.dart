// Kanso — editor_shortcut_bindings_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart' show FormatKind;
import 'package:kanso/editor/models/data/editor_intent.dart';
import 'package:kanso/editor/utils/editor_shortcut_bindings.dart';

SingleActivator _activator(ShortcutActivator activator) =>
    activator as SingleActivator;

void main() {
  group('editorShortcutBindings', () {
    test('has the keys of the table, on every platform', () {
      for (final platform in TargetPlatform.values) {
        final apple =
            platform == TargetPlatform.iOS || platform == TargetPlatform.macOS;
        final bindings = <ShortcutActivator, Intent>{
          ...editorShortcutBindings(platform),
        };
        SingleActivator find(LogicalKeyboardKey key, {bool shift = false}) {
          return bindings.keys
              .map(_activator)
              .singleWhere((a) => a.trigger == key && a.shift == shift);
        }

        final bold = find(LogicalKeyboardKey.keyB);
        expect(bold.meta, apple, reason: '$platform');
        expect(bold.control, !apple, reason: '$platform');
        expect(
          (bindings[bold]! as FormatIntent).format,
          FormatKind.bold,
        );
        expect(
          (bindings[find(LogicalKeyboardKey.keyI)]! as FormatIntent).format,
          FormatKind.italic,
        );
        expect(bindings[find(LogicalKeyboardKey.keyS)], isA<SaveIntent>());
        // The keys that are not Command or Control's.
        final tab = find(LogicalKeyboardKey.tab);
        expect(tab.meta || tab.control, isFalse);
        expect((bindings[tab]! as TabIntent).out, isFalse);
        expect(
          (bindings[find(LogicalKeyboardKey.tab, shift: true)]! as TabIntent)
              .out,
          isTrue,
        );
        expect(
          bindings[find(LogicalKeyboardKey.escape)],
          isA<EscapeIntent>(),
        );
        expect(
          (bindings[find(LogicalKeyboardKey.arrowDown)]!
                  as CompletionStepIntent)
              .delta,
          1,
        );
        expect(
          (bindings[find(LogicalKeyboardKey.arrowUp)]! as CompletionStepIntent)
              .delta,
          -1,
        );
        expect(
          bindings[find(LogicalKeyboardKey.enter)],
          isA<CompletionChooseIntent>(),
        );
      }
    });

    test('leaves undo and redo to the platform', () {
      final keys = editorShortcutBindings(
        TargetPlatform.iOS,
      ).keys.map(_activator).map((a) => a.trigger);
      expect(keys, isNot(contains(LogicalKeyboardKey.keyZ)));
    });
  });
}

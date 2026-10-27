// Kanso — kanso_shortcut_bindings_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/shell/models/data/kanso_intent.dart';
import 'package:kanso/shell/utils/kanso_shortcut_bindings.dart';

SingleActivator _activator(ShortcutActivator activator) =>
    activator as SingleActivator;

void main() {
  group('kansoShortcutBindings', () {
    test('every intent has exactly one shortcut', () {
      for (final platform in TargetPlatform.values) {
        final bindings = kansoShortcutBindings(platform);
        expect(bindings.values.map((i) => i.runtimeType).toSet(), <Type>{
          BackIntent,
          ForwardIntent,
          SearchAllIntent,
          SettingsIntent,
          ToggleLibraryIntent,
          NextNoteIntent,
          PreviousNoteIntent,
          PinNoteIntent,
          NewNoteIntent,
          NewNotebookIntent,
          EditNoteIntent,
          PreviewNoteIntent,
        });
        expect(bindings, hasLength(12));
      }
    });

    test(
      'the secondary key is Command on Apple devices, Control elsewhere',
      () {
        for (final platform in TargetPlatform.values) {
          final apple =
              platform == TargetPlatform.iOS ||
              platform == TargetPlatform.macOS;
          for (final activator in kansoShortcutBindings(platform).keys) {
            final single = _activator(activator);
            expect(single.meta, apple, reason: '$platform $single');
            expect(single.control, !apple, reason: '$platform $single');
          }
        }
      },
    );

    test('the keys are the desktop’s', () {
      final bindings = <Type, SingleActivator>{
        for (final entry in kansoShortcutBindings(TargetPlatform.iOS).entries)
          entry.value.runtimeType: _activator(entry.key),
      };
      expect(bindings[BackIntent]!.trigger, LogicalKeyboardKey.bracketLeft);
      expect(bindings[ForwardIntent]!.trigger, LogicalKeyboardKey.bracketRight);
      expect(bindings[SearchAllIntent]!.trigger, LogicalKeyboardKey.keyF);
      expect(bindings[SearchAllIntent]!.shift, isTrue);
      expect(bindings[SettingsIntent]!.trigger, LogicalKeyboardKey.comma);
      expect(
        bindings[ToggleLibraryIntent]!.trigger,
        LogicalKeyboardKey.backslash,
      );
      expect(bindings[NextNoteIntent]!.trigger, LogicalKeyboardKey.arrowDown);
      expect(bindings[NextNoteIntent]!.alt, isTrue);
      expect(
        bindings[PreviousNoteIntent]!.trigger,
        LogicalKeyboardKey.arrowUp,
      );
      expect(bindings[PreviousNoteIntent]!.alt, isTrue);
      expect(bindings[PinNoteIntent]!.trigger, LogicalKeyboardKey.keyP);
      expect(bindings[PinNoteIntent]!.shift, isTrue);
      expect(bindings[NewNoteIntent]!.trigger, LogicalKeyboardKey.keyN);
      expect(bindings[NewNoteIntent]!.shift, isFalse);
      expect(bindings[NewNotebookIntent]!.trigger, LogicalKeyboardKey.keyN);
      expect(bindings[NewNotebookIntent]!.shift, isTrue);
      expect(bindings[EditNoteIntent]!.trigger, LogicalKeyboardKey.digit1);
      expect(bindings[PreviewNoteIntent]!.trigger, LogicalKeyboardKey.digit3);
    });

    test('there is no shortcut for what is not built yet', () {
      final triggers = kansoShortcutBindings(
        TargetPlatform.iOS,
      ).keys.map((a) => _activator(a).trigger).toSet();
      for (final key in <LogicalKeyboardKey>[
        LogicalKeyboardKey.keyK,
        // The phone has no split view of a note.
        LogicalKeyboardKey.digit2,
      ]) {
        expect(triggers, isNot(contains(key)));
      }
      // F is only ever shifted: find in a note is the editor's.
      final plainF = kansoShortcutBindings(TargetPlatform.iOS).keys
          .map(_activator)
          .where((a) => a.trigger == LogicalKeyboardKey.keyF && !a.shift);
      expect(plainF, isEmpty);
    });
  });
}

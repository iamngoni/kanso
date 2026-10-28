// Kanso — kanso_shortcut_bindings_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/editor/models/data/editor_intent.dart';
import 'package:kanso/shell/models/data/kanso_intent.dart';
import 'package:kanso/shell/utils/kanso_shortcut_bindings.dart';
import 'package:kanso/tabs/models/data/tab_intent.dart';
import 'package:kanso/window/models/data/window_intent.dart';

import '../../helpers/core_library.dart';

SingleActivator _activator(ShortcutActivator activator) =>
    activator as SingleActivator;

void main() {
  setUpAll(initCoreForTests);

  group('kansoShortcutBindings', () {
    test('every bound global/list intent has a catalog shortcut', () {
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
          SaveIntent,
          OpenPaletteIntent,
          OpenTodayIntent,
          ReplaceInNotesIntent,
          ToggleFocusModeIntent,
          ToggleInspectorIntent,
          OpenInNewWindowIntent,
          CloseTabIntent,
          NextTabIntent,
          PreviousTabIntent,
          ReopenClosedTabIntent,
          GoToTabIntent,
        });
        expect(bindings, hasLength(32));
      }
    });

    test('secondary is Command on Apple and Control elsewhere', () {
      for (final platform in TargetPlatform.values) {
        final apple =
            platform == TargetPlatform.iOS || platform == TargetPlatform.macOS;
        for (final activator in kansoShortcutBindings(platform).keys) {
          final single = _activator(activator);
          if (single.trigger == LogicalKeyboardKey.tab) {
            expect(single.control, isTrue, reason: '$platform $single');
            expect(single.meta, isFalse, reason: '$platform $single');
          } else if (single.alt &&
              (single.trigger == LogicalKeyboardKey.arrowLeft ||
                  single.trigger == LogicalKeyboardKey.arrowRight)) {
            expect(single.control, isFalse, reason: '$platform $single');
            expect(single.meta, isFalse, reason: '$platform $single');
          } else {
            expect(single.meta, apple, reason: '$platform $single');
            expect(single.control, !apple, reason: '$platform $single');
          }
        }
      }
    });

    test('the keys still present from the original shell map are exact', () {
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
      expect(bindings[PreviousNoteIntent]!.trigger, LogicalKeyboardKey.arrowUp);
      expect(bindings[PreviousNoteIntent]!.alt, isTrue);
      expect(bindings[PinNoteIntent]!.trigger, LogicalKeyboardKey.keyP);
      expect(bindings[PinNoteIntent]!.shift, isTrue);
      expect(bindings[NewNoteIntent]!.trigger, LogicalKeyboardKey.keyN);
      expect(bindings[NewNotebookIntent]!.trigger, LogicalKeyboardKey.keyN);
      expect(bindings[NewNotebookIntent]!.shift, isTrue);
      expect(bindings[EditNoteIntent]!.trigger, LogicalKeyboardKey.digit1);
      expect(bindings[PreviewNoteIntent]!.trigger, LogicalKeyboardKey.digit3);
    });

    test('Escape leaves focus mode only while it is on', () {
      final focus = shortcutBindingsFor(
        const <ShortcutContext>[ShortcutContext.focusMode],
        TargetPlatform.iOS,
      );
      final escape = focus.entries.singleWhere(
        (entry) => entry.value is ExitFocusModeIntent,
      );
      expect(_activator(escape.key).trigger, LogicalKeyboardKey.escape);
      expect(
        kansoShortcutBindings(TargetPlatform.iOS).values,
        isNot(contains(isA<ExitFocusModeIntent>())),
      );
    });

    test('the command palette is bound to its catalog shortcut', () {
      final palette = kansoShortcutBindings(TargetPlatform.iOS).entries
          .where((entry) => entry.value is OpenPaletteIntent)
          .map((entry) => entry.key)
          .toList();
      expect(palette, hasLength(1));
    });

    test('there is no shortcut for unsupported list focus or page zoom', () {
      final plainF = kansoShortcutBindings(TargetPlatform.iOS).keys
          .map(_activator)
          .where(
            (activator) =>
                activator.trigger == LogicalKeyboardKey.keyF &&
                !activator.shift,
          );
      expect(plainF, isEmpty);
    });
  });
}

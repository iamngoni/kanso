// Kanso — editor_shortcut_bindings.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../core/bridge/api/types.dart' show FormatKind;
import '../models/data/editor_intent.dart';

/// The keyboard shortcuts of the note's editor on [platform]: "secondary" is
/// Command on Apple devices and Control elsewhere.
///
/// | Shortcut | Does |
/// | --- | --- |
/// | secondary + B, I | bold, italic |
/// | secondary + S | save now |
/// | Tab, Shift + Tab | indent, outdent (or the next and previous cell) |
/// | Escape | close the completions, else go to Preview |
/// | Down, Up, Return | move through and choose in the completions |
///
/// Undo and redo (secondary + Z, with Shift) are the platform's own
/// shortcuts, and the editor answers them with the Rust core's history.
/// The shortcuts for the completions and Escape only take the key while
/// there is something for them to do.
Map<ShortcutActivator, Intent> editorShortcutBindings(TargetPlatform platform) {
  final apple =
      platform == TargetPlatform.iOS || platform == TargetPlatform.macOS;
  SingleActivator secondary(LogicalKeyboardKey key) =>
      SingleActivator(key, meta: apple, control: !apple);
  return <ShortcutActivator, Intent>{
    secondary(LogicalKeyboardKey.keyB): const FormatIntent(FormatKind.bold),
    secondary(LogicalKeyboardKey.keyI): const FormatIntent(FormatKind.italic),
    secondary(LogicalKeyboardKey.keyS): const SaveIntent(),
    const SingleActivator(LogicalKeyboardKey.tab): const TabIntent(out: false),
    const SingleActivator(LogicalKeyboardKey.tab, shift: true): const TabIntent(
      out: true,
    ),
    const SingleActivator(LogicalKeyboardKey.escape): const EscapeIntent(),
    const SingleActivator(LogicalKeyboardKey.arrowDown):
        const CompletionStepIntent(1),
    const SingleActivator(LogicalKeyboardKey.arrowUp):
        const CompletionStepIntent(-1),
    const SingleActivator(LogicalKeyboardKey.enter):
        const CompletionChooseIntent(),
  };
}

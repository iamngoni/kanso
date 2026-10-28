// Kanso — kanso_shortcut_bindings.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../core/bridge/api/shortcuts.dart';
import '../../core/bridge/api/types.dart' show ShortcutEntry, ShortcutPlace;
import 'shortcut_intents.dart';
import 'shortcut_keys.dart';

/// Dart name for the contexts carried by the bridge's [ShortcutEntry.place].
typedef ShortcutContext = ShortcutPlace;

/// The mobile bindings for the given contexts, read from the desktop catalog.
Map<ShortcutActivator, Intent> shortcutBindingsFor(
  Iterable<ShortcutContext> contexts,
  TargetPlatform platform,
) {
  final wanted = contexts.toSet();
  final bindings = <ShortcutActivator, Intent>{};
  final activators =
      <
        ({
          LogicalKeyboardKey key,
          bool control,
          bool alt,
          bool shift,
          bool meta,
        })
      >{};
  for (final ShortcutEntry entry in shortcutCatalog()) {
    if (!wanted.contains(entry.place)) {
      continue;
    }
    final keys = shortcutKeysFor(entry, platform);
    final activator = activatorFor(keys, platform);
    final intent = intentForShortcut(entry);
    if (activator == null || intent == null) {
      continue;
    }
    final identity = (
      key: activator.trigger,
      control: activator.control,
      alt: activator.alt,
      shift: activator.shift,
      meta: activator.meta,
    );
    // When catalog contexts reuse a keystroke, the first context owns it.
    if (!activators.add(identity)) {
      continue;
    }
    bindings[activator] = intent;
  }
  return bindings;
}

/// The global and note-list bindings for [platform]. Editor keys stay with
/// the editor's existing shortcut and intent plumbing.
Map<ShortcutActivator, Intent> kansoShortcutBindings(TargetPlatform platform) =>
    shortcutBindingsFor(
      const <ShortcutContext>[
        ShortcutContext.global,
        ShortcutContext.noteList,
      ],
      platform,
    );

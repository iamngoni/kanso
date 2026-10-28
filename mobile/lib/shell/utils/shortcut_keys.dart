// Kanso — shortcut_keys.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../core/bridge/api/types.dart' show ShortcutEntry;

/// The names of the keys the desktop's shortcut catalog uses, as the desktop
/// writes them (GPUI's keystroke syntax: `secondary-shift-t`, `ctrl-tab`,
/// `cmd-[`, `escape`), and the Flutter keys they are.
final Map<String, LogicalKeyboardKey> shortcutKeyNames =
    <String, LogicalKeyboardKey>{
      for (var i = 0; i < 26; i++) String.fromCharCode(0x61 + i): _letters[i],
      for (var i = 0; i < 10; i++) '$i': _digits[i],
      '[': LogicalKeyboardKey.bracketLeft,
      ']': LogicalKeyboardKey.bracketRight,
      r'\': LogicalKeyboardKey.backslash,
      ',': LogicalKeyboardKey.comma,
      '.': LogicalKeyboardKey.period,
      '=': LogicalKeyboardKey.equal,
      '+': LogicalKeyboardKey.add,
      '-': LogicalKeyboardKey.minus,
      'tab': LogicalKeyboardKey.tab,
      'enter': LogicalKeyboardKey.enter,
      'escape': LogicalKeyboardKey.escape,
      'space': LogicalKeyboardKey.space,
      'backspace': LogicalKeyboardKey.backspace,
      'left': LogicalKeyboardKey.arrowLeft,
      'right': LogicalKeyboardKey.arrowRight,
      'up': LogicalKeyboardKey.arrowUp,
      'down': LogicalKeyboardKey.arrowDown,
      'pageup': LogicalKeyboardKey.pageUp,
      'pagedown': LogicalKeyboardKey.pageDown,
    };

const List<LogicalKeyboardKey> _letters = <LogicalKeyboardKey>[
  LogicalKeyboardKey.keyA,
  LogicalKeyboardKey.keyB,
  LogicalKeyboardKey.keyC,
  LogicalKeyboardKey.keyD,
  LogicalKeyboardKey.keyE,
  LogicalKeyboardKey.keyF,
  LogicalKeyboardKey.keyG,
  LogicalKeyboardKey.keyH,
  LogicalKeyboardKey.keyI,
  LogicalKeyboardKey.keyJ,
  LogicalKeyboardKey.keyK,
  LogicalKeyboardKey.keyL,
  LogicalKeyboardKey.keyM,
  LogicalKeyboardKey.keyN,
  LogicalKeyboardKey.keyO,
  LogicalKeyboardKey.keyP,
  LogicalKeyboardKey.keyQ,
  LogicalKeyboardKey.keyR,
  LogicalKeyboardKey.keyS,
  LogicalKeyboardKey.keyT,
  LogicalKeyboardKey.keyU,
  LogicalKeyboardKey.keyV,
  LogicalKeyboardKey.keyW,
  LogicalKeyboardKey.keyX,
  LogicalKeyboardKey.keyY,
  LogicalKeyboardKey.keyZ,
];

const List<LogicalKeyboardKey> _digits = <LogicalKeyboardKey>[
  LogicalKeyboardKey.digit0,
  LogicalKeyboardKey.digit1,
  LogicalKeyboardKey.digit2,
  LogicalKeyboardKey.digit3,
  LogicalKeyboardKey.digit4,
  LogicalKeyboardKey.digit5,
  LogicalKeyboardKey.digit6,
  LogicalKeyboardKey.digit7,
  LogicalKeyboardKey.digit8,
  LogicalKeyboardKey.digit9,
];

/// The modifiers a keystroke may begin with. `secondary` is Command on
/// Apple devices and Control elsewhere; `cmd` is always Command.
const Set<String> _modifierNames = <String>{
  'secondary',
  'cmd',
  'ctrl',
  'alt',
  'shift',
};

/// Whether [platform] is an Apple one, where `secondary` is Command and the
/// catalog's `mac` keys are used.
bool isApplePlatform(TargetPlatform platform) =>
    platform == TargetPlatform.iOS || platform == TargetPlatform.macOS;

/// The keys of [entry] on [platform]: the desktop's macOS keys on an iPad,
/// its Windows and Linux keys on Android. Empty when the desktop binds
/// nothing there.
String shortcutKeysFor(ShortcutEntry entry, TargetPlatform platform) =>
    isApplePlatform(platform) ? entry.mac : entry.other;

/// A keystroke split into its modifiers and its key, or null when it isn't
/// one the catalog writes.
({Set<String> modifiers, String key})? _split(String keystroke) {
  final modifiers = <String>{};
  var rest = keystroke;
  while (true) {
    final dash = rest.indexOf('-');
    // A modifier is a whole word before a dash that has a key after it, so
    // the minus key itself (`secondary--`) is the key, not a modifier.
    if (dash <= 0 || dash == rest.length - 1) {
      break;
    }
    final word = rest.substring(0, dash);
    if (!_modifierNames.contains(word)) {
      return null;
    }
    if (!modifiers.add(word)) {
      return null;
    }
    rest = rest.substring(dash + 1);
  }
  if (rest.isEmpty || _modifierNames.contains(rest)) {
    return null;
  }
  return (modifiers: modifiers, key: rest);
}

/// The Flutter shortcut for the desktop keystroke [keys] on [platform], or
/// null for an empty string, a key the catalog doesn't use, or a sequence of
/// keystrokes (the desktop binds none).
SingleActivator? activatorFor(String keys, TargetPlatform platform) {
  final parts = _split(keys);
  if (parts == null) {
    return null;
  }
  final key = shortcutKeyNames[parts.key];
  if (key == null) {
    return null;
  }
  final apple = isApplePlatform(platform);
  final modifiers = parts.modifiers;
  return SingleActivator(
    key,
    meta:
        modifiers.contains('cmd') || (apple && modifiers.contains('secondary')),
    control:
        modifiers.contains('ctrl') ||
        (!apple && modifiers.contains('secondary')),
    alt: modifiers.contains('alt'),
    shift: modifiers.contains('shift'),
  );
}

/// The desktop keystroke for [activator]: the inverse of [activatorFor] for
/// an activator with keys the catalog uses (it writes `cmd` and `ctrl`,
/// never `secondary`). Null for a key the catalog doesn't use.
String? keystrokeOf(SingleActivator activator) {
  String? name;
  for (final entry in shortcutKeyNames.entries) {
    if (entry.value == activator.trigger) {
      name = entry.key;
      break;
    }
  }
  if (name == null) {
    return null;
  }
  return <String>[
    if (activator.control) 'ctrl',
    if (activator.alt) 'alt',
    if (activator.shift) 'shift',
    if (activator.meta) 'cmd',
    name,
  ].join('-');
}

const Map<String, String> _appleGlyphs = <String, String>{
  'tab': '⇥',
  'enter': '↩',
  'escape': '⎋',
  'space': '␣',
  'backspace': '⌫',
  'left': '←',
  'right': '→',
  'up': '↑',
  'down': '↓',
  'pageup': '⇞',
  'pagedown': '⇟',
};

const Map<String, String> _otherNames = <String, String>{
  'tab': 'Tab',
  'enter': 'Enter',
  'escape': 'Esc',
  'space': 'Space',
  'backspace': 'Backspace',
  'left': 'Left',
  'right': 'Right',
  'up': 'Up',
  'down': 'Down',
  'pageup': 'PgUp',
  'pagedown': 'PgDn',
};

/// How the shortcut overlay writes the desktop keystroke [keys] on
/// [platform]: `⌘⇧T` on Apple devices, with the modifiers in Apple's order
/// (⌃⌥⇧⌘), and `Ctrl+Shift+T` elsewhere. Empty for keys that aren't a
/// keystroke the catalog writes.
String keyLabel(String keys, TargetPlatform platform) {
  final parts = _split(keys);
  if (parts == null || !shortcutKeyNames.containsKey(parts.key)) {
    return '';
  }
  final apple = isApplePlatform(platform);
  final modifiers = parts.modifiers;
  final command =
      modifiers.contains('cmd') || (apple && modifiers.contains('secondary'));
  final control =
      modifiers.contains('ctrl') || (!apple && modifiers.contains('secondary'));
  final alt = modifiers.contains('alt');
  final shift = modifiers.contains('shift');
  final key = parts.key;
  if (apple) {
    return <String>[
      if (control) '⌃',
      if (alt) '⌥',
      if (shift) '⇧',
      if (command) '⌘',
      _appleGlyphs[key] ?? key.toUpperCase(),
    ].join();
  }
  return <String>[
    if (control) 'Ctrl',
    if (alt) 'Alt',
    if (shift) 'Shift',
    if (command) 'Meta',
    _otherNames[key] ?? key.toUpperCase(),
  ].join('+');
}

// Kanso — shortcut_keys_test.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart'
    show Any, AnyUtils, BoolAny, CombinableAny, Generator, Glados, Glados2, any;
import 'package:kanso/shell/utils/shortcut_keys.dart';

import '../../helpers/property.dart';

const TargetPlatform _ios = TargetPlatform.iOS;
const TargetPlatform _android = TargetPlatform.android;

/// A `SingleActivator`'s every field, in order. `SingleActivator` has no
/// value equality, so two built from the same keys are different objects;
/// comparing these records compares the keys.
typedef _Shape = (
  LogicalKeyboardKey,
  bool,
  bool,
  bool,
  bool,
  bool,
  LockState,
);

_Shape _shape(SingleActivator activator) => (
  activator.trigger,
  activator.control,
  activator.shift,
  activator.alt,
  activator.meta,
  activator.includeRepeats,
  activator.numLock,
);

/// The shape of `keys` on `platform`, or null when they are not a keystroke.
_Shape? _shapeOf(String keys, TargetPlatform platform) {
  final activator = activatorFor(keys, platform);
  return activator == null ? null : _shape(activator);
}

/// The shape of `expected`, to compare with `_shapeOf`.
_Shape _expected(SingleActivator activator) => _shape(activator);

extension _ActivatorAny on Any {
  /// Any key the catalog uses, with any modifiers.
  Generator<SingleActivator> get activator => combine5(
    choose(shortcutKeyNames.values.toList()),
    this.bool,
    this.bool,
    this.bool,
    this.bool,
    (LogicalKeyboardKey key, bool control, bool alt, bool shift, bool meta) =>
        SingleActivator(
          key,
          control: control,
          alt: alt,
          shift: shift,
          meta: meta,
        ),
  );
}

void main() {
  group('activatorFor', () {
    test('resolves secondary to Command on iOS and Control on Android', () {
      expect(
        _shapeOf('secondary-shift-t', _ios),
        _expected(
          const SingleActivator(
            LogicalKeyboardKey.keyT,
            meta: true,
            shift: true,
          ),
        ),
      );
      expect(
        _shapeOf('secondary-shift-t', _android),
        _expected(
          const SingleActivator(
            LogicalKeyboardKey.keyT,
            control: true,
            shift: true,
          ),
        ),
      );
    });

    test('keeps cmd and ctrl as they are on both', () {
      for (final platform in <TargetPlatform>[_ios, _android]) {
        expect(
          _shapeOf('cmd-[', platform),
          _expected(
            const SingleActivator(LogicalKeyboardKey.bracketLeft, meta: true),
          ),
        );
        expect(
          _shapeOf('ctrl-tab', platform),
          _expected(
            const SingleActivator(LogicalKeyboardKey.tab, control: true),
          ),
        );
        expect(
          _shapeOf('ctrl-shift-tab', platform),
          _expected(
            const SingleActivator(
              LogicalKeyboardKey.tab,
              control: true,
              shift: true,
            ),
          ),
        );
      }
    });

    test('reads the keys that are also separators and modifiers', () {
      expect(
        _shapeOf('secondary--', _ios),
        _expected(const SingleActivator(LogicalKeyboardKey.minus, meta: true)),
      );
      expect(
        _shapeOf('secondary-+', _ios),
        _expected(const SingleActivator(LogicalKeyboardKey.add, meta: true)),
      );
      expect(
        _shapeOf('secondary-=', _android),
        _expected(
          const SingleActivator(LogicalKeyboardKey.equal, control: true),
        ),
      );
      expect(
        _shapeOf(r'secondary-\', _ios),
        _expected(
          const SingleActivator(LogicalKeyboardKey.backslash, meta: true),
        ),
      );
      expect(
        _shapeOf('secondary-shift-.', _ios),
        _expected(
          const SingleActivator(
            LogicalKeyboardKey.period,
            meta: true,
            shift: true,
          ),
        ),
      );
      expect(
        _shapeOf('secondary-alt-1', _ios),
        _expected(
          const SingleActivator(
            LogicalKeyboardKey.digit1,
            meta: true,
            alt: true,
          ),
        ),
      );
    });

    test('reads the named keys', () {
      const named = <String, LogicalKeyboardKey>{
        'escape': LogicalKeyboardKey.escape,
        'enter': LogicalKeyboardKey.enter,
        'space': LogicalKeyboardKey.space,
        'backspace': LogicalKeyboardKey.backspace,
        'left': LogicalKeyboardKey.arrowLeft,
        'pageup': LogicalKeyboardKey.pageUp,
        'pagedown': LogicalKeyboardKey.pageDown,
        'j': LogicalKeyboardKey.keyJ,
      };
      for (final entry in named.entries) {
        expect(
          _shapeOf(entry.key, _ios),
          _expected(SingleActivator(entry.value)),
        );
      }
    });

    test('gives nothing for what the catalog does not write', () {
      for (final keys in <String>[
        '',
        'secondary',
        'secondary-',
        'shift-shift-t',
        'hyper-t',
        'f13',
        'a b',
        'secondary-shift',
      ]) {
        expect(activatorFor(keys, _ios), isNull, reason: keys);
      }
    });
  });

  group('keyLabel', () {
    test('writes glyphs in Apple order on iOS', () {
      // Apple writes the modifiers ⌃⌥⇧⌘: Shift before Command.
      expect(keyLabel('secondary-shift-t', _ios), '⇧⌘T');
      expect(keyLabel('secondary-alt-i', _ios), '⌥⌘I');
      expect(keyLabel('ctrl-shift-tab', _ios), '⌃⇧⇥');
      expect(keyLabel('cmd-[', _ios), '⌘[');
      expect(keyLabel('escape', _ios), '⎋');
      expect(keyLabel('secondary-alt-down', _ios), '⌥⌘↓');
      expect(keyLabel('j', _ios), 'J');
    });

    test('writes names joined by plus on Android', () {
      expect(keyLabel('secondary-shift-t', _android), 'Ctrl+Shift+T');
      expect(keyLabel('secondary-alt-i', _android), 'Ctrl+Alt+I');
      expect(keyLabel('alt-left', _android), 'Alt+Left');
      expect(keyLabel('ctrl-tab', _android), 'Ctrl+Tab');
      expect(keyLabel('escape', _android), 'Esc');
      expect(keyLabel('secondary-enter', _android), 'Ctrl+Enter');
    });

    test('is empty for keys that are not a keystroke', () {
      expect(keyLabel('', _ios), '');
      expect(keyLabel('nonsense-key', _android), '');
    });
  });

  group('round trips', () {
    Glados<SingleActivator>(any.activator, kansoExploreConfig()).test(
      'a keystroke written for an activator reads back as it',
      (activator) {
        final keys = keystrokeOf(activator)!;
        for (final platform in <TargetPlatform>[_ios, _android]) {
          expect(_shapeOf(keys, platform), _expected(activator), reason: keys);
        }
      },
    );

    Glados2<SingleActivator, SingleActivator>(
      any.activator,
      any.activator,
      kansoExploreConfig(),
    ).test('different keystrokes never read as equal activators', (a, b) {
      final first = keystrokeOf(a)!;
      final second = keystrokeOf(b)!;
      for (final platform in <TargetPlatform>[_ios, _android]) {
        final same = _shapeOf(first, platform) == _shapeOf(second, platform);
        expect(same, first == second, reason: '$first $second');
      }
    });

    Glados<SingleActivator>(any.activator, kansoExploreConfig()).test(
      'secondary is Command on iOS and Control on Android',
      (activator) {
        final keys = keystrokeOf(activator)!;
        // Replace the one modifier a platform shares with "secondary".
        final apple = keys.replaceFirst('cmd-', 'secondary-');
        final other = keys.replaceFirst('ctrl-', 'secondary-');
        if (activator.meta && !activator.control) {
          expect(_shapeOf(apple, _ios), _expected(activator));
        }
        if (activator.control && !activator.meta) {
          expect(_shapeOf(other, _android), _expected(activator));
        }
        // Its label is the same glyphs with either spelling.
        if (activator.meta && !activator.control) {
          expect(keyLabel(apple, _ios), keyLabel(keys, _ios));
        }
        if (activator.control && !activator.meta) {
          expect(keyLabel(other, _android), keyLabel(keys, _android));
        }
      },
    );
  });
}

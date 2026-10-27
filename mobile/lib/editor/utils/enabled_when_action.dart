// Kanso — enabled_when_action.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

/// An action that is there only while [enabled] says so, so that the key of
/// its shortcut goes on to the field the rest of the time.
class EnabledWhenAction<T extends Intent> extends Action<T> {
  EnabledWhenAction({required this.enabled, required this.onInvoke});

  final ValueGetter<bool> enabled;
  final Object? Function(T intent) onInvoke;

  @override
  bool isEnabled(T intent) => enabled();

  @override
  bool consumesKey(T intent) => enabled();

  @override
  Object? invoke(T intent) => onInvoke(intent);
}

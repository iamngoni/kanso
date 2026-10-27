// Kanso — widget_colors_json.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:ui';

import '../configs/kanso_colors.dart';

/// The version of the file's layout, which the widgets check.
const int widgetColorsVersion = 1;

/// `#RRGGBB` for [color]: its alpha is dropped, as the widgets draw opaque
/// colours.
String hexOf(Color color) {
  final rgb = color.toARGB32() & 0xFFFFFF;
  return '#${rgb.toRadixString(16).padLeft(6, '0').toUpperCase()}';
}

Map<String, String> _section(KansoColors colors) => <String, String>{
  'background': hexOf(colors.app),
  'surface': hexOf(colors.elevated),
  'text': hexOf(colors.text),
  'muted': hexOf(colors.textMuted),
  'accent': hexOf(colors.accent),
};

/// The text of `widget.json` for the app's [light] and [dark] colours, as
/// `mobile/docs/capture.md` lays it out: the widgets' background (the
/// theme's `app`), surface (`elevated`), text, muted text and accent.
String widgetColorsJson({
  required KansoColors light,
  required KansoColors dark,
}) {
  return jsonEncode(<String, Object>{
    'version': widgetColorsVersion,
    'light': _section(light),
    'dark': _section(dark),
  });
}

// Kanso — ink_colors.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

/// The pen colours the toolbar offers, as `#rrggbb`. The first is the
/// default ink: it turns to the page's text colour on a dark page.
const List<String> inkPalette = <String>[
  '#1e1e1e',
  '#e03131',
  '#f08c00',
  '#2f9e44',
  '#1971c2',
  '#9c36b5',
];

/// The JSON Canvas card colours `1` to `6`.
const Map<String, String> cardPresets = <String, String>{
  '1': '#fb464c',
  '2': '#e9973f',
  '3': '#e0de71',
  '4': '#44cf6e',
  '5': '#53dfdd',
  '6': '#a882ff',
};

/// `#rgb` or `#rrggbb` as a colour; null for anything else.
Color? parseHex(String? hex) {
  if (hex == null || !hex.startsWith('#')) {
    return null;
  }
  var digits = hex.substring(1);
  if (digits.length == 3) {
    digits = digits.split('').map((c) => '$c$c').join();
  }
  if (digits.length != 6) {
    return null;
  }
  final value = int.tryParse(digits, radix: 16);
  return value == null ? null : Color(0xff000000 | value);
}

/// A card or arrow colour: a preset name, a hex colour, or null.
Color? cardColor(String? color) => parseHex(cardPresets[color] ?? color);

/// The colour a pen of [hex] draws in on a page whose text colour is [text]:
/// a near-black pen turns to the text colour on a dark page and a near-white
/// one on a light page, so the ink never vanishes into the background. The
/// same rule the SVG export follows.
Color inkColor(String hex, {required bool dark, required Color text}) {
  final color = parseHex(hex);
  if (color == null) {
    return text;
  }
  final luma =
      (0.299 * (color.r * 255) +
          0.587 * (color.g * 255) +
          0.114 * (color.b * 255)) /
      255;
  if ((dark && luma < 0.18) || (!dark && luma > 0.85)) {
    return text;
  }
  return color;
}

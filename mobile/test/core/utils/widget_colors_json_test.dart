// Kanso — widget_colors_json_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

import 'package:flutter/painting.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados2, any;
import 'package:kanso/core/configs/kanso_colors.dart';
import 'package:kanso/core/utils/widget_colors_json.dart';

import '../../helpers/property.dart';

const List<String> _keys = <String>[
  'background',
  'surface',
  'text',
  'muted',
  'accent',
];

final RegExp _hex = RegExp(r'^#[0-9A-F]{6}$');

KansoColors _coloured(Color color) => KansoColors.light.copyWith(
  app: color,
  elevated: color,
  text: color,
  textMuted: color,
  accent: color,
);

void main() {
  group('widgetColorsJson', () {
    test("is Kanso Light and Kanso Dark's tokens, as the contract shows", () {
      final json =
          jsonDecode(
                widgetColorsJson(
                  light: KansoColors.light,
                  dark: KansoColors.dark,
                ),
              )
              as Map<String, dynamic>;
      expect(json['version'], 1);
      expect(json['light'], <String, String>{
        'background': '#F4F1EA',
        'surface': '#FFFDF8',
        'text': '#26241F',
        'muted': '#857C6E',
        'accent': '#5E7895',
      });
      expect(json['dark'], <String, String>{
        'background': '#181816',
        'surface': '#30302B',
        'text': '#F1EFE7',
        'muted': '#7E7A70',
        'accent': '#7E96B3',
      });
    });

    test('drops the alpha and pads short values', () {
      expect(hexOf(const Color(0x80000A0B)), '#000A0B');
      expect(hexOf(const Color(0xFFFFFFFF)), '#FFFFFF');
      expect(hexOf(const Color(0xFF000000)), '#000000');
    });

    Glados2<Color, Color>(
      any.opaqueColor,
      any.opaqueColor,
      kansoExploreConfig(),
    ).test('is always JSON with all ten colours as #RRGGBB', (light, dark) {
      final text = widgetColorsJson(
        light: _coloured(light),
        dark: _coloured(dark),
      );
      final json = jsonDecode(text) as Map<String, dynamic>;
      expect(json.keys.toSet(), <String>{'version', 'light', 'dark'});
      var colours = 0;
      for (final mode in <String>['light', 'dark']) {
        final section = json[mode] as Map<String, dynamic>;
        expect(section.keys.toSet(), _keys.toSet());
        for (final key in _keys) {
          expect(section[key], matches(_hex));
          colours++;
        }
      }
      expect(colours, 10);
      // The colours are the ones given.
      expect((json['light'] as Map<String, dynamic>)['text'], hexOf(light));
      expect((json['dark'] as Map<String, dynamic>)['accent'], hexOf(dark));
    });
  });
}

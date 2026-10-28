// Kanso — theme_preview_from_installed_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:io';
import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/themes/utils/theme_preview_from_installed.dart';

import '../../helpers/real_themes.dart';

Color _hex(String value) {
  return Color(0xFF000000 | int.parse(value.substring(1, 7), radix: 16));
}

Map<String, dynamic> _object(Object? value) => value! as Map<String, dynamic>;

void main() {
  loadRealThemes();

  test('every theme previews in the colours the desktop draws it in', () {
    final golden = _object(
      _object(
        jsonDecode(
          File('../crates/kanso-app/src/themes.golden.json').readAsStringSync(),
        ),
      )['themes'],
    );
    expect(realThemes, hasLength(greaterThan(70)));
    for (final theme in realThemes) {
      final entry = _object(golden[theme.id]);
      final colors = _object(entry['colors']);
      final syntax = _object(_object(entry['own_panes'])['syntax']);
      Color token(String name, Color fallback) {
        final color = _object(syntax[name])['color'] as String?;
        return color == null ? fallback : _hex(color);
      }

      final id = theme.id;
      final preview = themePreviewFromInstalled(installedFromBuiltin(theme));
      final text = _hex(colors['text'] as String);
      final marker = token('marker', token('text', text));
      expect(preview.id, id);
      expect(preview.name, entry['name'], reason: id);
      expect(preview.dark, entry['dark'], reason: id);
      expect(preview.builtIn, isTrue, reason: id);
      expect(preview.editor, _hex(colors['editor'] as String), reason: id);
      expect(preview.text, text, reason: id);
      expect(preview.accent, _hex(colors['accent'] as String), reason: id);
      expect(preview.marker, marker, reason: '$id: marker');
      expect(
        preview.heading,
        token('heading', token('text', text)),
        reason: '$id: heading',
      );
      expect(preview.link, token('link', token('text', text)), reason: id);
      expect(preview.url, token('url', token('text', text)), reason: id);
      expect(preview.code, token('code', token('text', text)), reason: id);
      // The quote falls back to the marker, bold and italic to the text.
      expect(preview.quote, token('quote', marker), reason: '$id: quote');
      expect(preview.strong, token('strong', text), reason: '$id: strong');
      expect(
        preview.emphasis,
        token('emphasis', text),
        reason: '$id: emphasis',
      );
    }
  });
}

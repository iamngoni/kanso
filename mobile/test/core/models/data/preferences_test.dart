// Kanso — preferences_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_fonts.dart';
import 'package:kanso/core/models/data/kanso_text_size.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/theme_selection.dart';

import '../../../helpers/property.dart';

void main() {
  group('Preferences', () {
    test('defaults', () {
      const preferences = Preferences();
      expect(preferences.textSize, KansoTextSize.standard);
      expect(preferences.previewTextSize, KansoTextSize.standard);
      expect(preferences.diagramRenderer, DiagramRenderer.pele);
      expect(preferences.themeSelection.syntax, isNull);
      expect(preferences.themeSelection.preview, isNull);
      expect(preferences.sort, SortKey.updated);
      expect(preferences.descending, isTrue);
      expect(preferences.backupNoticeSeen, isFalse);
      expect(preferences.iconTheme, 'lucide');
      expect(preferences.fonts, const KansoFonts());
    });

    Glados<Preferences>(any.preferences, kansoExploreConfig()).test(
      'survive being written as JSON text and read back',
      (preferences) {
        final text = jsonEncode(preferences.toJson());
        expect(
          Preferences.fromJson(jsonDecode(text) as Map<String, dynamic>),
          preferences,
        );
      },
    );

    test('an empty file is the defaults', () {
      expect(Preferences.fromJson(<String, dynamic>{}), const Preferences());
    });

    test('a choice this version does not know is the default one', () {
      final read = Preferences.fromJson(<String, dynamic>{
        'textSize': 'huge',
        'sort': 'relevance',
        'fonts': <String, dynamic>{
          'editorFont': 'comicSans',
          'previewFont': 'literata',
          'codeFont': 'wingdings',
        },
        'descending': false,
      });
      expect(
        read,
        const Preferences(
          descending: false,
          fonts: KansoFonts(previewFont: KansoFontFamily.literata),
        ),
      );
    });

    test('an icon theme is kept by its ID', () {
      final json = const Preferences(iconTheme: 'phosphor').toJson();
      expect(json['iconTheme'], 'phosphor');
      expect(
        Preferences.fromJson(json),
        const Preferences(iconTheme: 'phosphor'),
      );
    });

    test('is written with names, not numbers', () {
      final json = const Preferences(
        sort: SortKey.title,
        textSize: KansoTextSize.large,
      ).toJson();
      expect(json['sort'], 'title');
      expect(json['textSize'], 'large');
      expect(json['themeSelection'], isA<Map<String, dynamic>>());
    });

    test(
      'a file from before the preview settings loads with their defaults',
      () {
        final read = Preferences.fromJson(<String, dynamic>{
          'themeSelection': <String, dynamic>{
            'followSystem': false,
            'ui': 'nord',
            'light': 'kanso-light',
            'dark': 'kanso-dark',
          },
          'fonts': <String, dynamic>{'editorFont': 'literata'},
          'textSize': 'large',
          'sort': 'title',
          'descending': false,
          'backupNoticeSeen': true,
        });
        expect(read.textSize, KansoTextSize.large);
        expect(read.previewTextSize, KansoTextSize.standard);
        expect(read.diagramRenderer, DiagramRenderer.pele);
        expect(read.themeSelection.syntax, isNull);
        expect(read.themeSelection.preview, isNull);
        expect(
          read.themeSelection,
          const ThemeSelection(followSystem: false, ui: 'nord'),
        );
        expect(read.sort, SortKey.title);
        expect(read.descending, isFalse);
        expect(read.backupNoticeSeen, isTrue);
      },
    );

    test('a preview size or diagram renderer this version does not know is '
        'the default one', () {
      final read = Preferences.fromJson(<String, dynamic>{
        'previewTextSize': 'enormous',
        'diagramRenderer': 'mermaidJs',
      });
      expect(read.previewTextSize, KansoTextSize.standard);
      expect(read.diagramRenderer, DiagramRenderer.pele);
    });

    test('writes the preview settings with names', () {
      final json = const Preferences(
        previewTextSize: KansoTextSize.larger,
        diagramRenderer: DiagramRenderer.kanso,
      ).toJson();
      expect(json['previewTextSize'], 'larger');
      expect(json['diagramRenderer'], 'kanso');
    });
  });
}

// Kanso — preferences_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/kanso_font_family.dart';
import 'package:kanso/core/models/data/kanso_fonts.dart';
import 'package:kanso/core/models/data/preferences.dart';
import 'package:kanso/core/models/data/theme_selection.dart';

import '../../../helpers/property.dart';

void main() {
  group('Preferences', () {
    test('defaults', () {
      const preferences = Preferences();
      expect(preferences.textSize, 1.0);
      expect(preferences.previewTextSize, 1.0);
      expect(preferences.interfaceTextSize, 1.0);
      expect(preferences.diagramRenderer, DiagramRenderer.pele);
      expect(preferences.themeSelection.syntax, isNull);
      expect(preferences.themeSelection.preview, isNull);
      expect(preferences.sort, SortKey.updated);
      expect(preferences.descending, isTrue);
      expect(preferences.backupNoticeSeen, isFalse);
      expect(preferences.iconTheme, 'lucide');
      expect(preferences.fonts, const KansoFonts());
      expect(preferences.focusMode, isFalse);
      expect(preferences.livePreviewInFocus, isFalse);
      expect(preferences.pdfOriginalPages, isEmpty);
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

    test('keeps the PDFs shown as printed by attachment ID', () {
      const preferences = Preferences(
        pdfOriginalPages: <String, bool>{'file:a': true},
      );
      final json = preferences.toJson();
      expect(json['pdfOriginalPages'], <String, dynamic>{'file:a': true});
      expect(Preferences.fromJson(json), preferences);
    });

    test('writes note scales as numbers', () {
      final json = const Preferences(
        sort: SortKey.title,
        textSize: 1.15,
      ).toJson();
      expect(json['sort'], 'title');
      expect(json['textSize'], 1.15);
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
        expect(read.textSize, 1.15);
        expect(read.previewTextSize, 1.0);
        expect(read.interfaceTextSize, 1.0);
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
      expect(read.previewTextSize, 1.0);
      expect(read.diagramRenderer, DiagramRenderer.pele);
    });

    test('writes and reads the interface scale independently', () {
      final json = const Preferences(interfaceTextSize: 1.2).toJson();
      expect(json['interfaceTextSize'], 1.2);
      expect(
        Preferences.fromJson(<String, dynamic>{
          'interfaceTextSize': json['interfaceTextSize'],
        }).interfaceTextSize,
        1.2,
      );
      expect(
        Preferences.fromJson(<String, dynamic>{
          'interfaceTextSize': 1.23,
        }).interfaceTextSize,
        1.25,
      );
    });

    test('bounds and steps the interface scale within its own range', () {
      expect(
        Preferences.fromJson(<String, dynamic>{
          'interfaceTextSize': 0.1,
        }).interfaceTextSize,
        0.9,
      );
      expect(
        Preferences.fromJson(<String, dynamic>{
          'interfaceTextSize': 4,
        }).interfaceTextSize,
        1.3,
      );
      expect(
        Preferences.fromJson(<String, dynamic>{
          'interfaceTextSize': 'huge',
        }).interfaceTextSize,
        1.0,
      );
      expect(
        Preferences.fromJson(<String, dynamic>{
          'interfaceTextSize': double.nan,
        }).interfaceTextSize,
        1.0,
      );
    });

    test('reads every legacy preset name and writes numeric scales', () {
      final legacy = <String, double>{
        'small': 0.9,
        'standard': 1,
        'large': 1.15,
        'larger': 1.3,
      };
      for (final entry in legacy.entries) {
        expect(
          Preferences.fromJson(<String, dynamic>{
            'textSize': entry.key,
          }).textSize,
          entry.value,
        );
      }
      final json = const Preferences(
        previewTextSize: 1.3,
        diagramRenderer: DiagramRenderer.kanso,
      ).toJson();
      expect(json['previewTextSize'], 1.3);
      expect(json['diagramRenderer'], 'kanso');
    });

    test('normalizes numeric scales to bounds and the nearest step', () {
      expect(
        Preferences.fromJson(<String, dynamic>{'textSize': 1.23}).textSize,
        1.25,
      );
      expect(
        Preferences.fromJson(<String, dynamic>{'textSize': 0.1}).textSize,
        0.8,
      );
      expect(
        Preferences.fromJson(<String, dynamic>{
          'previewTextSize': 4,
        }).previewTextSize,
        1.8,
      );
      expect(
        Preferences.fromJson(<String, dynamic>{'textSize': 'unknown'}).textSize,
        1.0,
      );
      expect(Preferences.fromJson(<String, dynamic>{}).textSize, 1.0);
    });
  });
}

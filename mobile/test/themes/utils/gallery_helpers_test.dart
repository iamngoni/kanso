// Kanso — gallery_helpers_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados, Glados2, IntAnys, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/utils/palette_from_tokens.dart';
import 'package:kanso/themes/models/data/gallery_problem.dart';
import 'package:kanso/themes/utils/gallery_problem_of.dart';
import 'package:kanso/themes/utils/gallery_source_label.dart';
import 'package:kanso/themes/utils/installed_color_theme_ids.dart';
import 'package:kanso/themes/utils/theme_preview_from_gallery.dart';

import '../../helpers/gallery_fixtures.dart';
import '../../helpers/property.dart';
import '../../helpers/sharing_property.dart';

void main() {
  group('gallerySourceLabel', () {
    test('drops the scheme, www and a closing slash', () {
      expect(
        gallerySourceLabel('https://github.com/iamngoni/kanso-themes/'),
        'github.com/iamngoni/kanso-themes',
      );
      expect(
        gallerySourceLabel(' http://www.example.invalid/x '),
        'example.invalid/x',
      );
      expect(gallerySourceLabel('github.com/a/b'), 'github.com/a/b');
    });

    Glados<int>(any.intInRange(0, 1000), kansoExploreConfig()).test(
      'is the same once more, and leaves no scheme or closing slash',
      (n) {
        for (final source in <String>[
          'https://github.com/u$n/repo/',
          'http://www.site$n.invalid//',
          'github.com/u$n',
        ]) {
          final label = gallerySourceLabel(source);
          expect(gallerySourceLabel(label), label);
          expect(label.contains('://'), isFalse);
          expect(label.endsWith('/'), isFalse);
          expect(label.startsWith('www.'), isFalse);
        }
      },
    );
  });

  group('galleryProblemOf', () {
    test('shows the library’s sentence as it is, trimmed', () {
      expect(
        galleryProblemOf(
          const ApplicationError.invalid(
            message: ' Nord needs a newer Kanso. ',
          ),
        ),
        const GalleryProblem.words('Nord needs a newer Kanso.'),
      );
    });

    test('says offline for the network, and nothing of the rest', () {
      expect(
        galleryProblemOf(const ApplicationError.network(message: 'dns')),
        const GalleryProblem.offline(),
      );
      for (final error in const <ApplicationError>[
        ApplicationError.invalid(message: '  '),
        ApplicationError.internal(message: 'boom'),
        ApplicationError.storage(message: 'disk'),
        ApplicationError.notFound(message: 'x'),
        ApplicationError.unexpected(message: 'y'),
      ]) {
        expect(galleryProblemOf(error), const GalleryProblem.unexpected());
      }
    });
  });

  group('installedColorThemeIds', () {
    Glados2<List<GalleryItem>, String>(
      any.galleryItems,
      any.gallerySearch,
      kansoExploreConfig(),
    ).test('are installed colour themes that are drawn, and only those', (
      items,
      _,
    ) {
      final ids = installedColorThemeIds(items);
      for (final item in items) {
        expect(
          ids.contains(item.id),
          item.installed &&
              !item.disabled &&
              item.kind == GalleryKind.colorTheme,
        );
      }
    });
  });

  group('themePreviewFromGallery', () {
    test('draws a theme from the catalog’s five colours', () {
      final preview = themePreviewFromGallery(
        galleryItem('dusk', name: 'Dusk', dark: true),
      )!;
      expect(preview.id, 'dusk');
      expect(preview.name, 'Dusk');
      expect(preview.dark, isTrue);
      expect(preview.builtIn, isFalse);
      expect(preview.editor, opaqueColor(darkHeadline.editor));
      expect(preview.text, opaqueColor(darkHeadline.text));
      expect(preview.accent, opaqueColor(darkHeadline.accent));
      expect(preview.heading, preview.accent);
    });

    test('has none for an icon theme', () {
      expect(
        themePreviewFromGallery(
          galleryItem('lines', kind: GalleryKind.iconTheme),
        ),
        isNull,
      );
    });

    Glados2<int, int>(
      any.intInRange(0, 0x1000000),
      any.intInRange(0, 0x1000000),
      kansoExploreConfig(),
    ).test('every colour is opaque and the faded ones lie between', (a, b) {
      final preview = themePreviewFromGallery(
        galleryItem(
          'x',
          headline: GalleryColors(
            editor: a,
            text: b,
            accent: a ^ b,
            sidebar: a,
            list: b,
          ),
        ),
      )!;
      for (final color in <Color>[
        preview.editor,
        preview.text,
        preview.accent,
        preview.heading,
        preview.marker,
        preview.quote,
        preview.link,
        preview.url,
        preview.code,
      ]) {
        expect(color.a, 1.0);
      }
    });
  });
}

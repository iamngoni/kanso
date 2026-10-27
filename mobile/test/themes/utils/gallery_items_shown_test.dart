// Kanso — gallery_items_shown_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter_test/flutter_test.dart';
import 'package:glados/glados.dart' show Glados2, any;
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/themes/models/data/gallery_filter.dart';
import 'package:kanso/themes/utils/gallery_items_shown.dart';

import '../../helpers/gallery_fixtures.dart';
import '../../helpers/property.dart';
import '../../helpers/sharing_property.dart';

Set<String> _ids(List<GalleryItem> items) => {
  for (final item in items) item.id,
};

void main() {
  group('galleryItemsShown properties', () {
    Glados2<List<GalleryItem>, String>(
      any.galleryItems,
      any.gallerySearch,
      kansoExploreConfig(),
    ).test('lists only items of the gallery, none twice', (items, query) {
      for (final filter in GalleryFilter.values) {
        final shown = galleryItemsShown(items, query, filter);
        expect(_ids(shown).length, shown.length);
        expect(_ids(items).containsAll(_ids(shown)), isTrue);
      }
    });

    Glados2<List<GalleryItem>, String>(
      any.galleryItems,
      any.gallerySearch,
      kansoExploreConfig(),
    ).test('the order does not change which items are found', (items, query) {
      expect(
        _ids(galleryItemsShown(items, query, GalleryFilter.popular)),
        _ids(galleryItemsShown(items, query, GalleryFilter.recent)),
      );
    });

    Glados2<List<GalleryItem>, String>(
      any.galleryItems,
      any.gallerySearch,
      kansoExploreConfig(),
    ).test('the list does not depend on the order the catalog gave', (
      items,
      query,
    ) {
      for (final filter in GalleryFilter.values) {
        expect(
          galleryItemsShown(items.reversed.toList(), query, filter),
          galleryItemsShown(items, query, filter),
        );
      }
    });

    Glados2<List<GalleryItem>, String>(
      any.galleryItems,
      any.gallerySearch,
      kansoExploreConfig(),
    ).test('every item shown has every word of the search', (items, query) {
      final words = query
          .toLowerCase()
          .split(' ')
          .where((word) => word.isNotEmpty);
      for (final item in galleryItemsShown(
        items,
        query,
        GalleryFilter.popular,
      )) {
        final text =
            '${item.name} ${item.author} ${item.id} ${item.description}'
                .toLowerCase();
        for (final word in words) {
          expect(text, contains(word));
        }
      }
    });

    Glados2<List<GalleryItem>, String>(
      any.galleryItems,
      any.gallerySearch,
      kansoExploreConfig(),
    ).test('no search finds everything', (items, _) {
      for (final blank in <String>['', '   ']) {
        expect(
          _ids(galleryItemsShown(items, blank, GalleryFilter.popular)),
          _ids(items),
        );
      }
    });

    Glados2<List<GalleryItem>, String>(
      any.galleryItems,
      any.gallerySearch,
      kansoExploreConfig(),
    ).test('recently updated puts the newest first', (items, query) {
      final shown = galleryItemsShown(items, query, GalleryFilter.recent);
      for (var i = 1; i < shown.length; i++) {
        expect(
          shown[i - 1].updatedMs,
          greaterThanOrEqualTo(shown[i].updatedMs),
        );
      }
    });

    Glados2<List<GalleryItem>, String>(
      any.galleryItems,
      any.gallerySearch,
      kansoExploreConfig(),
    ).test('popular puts the most installed first, Kanso’s own among equals', (
      items,
      query,
    ) {
      final shown = galleryItemsShown(items, query, GalleryFilter.popular);
      for (var i = 1; i < shown.length; i++) {
        final before = shown[i - 1];
        final after = shown[i];
        expect(before.installs, greaterThanOrEqualTo(after.installs));
        if (before.installs == after.installs &&
            before.official != after.official) {
          expect(before.official, isTrue);
        }
      }
    });
  });

  group('galleryItemsShown examples', () {
    final items = <GalleryItem>[
      galleryItem('nord', name: 'Nord', updatedMs: 1),
      galleryItem(
        'dusk',
        name: 'Dusk',
        author: 'Kanso',
        official: true,
        updatedMs: 3,
      ),
      galleryItem('paper', name: 'Paper', author: 'Bo', updatedMs: 2),
    ];

    test('popular, with no installs counted, is Kanso’s own then by name', () {
      expect(
        galleryItemsShown(items, '', GalleryFilter.popular).map((i) => i.id),
        <String>['dusk', 'nord', 'paper'],
      );
    });

    test('recently updated is the newest first', () {
      expect(
        galleryItemsShown(items, '', GalleryFilter.recent).map((i) => i.id),
        <String>['dusk', 'paper', 'nord'],
      );
    });

    test('a search finds a name or an author, whatever the case', () {
      expect(
        galleryItemsShown(
          items,
          'NORD',
          GalleryFilter.popular,
        ).map((i) => i.id),
        <String>['nord'],
      );
      expect(
        galleryItemsShown(
          items,
          ' kanso ',
          GalleryFilter.popular,
        ).map((i) => i.id),
        <String>['dusk'],
      );
    });

    test('a search with no match finds nothing', () {
      expect(galleryItemsShown(items, 'zzz', GalleryFilter.popular), isEmpty);
    });
  });
}

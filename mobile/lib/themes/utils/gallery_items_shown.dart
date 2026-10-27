// Kanso — gallery_items_shown.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import '../../core/bridge/api/types.dart';
import '../models/data/gallery_filter.dart';

/// The gallery's [items] that match [query], in the order [filter] gives.
///
/// An item matches when every word of the query is in its name, author, id
/// or description, whatever the case; no query matches all. The order is a
/// total one (ties go by name and then id), so the same items are always
/// listed the same way.
List<GalleryItem> galleryItemsShown(
  List<GalleryItem> items,
  String query,
  GalleryFilter filter,
) {
  final words = query
      .toLowerCase()
      .split(RegExp(r'\s+'))
      .where((word) => word.isNotEmpty)
      .toList();
  bool matches(GalleryItem item) {
    final text = '${item.name} ${item.author} ${item.id} ${item.description}'
        .toLowerCase();
    return words.every(text.contains);
  }

  int byName(GalleryItem a, GalleryItem b) {
    final named = a.name.toLowerCase().compareTo(b.name.toLowerCase());
    return named != 0 ? named : a.id.compareTo(b.id);
  }

  final shown = items.where(matches).toList()
    ..sort(switch (filter) {
      GalleryFilter.popular => (a, b) {
        final installs = b.installs.compareTo(a.installs);
        if (installs != 0) {
          return installs;
        }
        if (a.official != b.official) {
          return a.official ? -1 : 1;
        }
        return byName(a, b);
      },
      GalleryFilter.recent => (a, b) {
        final updated = b.updatedMs.compareTo(a.updatedMs);
        return updated != 0 ? updated : byName(a, b);
      },
    });
  return shown;
}

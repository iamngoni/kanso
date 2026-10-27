// Kanso — gallery_source_label.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// Where a theme's page is, as a short line: [source] without its scheme,
/// its `www.` and a closing slash. The address is shown, never fetched or
/// opened.
String gallerySourceLabel(String source) {
  var label = source.trim();
  label = label.replaceFirst(RegExp('^[a-zA-Z][a-zA-Z0-9+.-]*://'), '');
  label = label.replaceFirst(RegExp(r'^www\.'), '');
  while (label.endsWith('/')) {
    label = label.substring(0, label.length - 1);
  }
  return label;
}

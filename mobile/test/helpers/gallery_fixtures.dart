// Kanso — gallery_fixtures.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/core/bridge/api/types.dart';

/// A theme in the gallery, for tests: a colour theme with the colours of a
/// dark or light card unless said otherwise.
GalleryItem galleryItem(
  String id, {
  String? name,
  GalleryKind kind = GalleryKind.colorTheme,
  String author = 'Ana',
  bool official = false,
  String version = '2',
  int installs = 0,
  int updatedMs = 1790000000000,
  String? source,
  bool dark = false,
  GalleryColors? headline,
  bool installed = false,
  bool updateAvailable = false,
  bool runsHere = true,
  String description = 'A calm theme.',
  bool disabled = false,
}) {
  return GalleryItem(
    id: id,
    name: name ?? id,
    kind: kind,
    author: author,
    official: official,
    version: version,
    installs: installs,
    updatedMs: updatedMs,
    source: source ?? 'https://github.com/example/$id',
    dark: dark,
    headline: kind == GalleryKind.iconTheme
        ? null
        : headline ?? (dark ? darkHeadline : lightHeadline),
    installed: installed,
    updateAvailable: updateAvailable,
    runsHere: runsHere,
    description: description,
    license: 'MIT',
    edited: false,
    disabled: disabled,
  );
}

/// The headline colours of a light card.
const GalleryColors lightHeadline = GalleryColors(
  editor: 0xFBF7F0,
  text: 0x2B2723,
  accent: 0xB8651B,
  sidebar: 0xF2EBDF,
  list: 0xF7F1E6,
);

/// The headline colours of a dark card.
const GalleryColors darkHeadline = GalleryColors(
  editor: 0x1E2128,
  text: 0xD8DEE9,
  accent: 0x88C0D0,
  sidebar: 0x191C22,
  list: 0x22262E,
);

/// A small gallery: Kanso's own dark theme, a community light one, an icon
/// theme, one installed, and one that has an update.
List<GalleryItem> sampleGallery() {
  return <GalleryItem>[
    galleryItem(
      'dusk',
      name: 'Dusk',
      author: 'Kanso',
      official: true,
      dark: true,
      version: '3',
      source: 'https://github.com/iamngoni/kanso-themes',
    ),
    galleryItem(
      'paper',
      name: 'Paper',
      installed: true,
      updateAvailable: true,
      version: '4',
      updatedMs: 1790500000000,
    ),
    galleryItem(
      'moss',
      name: 'Moss',
      author: 'Bo',
      installed: true,
      headline: const GalleryColors(
        editor: 0xEEF2E6,
        text: 0x26301F,
        accent: 0x4F7A28,
        sidebar: 0xE3EAD8,
        list: 0xE9EFDF,
      ),
    ),
    galleryItem(
      'lines',
      name: 'Lines',
      kind: GalleryKind.iconTheme,
      author: 'Cy',
    ),
  ];
}

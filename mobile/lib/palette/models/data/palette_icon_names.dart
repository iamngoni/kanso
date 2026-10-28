// Kanso — palette_icon_names.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

/// The bundled Lucide icon nearest to a name the desktop's palette asks for
/// that this app does not bundle itself.
///
/// The catalog's names are the desktop's; the app's own set is the icons it
/// draws. A name outside it resolves to its nearest neighbour here, and a
/// name outside both resolves to [paletteFallbackIcon].
const Map<String, String> paletteIconRenames = <String, String>{
  'columns-2': 'layout-template',
  'file': 'file-text',
  'file-key': 'file-text',
  'file-plus': 'file-text',
  'focus': 'target',
  'goal': 'target',
  'hard-drive': 'database',
  'key-round': 'key',
  'link-2': 'link',
  'log-in': 'log-out',
  'message-square-plus': 'message-square',
  'panel-right': 'panel-left',
  'pin-off': 'pin',
  'refresh-ccw': 'refresh-cw',
  'replace': 'replace-all',
  'square-kanban': 'layout-template',
  'square-pen': 'pencil',
  'table': 'layout-template',
  'table-of-contents': 'list',
  'upload': 'download',
  'user-plus': 'users',
  'waypoints': 'git-branch',
};

/// What a palette row draws when even [paletteIconRenames] has no entry.
const String paletteFallbackIcon = 'file-text';

/// The icon [name] should draw, in the app's own set.
String paletteIconName(String name) => paletteIconRenames[name] ?? name;

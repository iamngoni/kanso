// Kanso — theme_changes.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import '../../themes/utils/installed_color_theme_ids.dart';
import '../bridge/api/types.dart';

/// Which colour themes installed from the gallery changed, told to whoever
/// listens to [changes].
///
/// The gallery's items are [seen] after every read of the index, install and
/// removal. A theme that was installed and no longer is (removed, or
/// withdrawn by the catalog and so removed by the fetch), one installed
/// since, and any of `touched` (installed again: new colours) is told. The
/// first items seen are the start, and only `touched` is told for them.
class ThemeChanges {
  final StreamController<String> _changes =
      StreamController<String>.broadcast();
  Set<String>? _installed;

  /// The ids of the colour themes that changed, as they change.
  Stream<String> get changes => _changes.stream;

  /// The gallery's [items] now; [touched] were just installed or removed.
  void seen(List<GalleryItem> items, {Set<String> touched = const {}}) {
    final after = installedColorThemeIds(items);
    final before = _installed ?? after;
    _installed = after;
    <String>{
      ...before.difference(after),
      ...after.difference(before),
      ...touched,
    }.forEach(_changes.add);
  }
}

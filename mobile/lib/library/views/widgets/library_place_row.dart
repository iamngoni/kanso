// Kanso — library_place_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';

/// A row of the library that opens the notes of a place. Where the library
/// is a drawer, it slides away as the list behind it changes.
class LibraryPlaceRow extends StatelessWidget {
  const LibraryPlaceRow({
    required this.place,
    required this.label,
    this.icon,
    this.leading,
    this.count,
    this.depth = 0,
    this.chevron = false,
    this.selected = false,
    this.locked = false,
    this.onLongPress,
    super.key,
  });

  final ListPlace place;
  final String label;
  final KansoIcons? icon;

  /// Replaces [icon], for a coloured dot.
  final Widget? leading;
  final int? count;
  final int depth;
  final bool chevron;
  final bool selected;

  /// Whether the place is a locked notebook.
  final bool locked;

  /// What a long press does: a notebook's menu.
  final VoidCallback? onLongPress;

  @override
  Widget build(BuildContext context) {
    return KansoSidebarRow(
      icon: icon,
      leading: leading,
      label: label,
      count: count,
      depth: depth,
      chevron: chevron,
      selected: selected,
      locked: locked,
      onLongPress: onLongPress,
      onTap: () {
        KansoDrawerScope.maybeOf(context)?.close();
        context.go(BrowseLocation.place(place).location);
      },
    );
  }
}

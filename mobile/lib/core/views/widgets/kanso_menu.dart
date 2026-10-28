// Kanso — kanso_menu.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_radii.dart';
import '../../utils/context_extensions.dart';

/// A row of a menu opened with [showKansoMenu]; [label] in the body style,
/// dimmed when it is not [enabled].
PopupMenuItem<T> kansoMenuItem<T>(
  BuildContext context,
  T value,
  String label, {
  bool enabled = true,
}) {
  return PopupMenuItem<T>(
    value: value,
    enabled: enabled,
    height: 40,
    child: Text(
      label,
      style: context.typography.body.copyWith(
        color: enabled ? context.colors.text : context.colors.textMuted,
      ),
    ),
  );
}

/// Shows [items] in a small menu with its top left corner at [at] (a point
/// of the screen), in the app's colours; what was chosen, or null.
Future<T?> showKansoMenu<T>(
  BuildContext context,
  Offset at,
  List<PopupMenuEntry<T>> items,
) {
  final colors = context.colors;
  final overlay = Overlay.of(context).context.findRenderObject()! as RenderBox;
  return showMenu<T>(
    context: context,
    position: RelativeRect.fromRect(
      Rect.fromLTWH(at.dx, at.dy, 0, 0),
      Offset.zero & overlay.size,
    ),
    color: colors.elevated,
    surfaceTintColor: Colors.transparent,
    shape: RoundedRectangleBorder(
      borderRadius: KansoRadii.mediumBorder,
      side: BorderSide(color: colors.divider),
    ),
    items: items,
  );
}

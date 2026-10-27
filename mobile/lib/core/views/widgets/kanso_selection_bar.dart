// Kanso — kanso_selection_bar.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../configs/kanso_icons.dart';
import '../../configs/kanso_spacing.dart';
import '../../utils/context_extensions.dart';
import 'kanso_icon_button.dart';
import 'kanso_pressable.dart';
import 'kanso_top_bar.dart';

/// The bar that stands in for a list's own while several notes are chosen:
/// how many, what to do with them and a way out.
///
/// Pinning and trashing are left out of [onPin] and [onTrash] when they do
/// not apply (the trash), and then are not shown.
class KansoSelectionBar extends StatelessWidget {
  const KansoSelectionBar({
    required this.count,
    required this.onDone,
    required this.onMove,
    this.pinned = false,
    this.onPin,
    this.onTrash,
    this.onMore,
    this.backgroundColor,
    super.key,
  });

  /// How many notes are chosen.
  final int count;

  /// Whether every note chosen is pinned: Pin then reads Unpin.
  final bool pinned;
  final VoidCallback onDone;
  final VoidCallback? onPin;
  final VoidCallback onMove;
  final VoidCallback? onTrash;

  /// What else can be done with the notes chosen, as a menu.
  final VoidCallback? onMore;
  final Color? backgroundColor;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final pin = onPin;
    final trash = onTrash;
    return KansoTopBar(
      backgroundColor: backgroundColor ?? colors.list,
      leading: KansoPressable(
        onTap: onDone,
        child: Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: KansoSpacing.sm,
            vertical: KansoSpacing.sm,
          ),
          child: Text(
            l10n.noteListDone,
            style: context.typography.body.copyWith(color: colors.accent),
          ),
        ),
      ),
      title: l10n.noteListSelected(count),
      trailing: <Widget>[
        if (pin != null)
          KansoIconButton(
            icon: KansoIcons.pin,
            semanticLabel: pinned ? l10n.noteListUnpin : l10n.noteListPin,
            selected: pinned,
            onPressed: pin,
          ),
        KansoIconButton(
          icon: KansoIcons.notebook,
          semanticLabel: l10n.noteListMove,
          onPressed: onMove,
        ),
        if (trash != null)
          KansoIconButton(
            icon: KansoIcons.trash,
            semanticLabel: l10n.noteListTrash,
            onPressed: trash,
          ),
        if (onMore != null)
          KansoIconButton(
            icon: KansoIcons.ellipsis,
            semanticLabel: l10n.noteActionMore,
            onPressed: onMore,
          ),
      ],
    );
  }
}

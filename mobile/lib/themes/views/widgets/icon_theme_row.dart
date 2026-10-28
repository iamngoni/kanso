// Kanso — icon_theme_row.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/models/data/installed_icon_theme.dart';
import '../../../core/models/data/installed_theme_origin.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_badge.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';

/// An icon theme in the list: its name, who made it and where it came from.
///
/// Choosing an icon theme comes later; its check mark and its tap go in
/// [trailing] and [onTap].
class IconThemeRow extends StatelessWidget {
  const IconThemeRow({
    required this.theme,
    this.trailing,
    this.onTap,
    super.key,
  });

  final InstalledIconTheme theme;
  final Widget? trailing;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final author = theme.author;
    return KansoSettingsRow(
      icon: KansoIcons.shapes,
      label: author.isEmpty
          ? theme.name
          : '${theme.name} · ${l10n.iconThemeAuthor(author)}',
      trailing:
          trailing ??
          KansoBadge(
            label: switch (theme.origin) {
              InstalledThemeOrigin.builtin => l10n.themesBadgeBuiltIn,
              InstalledThemeOrigin.own => l10n.themesBadgeYours,
              InstalledThemeOrigin.gallery => l10n.themesBadgeGallery,
            },
          ),
      onTap: onTap,
    );
  }
}

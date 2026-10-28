// Kanso — installed_theme_card.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/installed_theme_origin.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_badge.dart';
import '../../../core/views/widgets/kanso_check_badge.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../models/data/theme_card_kind.dart';
import '../../models/data/theme_preview.dart';
import 'theme_code_preview.dart';
import 'theme_note_preview.dart';
import 'theme_syntax_preview.dart';

/// A theme in a row of cards: a picture in its colours (of the kind of
/// colouring the row chooses, see [ThemeCardKind]), its name and a badge
/// that says where it came from. The theme in use is outlined in the accent
/// colour and wears a check mark badge on its picture. Pressing it chooses
/// the theme.
///
/// The "Same as theme" card of the syntax and preview rows is made with
/// [InstalledThemeCard.same]: it shows the picture of the interface theme in
/// use and carries no badge.
class InstalledThemeCard extends StatelessWidget {
  const InstalledThemeCard({
    required ThemePreview this.theme,
    this.kind = ThemeCardKind.ui,
    this.current = false,
    this.onTap,
    super.key,
  }) : same = false,
       _picture = theme;

  /// The "Same as theme" card, showing the picture of [sameAs], the
  /// interface theme in use.
  const InstalledThemeCard.same({
    required ThemePreview sameAs,
    required this.kind,
    this.current = false,
    this.onTap,
    super.key,
  }) : theme = null,
       same = true,
       _picture = sameAs;

  /// The theme the card chooses; null for the "Same as theme" card.
  final ThemePreview? theme;
  final ThemeCardKind kind;
  final bool current;

  /// Whether this is the "Same as theme" card.
  final bool same;

  /// The theme whose picture is drawn.
  final ThemePreview _picture;

  /// What choosing the theme does.
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final chosen = theme;
    final name = chosen == null ? l10n.themesSameAsTheme : chosen.name;
    final picture = switch (kind) {
      ThemeCardKind.ui => ThemeCodePreview(theme: _picture),
      ThemeCardKind.syntax => ThemeSyntaxPreview(theme: _picture),
      ThemeCardKind.preview => ThemeNotePreview(theme: _picture),
    };
    final card = Container(
      width: KansoSizes.themeCardWidth,
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: colors.elevated,
        borderRadius: KansoRadii.largeBorder,
        border: Border.all(color: colors.divider),
      ),
      // Drawn over the card, so the outline does not move its content.
      foregroundDecoration: current
          ? BoxDecoration(
              borderRadius: KansoRadii.largeBorder,
              border: Border.all(color: colors.accent, width: 2),
            )
          : null,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Stack(
            children: <Widget>[
              picture,
              if (current)
                PositionedDirectional(
                  top: KansoSpacing.xs,
                  end: KansoSpacing.xs,
                  child: KansoCheckBadge(semanticLabel: l10n.themesCurrent),
                ),
            ],
          ),
          Padding(
            padding: const EdgeInsets.all(KansoSpacing.sm),
            child: Row(
              children: <Widget>[
                Expanded(
                  child: Text(
                    name,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: context.typography.listTitle,
                  ),
                ),
                if (chosen != null) ...<Widget>[
                  const SizedBox(width: KansoSpacing.xs),
                  KansoBadge(
                    label: switch (chosen.origin) {
                      InstalledThemeOrigin.builtin => l10n.themesBadgeBuiltIn,
                      InstalledThemeOrigin.own => l10n.themesBadgeYours,
                      InstalledThemeOrigin.gallery => l10n.themesBadgeGallery,
                    },
                  ),
                ],
              ],
            ),
          ),
        ],
      ),
    );
    return Semantics(
      selected: current,
      child: KansoPressable(
        onTap: onTap,
        semanticLabel: name,
        child: card,
      ),
    );
  }
}

// Kanso — installed_theme_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/theme_selection.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../models/data/theme_card_kind.dart';
import '../../models/data/theme_preview.dart';
import 'installed_theme_card.dart';

/// The installed themes side by side, in a row that scrolls sideways, as
/// the choice of one kind of colouring ([kind]): the interface's, the
/// Markdown source's or the preview's. The ones [selection] has in use are
/// outlined; choosing a card picks its theme.
///
/// The syntax and preview rows begin with a "Same as theme" card, which
/// shows [sameAs], the interface theme in use, and picks no theme of its
/// own.
class InstalledThemeRow extends StatelessWidget {
  const InstalledThemeRow({
    required this.themes,
    required this.selection,
    this.kind = ThemeCardKind.ui,
    this.sameAs,
    super.key,
  });

  final List<ThemePreview> themes;
  final ThemeSelection selection;
  final ThemeCardKind kind;

  /// The interface theme in use, drawn by the "Same as theme" card; the row
  /// of the interface has none.
  final ThemePreview? sameAs;

  bool _current(ThemePreview theme) => switch (kind) {
    ThemeCardKind.ui => selection.isSelected(theme.id),
    ThemeCardKind.syntax => selection.syntax == theme.id,
    ThemeCardKind.preview => selection.preview == theme.id,
  };

  void _pick(BuildContext context, String? id, {bool dark = false}) {
    context.read<AppearanceBloc>().add(switch (kind) {
      ThemeCardKind.ui => AppearanceEvent.themePicked(id!, dark: dark),
      ThemeCardKind.syntax => AppearanceEvent.syntaxPicked(id),
      ThemeCardKind.preview => AppearanceEvent.previewPicked(id),
    });
  }

  @override
  Widget build(BuildContext context) {
    final same = sameAs;
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          if (kind != ThemeCardKind.ui && same != null) ...<Widget>[
            InstalledThemeCard.same(
              key: ValueKey<String>('same-${kind.name}'),
              sameAs: same,
              kind: kind,
              current: switch (kind) {
                ThemeCardKind.syntax => selection.syntax == null,
                _ => selection.preview == null,
              },
              onTap: () => _pick(context, null),
            ),
            const SizedBox(width: KansoSpacing.sm),
          ],
          for (final theme in themes) ...<Widget>[
            InstalledThemeCard(
              key: ValueKey<String>('${kind.name}-${theme.id}'),
              theme: theme,
              kind: kind,
              current: _current(theme),
              onTap: () => _pick(context, theme.id, dark: theme.dark),
            ),
            const SizedBox(width: KansoSpacing.sm),
          ],
        ],
      ),
    );
  }
}

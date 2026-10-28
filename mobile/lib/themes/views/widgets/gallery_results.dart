// Kanso — gallery_results.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../models/data/theme_preview.dart';
import '../../state/theme_gallery/theme_gallery_bloc.dart';
import '../../state/theme_picker/theme_picker_bloc.dart';
import 'gallery_theme_grid.dart';

/// What the gallery found for the search: its cards, or why there are none
/// (the gallery is empty, or nothing matches the words typed). It needs a
/// [ThemeGalleryBloc] and a [ThemePickerBloc] above it, the second for the
/// previews of the themes that are installed.
class GalleryResults extends StatelessWidget {
  const GalleryResults({required this.gallery, super.key});

  final ThemeGalleryState gallery;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    if (gallery.items.isEmpty) {
      return KansoEmptyState(
        title: l10n.galleryEmptyTitle,
        message: l10n.galleryEmptyMessage,
      );
    }
    final shown = gallery.shown;
    if (shown.isEmpty) {
      return KansoEmptyState(
        title: l10n.galleryNoResultsTitle,
        message: l10n.galleryNoResultsMessage(gallery.query.trim()),
      );
    }
    final bloc = context.read<ThemeGalleryBloc>();
    return BlocBuilder<ThemePickerBloc, ThemePickerState>(
      buildWhen: (before, after) => before.themes != after.themes,
      builder: (context, picker) {
        return GalleryThemeGrid(
          items: shown,
          installed: <String, ThemePreview>{
            for (final theme in picker.themes)
              if (!theme.builtIn) theme.id: theme,
          },
          working: gallery.working,
          onInstall: (id) => bloc.add(ThemeGalleryEvent.installed(id)),
          onRemove: (id) => bloc.add(ThemeGalleryEvent.uninstalled(id)),
        );
      },
    );
  }
}

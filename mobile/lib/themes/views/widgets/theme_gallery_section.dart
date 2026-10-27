// Kanso — theme_gallery_section.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_search_input.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../models/data/gallery_filter.dart';
import '../../state/theme_gallery/theme_gallery_bloc.dart';
import '../../state/theme_picker/theme_picker_bloc.dart';
import '../../utils/gallery_labels.dart';
import 'gallery_loading_grid.dart';
import 'gallery_results.dart';

/// The gallery on the theme picker: a search field, the two orders and the
/// community's themes as cards, each with Install, Update or Remove.
///
/// A theme installed or removed here changes the picker's installed themes
/// (the [ThemePickerBloc]); the app's appearance hears of it from the
/// gallery itself. It needs a [ThemeGalleryBloc] and a [ThemePickerBloc]
/// above it.
class ThemeGallerySection extends StatelessWidget {
  const ThemeGallerySection({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    return BlocListener<ThemeGalleryBloc, ThemeGalleryState>(
      listenWhen: (before, after) =>
          before.installedRevision != after.installedRevision &&
          after.changed.isNotEmpty,
      listener: (context, state) {
        context.read<ThemePickerBloc>().add(
          const ThemePickerEvent.installedChanged(),
        );
      },
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          KansoSectionHeader(label: l10n.galleryTitle),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                KansoSearchInput(
                  hint: l10n.gallerySearchHint,
                  clearLabel: l10n.gallerySearchClear,
                  onChanged: (query) => context.read<ThemeGalleryBloc>().add(
                    ThemeGalleryEvent.searched(query),
                  ),
                ),
                const SizedBox(height: KansoSpacing.xs),
                BlocBuilder<ThemeGalleryBloc, ThemeGalleryState>(
                  buildWhen: (before, after) => before.filter != after.filter,
                  builder: (context, state) {
                    return KansoSegmented<GalleryFilter>(
                      expanded: true,
                      semanticLabel: l10n.galleryFilterLabel,
                      selected: state.filter,
                      onChanged: (filter) => context
                          .read<ThemeGalleryBloc>()
                          .add(ThemeGalleryEvent.filterChosen(filter)),
                      segments: <KansoSegment<GalleryFilter>>[
                        for (final filter in GalleryFilter.values)
                          KansoSegment<GalleryFilter>(
                            value: filter,
                            label: l10n.galleryFilterName(filter),
                          ),
                      ],
                    );
                  },
                ),
                const SizedBox(height: KansoSpacing.sm),
                BlocBuilder<ThemeGalleryBloc, ThemeGalleryState>(
                  builder: (context, gallery) {
                    return switch (gallery.status) {
                      LoadStatus.loading => const GalleryLoadingGrid(),
                      LoadStatus.failed => Center(
                        child: KansoMessageCard(
                          title: l10n.galleryFailedTitle,
                          message: l10n.galleryFailedMessage,
                          actions: <Widget>[
                            KansoButton(
                              label: l10n.retry,
                              onPressed: () => context
                                  .read<ThemeGalleryBloc>()
                                  .add(const ThemeGalleryEvent.started()),
                            ),
                          ],
                        ),
                      ),
                      LoadStatus.ready => Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: <Widget>[
                          if (gallery.refreshing)
                            Padding(
                              padding: const EdgeInsets.only(
                                bottom: KansoSpacing.xs,
                              ),
                              child: Text(
                                l10n.galleryRefreshing,
                                style: typography.meta,
                              ),
                            ),
                          if (gallery.refreshFailed)
                            Row(
                              children: <Widget>[
                                Expanded(
                                  child: Text(
                                    l10n.galleryStale,
                                    style: typography.meta,
                                  ),
                                ),
                                KansoButton(
                                  label: l10n.retry,
                                  variant: KansoButtonVariant.quiet,
                                  expanded: false,
                                  compact: true,
                                  onPressed: () => context
                                      .read<ThemeGalleryBloc>()
                                      .add(const ThemeGalleryEvent.refreshed()),
                                ),
                              ],
                            ),
                          if (gallery.problem != null)
                            Padding(
                              padding: const EdgeInsets.only(
                                bottom: KansoSpacing.xs,
                              ),
                              child: Semantics(
                                liveRegion: true,
                                child: Text(
                                  l10n.galleryProblemText(gallery.problem!),
                                  style: typography.secondary.copyWith(
                                    color: colors.danger,
                                  ),
                                ),
                              ),
                            ),
                          GalleryResults(gallery: gallery),
                        ],
                      ),
                    };
                  },
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

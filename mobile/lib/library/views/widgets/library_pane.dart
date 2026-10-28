// Kanso — library_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/library_index_status.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/tag_row_color.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_dot.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_search_field.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';
import '../../../lock/utils/notebook_lock_rule.dart';
import '../../../sharing/views/widgets/shared_with_you_row.dart';
import 'library_header.dart';
import 'library_loading_rows.dart';
import 'library_place_row.dart';
import 'library_tasks_row.dart';
import 'new_notebook_sheet.dart';
import 'notebook_actions_sheet.dart';

/// The library: the app's mark and settings, a search field, then All
/// Notes, Templates (when there is a notebook of that role), Pinned,
/// Recent and Tasks (with the overdue ones counted), Shared with you (with
/// how many invitations wait), the notebooks with their counts, the tags
/// and Trash.
///
/// A drawer on a phone, the first pane on a tablet. The place [location]
/// is listing is selected. Its rows come from the `LibraryIndexBloc`
/// above it, and stay up to date as the library changes.
class LibraryPane extends StatelessWidget {
  const LibraryPane({
    required this.location,
    this.designGallery = false,
    super.key,
  });

  /// Where the person is; the row of its place is selected.
  final BrowseLocation? location;

  /// Whether to offer the design gallery.
  final bool designGallery;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final here = location;
    final current = here is PlaceLocation ? here.place : null;
    final bottom = MediaQuery.paddingOf(context).bottom;
    void leave(VoidCallback go) {
      // A drawer slides away as the page behind it changes.
      KansoDrawerScope.maybeOf(context)?.close();
      go();
    }

    return ColoredBox(
      color: context.colors.sidebar,
      child: Column(
        children: <Widget>[
          LibraryHeader(
            onSettings: () => leave(() => context.push(KansoRoutes.settings)),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              0,
              KansoSpacing.md,
              KansoSpacing.xs,
            ),
            child: KansoPressable(
              onTap: () => leave(
                () => context.go(const BrowseLocation.search().location),
              ),
              semanticLabel: l10n.searchHint,
              child: ExcludeSemantics(
                child: IgnorePointer(
                  child: KansoSearchField(hint: l10n.searchHint),
                ),
              ),
            ),
          ),
          Expanded(
            child: BlocBuilder<LibraryIndexBloc, LibraryIndexState>(
              builder: (context, index) {
                final templates = index.notebooks
                    .where((book) => book.role == NotebookRole.templates)
                    .firstOrNull;
                return switch (index.status) {
                  LibraryIndexStatus.loading => ListView(
                    physics: const NeverScrollableScrollPhysics(),
                    padding: const EdgeInsets.only(top: KansoSpacing.xxs),
                    children: const <Widget>[LibraryLoadingRows()],
                  ),
                  LibraryIndexStatus.failed => Center(
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.md),
                      child: KansoMessageCard(
                        icon: KansoIcons.notebook,
                        title: l10n.libraryFailedTitle,
                        message: l10n.libraryFailedMessage,
                        actions: <Widget>[
                          KansoButton(
                            label: l10n.retry,
                            onPressed: () => context
                                .read<LibraryIndexBloc>()
                                .add(const LibraryIndexEvent.retried()),
                          ),
                        ],
                      ),
                    ),
                  ),
                  LibraryIndexStatus.ready => ListView(
                    padding: EdgeInsets.only(
                      top: KansoSpacing.xxs,
                      bottom: bottom + KansoSpacing.md,
                    ),
                    children: <Widget>[
                      LibraryPlaceRow(
                        place: const ListPlace.all(),
                        icon: KansoIcons.files,
                        label: l10n.libraryAllNotes,
                        count: index.overview.all,
                        selected: current == const ListPlace.all(),
                      ),
                      if (templates != null)
                        LibraryPlaceRow(
                          place: ListPlace.notebook(id: templates.id),
                          icon: KansoIcons.layoutTemplate,
                          label: l10n.libraryTemplates,
                          selected:
                              current == ListPlace.notebook(id: templates.id),
                        ),
                      LibraryPlaceRow(
                        place: const ListPlace.pinned(),
                        icon: KansoIcons.pin,
                        label: l10n.libraryPinned,
                        count: index.overview.pinned,
                        selected: current == const ListPlace.pinned(),
                      ),
                      LibraryPlaceRow(
                        place: const ListPlace.recent(),
                        icon: KansoIcons.clock,
                        label: l10n.libraryRecent,
                        selected: current == const ListPlace.recent(),
                      ),
                      SharedWithYouRow(
                        onOpen: () =>
                            leave(() => context.push(KansoRoutes.shared)),
                      ),
                      LibraryTasksRow(
                        overdue: index.overdueTasks,
                        selected: here is TasksLocation,
                      ),
                      KansoSectionHeader(
                        label: l10n.libraryNotebooks,
                        trailing: <Widget>[
                          KansoIconButton(
                            icon: KansoIcons.plus,
                            semanticLabel: l10n.libraryNewNotebook,
                            iconSize: KansoSizes.iconSmall + 2,
                            onPressed: () => NewNotebookSheet.open(context),
                          ),
                        ],
                      ),
                      if (index.notebooks.every(
                        (book) => book.role == NotebookRole.templates,
                      ))
                        Padding(
                          padding: const EdgeInsets.symmetric(
                            horizontal: KansoSpacing.md,
                            vertical: KansoSpacing.xs,
                          ),
                          child: Text(
                            l10n.libraryNoNotebooks,
                            style: context.typography.secondary,
                          ),
                        ),
                      for (final notebook in index.notebooks)
                        if (notebook.role != NotebookRole.templates)
                          LibraryPlaceRow(
                            place: ListPlace.notebook(id: notebook.id),
                            icon: KansoIcons.notebook,
                            label: notebook.name,
                            count: notebook.count,
                            depth: notebook.depth,
                            chevron: true,
                            selected:
                                current == ListPlace.notebook(id: notebook.id),
                            locked:
                                notebookLockRule(notebook, index.notebooks) !=
                                NotebookLockRule.lock,
                            onLongPress: () =>
                                NotebookActionsSheet.show(context, notebook),
                          ),
                      const SizedBox(height: KansoSpacing.xs),
                      KansoSectionHeader(label: l10n.libraryTags),
                      if (index.tags.isEmpty)
                        Padding(
                          padding: const EdgeInsets.symmetric(
                            horizontal: KansoSpacing.md,
                            vertical: KansoSpacing.xs,
                          ),
                          child: Text(
                            l10n.libraryNoTags,
                            style: context.typography.secondary,
                          ),
                        ),
                      for (final tag in index.tags)
                        LibraryPlaceRow(
                          place: ListPlace.tag(id: tag.id),
                          leading: KansoDot(color: tag.tint),
                          label: tag.name,
                          count: tag.count,
                          selected: current == ListPlace.tag(id: tag.id),
                        ),
                      const SizedBox(height: KansoSpacing.xs),
                      LibraryPlaceRow(
                        place: const ListPlace.trash(),
                        icon: KansoIcons.trash,
                        label: l10n.libraryTrash,
                        count: index.overview.trash,
                        selected: current == const ListPlace.trash(),
                      ),
                      KansoSidebarRow(
                        icon: KansoIcons.replaceAll,
                        label: l10n.libraryReplace,
                        onTap: () => leave(
                          () => context.push(KansoRoutes.replace),
                        ),
                      ),
                      if (designGallery)
                        KansoSidebarRow(
                          icon: KansoIcons.palette,
                          label: l10n.libraryDesignGallery,
                          onTap: () =>
                              leave(() => context.push(KansoRoutes.design)),
                        ),
                    ],
                  ),
                };
              },
            ),
          ),
        ],
      ),
    );
  }
}

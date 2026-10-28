// Kanso — library_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/library_index_status.dart';
import '../../../core/models/data/note_drag_data.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/muted_tag_color.dart';
import '../../../core/utils/notebook_tint.dart';
import '../../../core/utils/status_label.dart';
import '../../../core/utils/tag_row_color.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_icon_set_scope.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_search_field.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sidebar_row.dart';
import '../../../core/views/widgets/kanso_status_glyph.dart';
import '../../../core/views/widgets/kanso_themed_icon.dart';
import '../../../injection.dart';
import '../../../lock/utils/notebook_lock_rule.dart';
import '../../../note_list/state/note_list/note_list_bloc.dart';
import '../../../palette/views/palette_launcher.dart';
import '../../../sharing/views/widgets/shared_with_you_row.dart';
import 'library_actions.dart';
import 'library_header.dart';
import 'library_loading_rows.dart';
import 'library_place_row.dart';
import 'library_tasks_row.dart';
import 'new_notebook_sheet.dart';
import 'notebook_actions_sheet.dart';
import 'tag_actions_sheet.dart';

/// The library: the app's mark and settings, a search field, then All
/// Notes, Templates (when there is a notebook of that role), Pinned,
/// Recent and Tasks (with the overdue ones counted), Shared with you (with
/// how many invitations wait), the notebooks with their counts, folded by
/// their arrows, the notes' Statuses, the tags as chips and a nested tree,
/// Trash, Graph and Replace.
///
/// A drawer on a phone, the first pane on a tablet. The place [location]
/// is listing is selected. Its rows come from the `LibraryIndexBloc`
/// above it, and stay up to date as the library changes.
class LibraryPane extends StatefulWidget {
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
  State<LibraryPane> createState() => _LibraryPaneState();
}

class _LibraryPaneState extends State<LibraryPane> {
  /// The place the tree is drawn around: the place of the list showing.
  ListPlace? get _place {
    final here = widget.location;
    return here is PlaceLocation ? here.place : null;
  }

  @override
  void initState() {
    super.initState();
    final place = _place;
    if (place != null) {
      // After the first frame: a bloc may not be changed while building.
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) {
          context.read<LibraryIndexBloc>().add(
            LibraryIndexEvent.placeChanged(place),
          );
        }
      });
    }
  }

  @override
  void didUpdateWidget(LibraryPane oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.location != oldWidget.location) {
      context.read<LibraryIndexBloc>().add(
        LibraryIndexEvent.placeChanged(_place),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final here = widget.location;
    final current = _place;
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
          KansoSettingsRow(
            icon: KansoIcons.terminal,
            label: l10n.paletteCommands,
            onTap: () => leave(() => unawaited(PaletteLauncher.open(context))),
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
                      if (_notebooks(index).isEmpty)
                        _EmptyNote(text: l10n.libraryNoNotebooks)
                      else
                        for (final notebook in _notebooks(index))
                          _notebookRow(context, index, current, notebook),
                      const SizedBox(height: KansoSpacing.xs),
                      _StatusSection(
                        index: index,
                        current: current,
                        onLeave: leave,
                      ),
                      const SizedBox(height: KansoSpacing.xs),
                      KansoSectionHeader(label: l10n.libraryTags),
                      _TagSection(
                        index: index,
                        current: current,
                        onLeave: leave,
                      ),
                      const SizedBox(height: KansoSpacing.xs),
                      LibraryPlaceRow(
                        place: const ListPlace.trash(),
                        icon: KansoIcons.trash,
                        label: l10n.libraryTrash,
                        count: index.overview.trash,
                        selected: current == const ListPlace.trash(),
                        onLongPress: () => _emptyTrash(context, index),
                      ),
                      KansoSidebarRow(
                        icon: KansoIcons.replaceAll,
                        label: l10n.libraryReplace,
                        onTap: () => leave(
                          () => context.push(KansoRoutes.replace),
                        ),
                      ),
                      if (widget.designGallery)
                        KansoSidebarRow(
                          icon: KansoIcons.palette,
                          label: l10n.libraryDesignGallery,
                          onTap: () =>
                              leave(() => context.push(KansoRoutes.design)),
                        ),
                      KansoSidebarRow(
                        icon: KansoIcons.share,
                        label: l10n.graphTitle,
                        onTap: () =>
                            leave(() => context.push(KansoRoutes.graph)),
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

  /// The notebook rows that show, without the Templates notebook.
  List<VisibleNotebook> _notebooks(
    LibraryIndexState index,
  ) => <VisibleNotebook>[
    for (final notebook in index.visibleNotebooks)
      if (notebook.row.role != NotebookRole.templates) notebook,
  ];

  Widget _notebookRow(
    BuildContext context,
    LibraryIndexState index,
    ListPlace? current,
    VisibleNotebook shown,
  ) {
    final notebook = shown.row;
    final place = ListPlace.notebook(id: notebook.id);
    final selected = current == place;
    final color = notebookColor(notebook.color);
    final dark = Theme.brightnessOf(context) == Brightness.dark;
    final icon = notebook.icon ?? KansoIcons.notebook.lucideName;
    return LibraryPlaceRow(
      place: place,
      label: notebook.name,
      count: notebook.count,
      depth: notebook.depth,
      leading: KansoThemedIcon(
        lucideName: icon,
        svg: KansoIconSetScope.of(context).svgFor(icon),
        color: selected
            ? context.colors.text
            : color == null
            ? context.colors.textSecondary
            : mutedTagColor(color, dark: dark),
      ),
      wash: color,
      foldOpen: shown.parent ? shown.open : null,
      onFold: shown.parent
          ? () => context.read<LibraryIndexBloc>().add(
              LibraryIndexEvent.notebookFolded(notebook.id),
            )
          : null,
      chevron: !shown.parent,
      selected: selected,
      onDropNotes: (ids) => _changeDraggedNotes(
        context,
        ids,
        NoteChange.move(notebookId: notebook.id),
      ),
      locked:
          notebookLockRule(notebook, index.notebooks) != NotebookLockRule.lock,
      onLongPress: () => NotebookActionsSheet.show(context, notebook),
    );
  }

  /// Empties the trash, as the Trash row's long press does on the desktop,
  /// after asking.
  Future<void> _emptyTrash(
    BuildContext context,
    LibraryIndexState index,
  ) async {
    if (index.overview.trash < 1) {
      return;
    }
    final l10n = context.l10n;
    final bloc = context.read<LibraryIndexBloc>();
    final host = Navigator.of(context, rootNavigator: true).overlay!.context;
    final yes = await KansoConfirmSheet.show(
      host,
      title: l10n.libraryEmptyTrashTitle(index.overview.trash),
      message: l10n.libraryEmptyTrashMessage,
      confirmLabel: l10n.libraryEmptyTrashConfirm,
    );
    if (!yes) {
      return;
    }
    await runLibraryAction(
      host,
      index: bloc,
      action: getIt<LibraryRepository>().emptyTrash,
    );
  }
}

void _changeDraggedNotes(
  BuildContext context,
  List<String> ids,
  NoteChange change,
) {
  if (ids.isEmpty) {
    return;
  }
  context.read<NoteListBloc>().add(NoteListEvent.changeRequested(change, ids));
}

/// A quiet line where a section has no rows.
class _EmptyNote extends StatelessWidget {
  const _EmptyNote({required this.text});

  final String text;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.md,
        vertical: KansoSpacing.xs,
      ),
      child: Text(text, style: context.typography.secondary),
    );
  }
}

/// The Status group: a header that opens and folds it, and a row for each
/// status the desktop lists, with how many notes have it.
class _StatusSection extends StatelessWidget {
  const _StatusSection({
    required this.index,
    required this.current,
    required this.onLeave,
  });

  final LibraryIndexState index;
  final ListPlace? current;
  final void Function(VoidCallback) onLeave;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<LibraryIndexBloc>();
    final open = index.statusOpen;
    // None is a status a note can be given, but the group lists the four
    // the desktop lists.
    final rows = <StatusChoice>[
      for (final choice in index.statuses)
        if (choice.status != 'none') choice,
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        KansoPressable(
          onTap: () => bloc.add(const LibraryIndexEvent.statusGroupToggled()),
          semanticLabel: l10n.libraryStatuses,
          child: KansoSectionHeader(
            label: l10n.libraryStatuses,
            trailing: <Widget>[
              KansoIcon(
                open ? KansoIcons.chevronDown : KansoIcons.chevronRight,
                size: KansoSizes.iconSmall,
                color: context.colors.textMuted,
              ),
            ],
          ),
        ),
        if (open)
          for (final choice in rows)
            LibraryPlaceRow(
              place: ListPlace.status(status: choice.status),
              leading: KansoStatusGlyph(
                status: noteStatusOf(choice.status),
              ),
              label: choice.label,
              count: index.statusCount(choice.status),
              selected: current == ListPlace.status(status: choice.status),
            ),
      ],
    );
  }
}

/// The tags: the chips of the tags with no tags inside them, a "…" or
/// "Fewer" toggle, and the nested tags as a tree folded by their arrows.
class _TagSection extends StatelessWidget {
  const _TagSection({
    required this.index,
    required this.current,
    required this.onLeave,
  });

  final LibraryIndexState index;
  final ListPlace? current;
  final void Function(VoidCallback) onLeave;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<LibraryIndexBloc>();
    final places = index.tagPlaces;
    if (index.tags.isEmpty) {
      return _EmptyNote(text: l10n.libraryNoTags);
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
          child: Wrap(
            spacing: KansoSpacing.xs,
            runSpacing: KansoSpacing.xxs,
            crossAxisAlignment: WrapCrossAlignment.center,
            children: <Widget>[
              for (final tag in places.chips)
                _TagChip(
                  tag: tag,
                  selected: current == ListPlace.tag(id: tag.id),
                  onDropNotes: (ids) => _changeDraggedNotes(
                    context,
                    ids,
                    NoteChange.addTag(tagId: tag.id),
                  ),
                  onTap: () => onLeave(
                    () => context.go(
                      BrowseLocation.place(ListPlace.tag(id: tag.id)).location,
                    ),
                  ),
                  onLongPress: () => TagActionsSheet.show(context, tag),
                ),
              if (places.more)
                KansoPressable(
                  onTap: () =>
                      bloc.add(const LibraryIndexEvent.allTagsToggled()),
                  semanticLabel: index.allTags
                      ? l10n.libraryTagsFewer
                      : l10n.libraryTagsMore,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: KansoSpacing.xs,
                      vertical: KansoSpacing.xxs,
                    ),
                    child: Text(
                      index.allTags ? l10n.libraryTagsFewer : '…',
                      style: context.typography.secondary,
                    ),
                  ),
                ),
            ],
          ),
        ),
        const SizedBox(height: KansoSpacing.xxs),
        for (final row in places.tree)
          LibraryPlaceRow(
            place: row.tagId == null
                ? ListPlace.tagPath(path: row.path)
                : ListPlace.tag(id: row.tagId!),
            leading: _Hash(),
            label: row.label,
            count: row.count,
            depth: row.depth,
            foldOpen: row.parent ? row.open : null,
            onFold: row.parent
                ? () => bloc.add(LibraryIndexEvent.tagToggled(row.path))
                : null,
            selected:
                current ==
                (row.tagId == null
                    ? ListPlace.tagPath(path: row.path)
                    : ListPlace.tag(id: row.tagId!)),
            onDropNotes: row.tagId == null
                ? null
                : (ids) => _changeDraggedNotes(
                    context,
                    ids,
                    NoteChange.addTag(tagId: row.tagId!),
                  ),
            onLongPress: () {
              final tag = row.tagId == null ? null : index.tag(row.tagId!);
              if (tag != null) {
                TagActionsSheet.show(context, tag);
              }
            },
          ),
      ],
    );
  }
}

/// The `#` before a tag's name in the tree.
class _Hash extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return Text(
      '#',
      style: context.typography.secondary.copyWith(
        color: context.colors.textMuted,
      ),
    );
  }
}

/// One chip of the tag list.
class _TagChip extends StatelessWidget {
  const _TagChip({
    required this.tag,
    required this.selected,
    required this.onTap,
    required this.onLongPress,
    required this.onDropNotes,
  });

  final TagRow tag;
  final bool selected;
  final VoidCallback onTap;
  final VoidCallback onLongPress;
  final ValueChanged<List<String>> onDropNotes;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    Widget chip({required bool hovering}) => Semantics(
      selected: selected,
      label: '#${tag.name}',
      child: KansoPressable(
        onTap: onTap,
        onLongPress: onLongPress,
        child: Container(
          height: KansoSizes.chip + 4,
          padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.xs),
          decoration: BoxDecoration(
            color: selected
                ? colors.sidebarActive
                : hovering
                ? Color.alphaBlend(
                    colors.accent.withValues(alpha: 0.16),
                    colors.sidebar,
                  )
                : null,
            border: Border.all(
              color: selected ? colors.sidebarActive : colors.divider,
            ),
            borderRadius: KansoRadii.smallBorder,
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              Text(
                '#',
                style: context.typography.secondary.copyWith(
                  color: mutedTagColor(
                    tag.tint,
                    dark: Theme.brightnessOf(context) == Brightness.dark,
                  ),
                ),
              ),
              const SizedBox(width: 2),
              Text(
                tag.name,
                style: context.typography.secondary.copyWith(
                  color: selected ? colors.text : colors.textSecondary,
                ),
              ),
              const SizedBox(width: KansoSpacing.xxs),
              Text(
                '${tag.count}',
                style: context.typography.secondary.copyWith(
                  color: colors.textMuted,
                ),
              ),
            ],
          ),
        ),
      ),
    );
    return DragTarget<NoteDragData>(
      onWillAcceptWithDetails: (_) => true,
      onAcceptWithDetails: (details) => onDropNotes(details.data.noteIds),
      builder: (context, hovering, rejected) =>
          chip(hovering: hovering.isNotEmpty),
    );
  }
}

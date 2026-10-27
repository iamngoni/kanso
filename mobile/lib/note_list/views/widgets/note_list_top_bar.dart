// Kanso — note_list_top_bar.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';

/// The bar above a list of notes: the title with a quiet line under it.
///
/// Where the library is a drawer it has the menu that opens it and the
/// button that opens search; with the library beside the list it has
/// neither. [onOptions] opens how the list is sorted and narrowed, and
/// [onNewNote], when the list has no button afloat over it, makes a new
/// note.
class NoteListTopBar extends StatelessWidget {
  const NoteListTopBar({
    required this.title,
    this.subtitle,
    this.onSearch,
    this.onOptions,
    this.onNewNote,
    super.key,
  });

  final String title;
  final String? subtitle;
  final VoidCallback? onSearch;
  final VoidCallback? onOptions;
  final VoidCallback? onNewNote;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final drawer = KansoDrawerScope.maybeOf(context);
    return KansoTopBar(
      backgroundColor: context.colors.list,
      leading: drawer == null
          ? null
          : KansoIconButton(
              icon: KansoIcons.menu,
              semanticLabel: l10n.openLibrary,
              onPressed: drawer.open,
            ),
      title: title,
      subtitle: subtitle,
      trailing: <Widget>[
        if (onNewNote != null)
          KansoIconButton(
            icon: KansoIcons.plus,
            semanticLabel: l10n.noteListNewNote,
            onPressed: onNewNote,
          ),
        if (drawer != null)
          KansoIconButton(
            icon: KansoIcons.search,
            semanticLabel: l10n.noteSearch,
            onPressed: onSearch,
          ),
        KansoIconButton(
          icon: KansoIcons.ellipsis,
          semanticLabel: l10n.noteListMore,
          onPressed: onOptions,
        ),
      ],
    );
  }
}

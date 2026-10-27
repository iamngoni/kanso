// Kanso — empty_place_pane.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';

/// A list with no notes in it: a quiet message that says what will appear
/// there. [name] is the title of the place, for the notebook's and the
/// tag's own words; [narrowed] says the notes are hidden by the list's
/// narrowing, not missing. An empty All Notes also offers to import notes
/// from another app.
class EmptyPlacePane extends StatelessWidget {
  const EmptyPlacePane({
    required this.place,
    required this.name,
    this.narrowed = false,
    super.key,
  });

  final ListPlace place;
  final String name;
  final bool narrowed;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final (title, message) = narrowed
        ? (l10n.emptyNarrowedTitle, l10n.emptyNarrowedMessage)
        : switch (place) {
            ListPlace_All() => (l10n.emptyAllTitle, l10n.emptyAllMessage),
            ListPlace_Pinned() => (
              l10n.emptyPinnedTitle,
              l10n.emptyPinnedMessage,
            ),
            ListPlace_Recent() => (
              l10n.emptyRecentTitle,
              l10n.emptyRecentMessage,
            ),
            ListPlace_Trash() => (l10n.emptyTrashTitle, l10n.emptyTrashMessage),
            ListPlace_Notebook() => (
              l10n.emptyNotebookTitle(name),
              l10n.emptyNotebookMessage,
            ),
            ListPlace_Tag() => (l10n.emptyTagTitle(name), l10n.emptyTagMessage),
            ListPlace_TagPath() => (
              l10n.emptyTagTitle(name),
              l10n.emptyTagMessage,
            ),
            ListPlace_Status() => (
              l10n.emptyStatusTitle(name),
              l10n.emptyStatusMessage,
            ),
          };
    final bringingNotesIn = !narrowed && place is ListPlace_All;
    return KansoEmptyState(
      title: title,
      message: message,
      action: bringingNotesIn
          ? KansoButton(
              label: l10n.importEmptyButton,
              icon: KansoIcons.download,
              variant: KansoButtonVariant.secondary,
              expanded: false,
              onPressed: () => context.push(KansoRoutes.import),
            )
          : null,
    );
  }
}

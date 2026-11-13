// Kanso — shared_notebooks_section.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_notice.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../injection.dart';
import '../../state/shared_notebooks/shared_notebooks_bloc.dart';
import '../../utils/book_labels.dart';
import 'book_avatars.dart';
import 'share_message_text.dart';
import 'share_notebook_sheet.dart';

/// The library's Shared section, the desktop sidebar's: the notebooks shared
/// here, each with its people's faces and whether it is up to date, and the
/// notebook invitations waiting, to accept or decline. Nothing at all while
/// there is neither (and signed out). It makes its own
/// [SharedNotebooksBloc].
class SharedNotebooksSection extends StatelessWidget {
  const SharedNotebooksSection({this.selectedNotebook, super.key});

  /// The notebook whose list shows, to mark its row.
  final String? selectedNotebook;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<SharedNotebooksBloc>(
      create: (_) =>
          getIt<SharedNotebooksBloc>()
            ..add(const SharedNotebooksEvent.started()),
      child: _SharedNotebooksView(selectedNotebook: selectedNotebook),
    );
  }
}

class _SharedNotebooksView extends StatelessWidget {
  const _SharedNotebooksView({required this.selectedNotebook});

  final String? selectedNotebook;

  void _open(BuildContext context, String notebookId) {
    KansoDrawerScope.maybeOf(context)?.close();
    context.go(
      BrowseLocation.place(ListPlace.notebook(id: notebookId)).location,
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocConsumer<SharedNotebooksBloc, SharedNotebooksState>(
      listenWhen: (before, after) => before.joined != after.joined,
      listener: (context, state) {
        final joined = state.joined;
        if (joined != null) {
          _open(context, joined);
        }
      },
      builder: (context, state) {
        if (state.books.isEmpty &&
            state.invitations.isEmpty &&
            state.notices.isEmpty) {
          return const SizedBox.shrink();
        }
        final bloc = context.read<SharedNotebooksBloc>();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            KansoSectionHeader(label: l10n.libraryShared),
            if (state.notices.isNotEmpty)
              Padding(
                padding: const EdgeInsets.fromLTRB(
                  KansoSpacing.sm,
                  0,
                  KansoSpacing.sm,
                  KansoSpacing.xs,
                ),
                child: KansoNotice(
                  message: state.notices.first,
                  dismissLabel: l10n.noteListDismiss,
                  onDismiss: () =>
                      bloc.add(const SharedNotebooksEvent.noticeRead()),
                ),
              ),
            for (final invitation in state.invitations)
              _InvitationTile(
                key: ValueKey<String>('book-invitation-${invitation.shareId}'),
                invitation: invitation,
                working: state.working.contains(invitation.shareId),
                onAccept: () => bloc.add(
                  SharedNotebooksEvent.accepted(invitation.shareId),
                ),
                onDecline: () => bloc.add(
                  SharedNotebooksEvent.declined(invitation.shareId),
                ),
              ),
            if (state.problem != null)
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.md,
                  vertical: KansoSpacing.xxs,
                ),
                child: ShareMessageText(problem: state.problem),
              ),
            for (final book in state.books)
              _SharedBookRow(
                key: ValueKey<String>('shared-book-${book.shareId}'),
                book: book,
                selected: book.notebookId == selectedNotebook,
                onTap: () => _open(context, book.notebookId),
                onLongPress: () =>
                    ShareNotebookSheet.show(context, book.notebookId),
              ),
          ],
        );
      },
    );
  }
}

/// A shared notebook: its name, its people's faces, and under the name
/// whether it is up to date ("Offline · 4 changes waiting").
class _SharedBookRow extends StatelessWidget {
  const _SharedBookRow({
    required this.book,
    required this.selected,
    required this.onTap,
    required this.onLongPress,
    super.key,
  });

  final SharedBookRow book;
  final bool selected;
  final VoidCallback onTap;
  final VoidCallback onLongPress;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final l10n = context.l10n;
    final status = book.status;
    final tone = status.offline
        ? colors.warning
        : status.waiting > 0
        ? colors.textMuted
        : colors.success;
    final line = bookStatusLabel(l10n, status);
    final background = selected ? colors.sidebarActive : colors.sidebar;
    return Semantics(
      button: true,
      selected: selected,
      label: <String>[
        book.name,
        l10n.sharedNotebookMembers(book.members.length),
        line,
      ].join(', '),
      child: KansoPressable(
        onTap: onTap,
        onLongPress: onLongPress,
        child: ExcludeSemantics(
          child: Container(
            margin: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.xs,
              vertical: 1,
            ),
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm,
              vertical: KansoSpacing.xs,
            ),
            decoration: BoxDecoration(
              color: background,
              borderRadius: KansoRadii.smallBorder,
            ),
            child: Row(
              children: <Widget>[
                KansoIcon(
                  KansoIcons.users,
                  size: 18,
                  color: selected ? colors.text : colors.textSecondary,
                ),
                const SizedBox(width: KansoSpacing.sm),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: <Widget>[
                      Text(
                        book.name,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: typography.body.copyWith(color: colors.text),
                      ),
                      const SizedBox(height: 1),
                      Row(
                        children: <Widget>[
                          Container(
                            width: 6,
                            height: 6,
                            decoration: BoxDecoration(
                              color: status.live ? colors.accent : tone,
                              shape: BoxShape.circle,
                            ),
                          ),
                          const SizedBox(width: KansoSpacing.xxs + 2),
                          Flexible(
                            child: Text(
                              line,
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: typography.meta,
                            ),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: KansoSpacing.xs),
                BookAvatars(names: book.members, ring: background),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// An invitation to a notebook: who it is from and the role, with Accept
/// (or Open here, for one another device joined) and Decline.
class _InvitationTile extends StatelessWidget {
  const _InvitationTile({
    required this.invitation,
    required this.working,
    required this.onAccept,
    required this.onDecline,
    super.key,
  });

  final BookInvitationRow invitation;
  final bool working;
  final VoidCallback onAccept;
  final VoidCallback onDecline;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final role = bookRoleLabel(l10n, invitation.role);
    return Container(
      margin: const EdgeInsets.fromLTRB(
        KansoSpacing.sm,
        0,
        KansoSpacing.sm,
        KansoSpacing.xs,
      ),
      padding: const EdgeInsets.all(KansoSpacing.sm),
      decoration: BoxDecoration(
        color: colors.elevated,
        borderRadius: KansoRadii.mediumBorder,
        border: Border.all(color: colors.divider),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Row(
            children: <Widget>[
              BookAvatars(
                names: <String>[invitation.from],
                size: 28,
                ring: colors.elevated,
              ),
              const SizedBox(width: KansoSpacing.sm),
              Expanded(
                child: Text(
                  invitation.invited
                      ? l10n.sharedNotebookInvitation(invitation.from, role)
                      : l10n.sharedNotebookElsewhere(invitation.from, role),
                  style: context.typography.secondary.copyWith(
                    color: colors.text,
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: KansoSpacing.sm),
          Row(
            children: <Widget>[
              if (invitation.invited) ...<Widget>[
                Expanded(
                  child: KansoButton(
                    label: l10n.sharedNotebookDecline,
                    variant: KansoButtonVariant.secondary,
                    compact: true,
                    onPressed: working ? null : onDecline,
                  ),
                ),
                const SizedBox(width: KansoSpacing.xs),
              ],
              Expanded(
                child: KansoButton(
                  label: invitation.invited
                      ? l10n.sharedNotebookAccept
                      : l10n.sharedNotebookOpenHere,
                  compact: true,
                  onPressed: working ? null : onAccept,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

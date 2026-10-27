// Kanso — comments_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../state/comments/comments_bloc.dart';
import 'comment_compose_sheet.dart';
import 'comment_tile.dart';

/// A shared note's comments, the desktop's Comments tab: those on words
/// still in the text, then the orphaned ones under their own heading, with
/// the count of open and resolved ones at the foot. Tapping a comment puts
/// the sheet away and runs [onSelect] with its ID, to select its words.
class CommentsSheet extends StatelessWidget {
  const CommentsSheet({required this.onSelect, super.key});

  final ValueChanged<String> onSelect;

  /// Shows the comments [bloc] holds over the app.
  static Future<void> show(
    BuildContext context, {
    required CommentsBloc bloc,
    required ValueChanged<String> onSelect,
  }) {
    bloc.add(const CommentsEvent.refreshed());
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => BlocProvider<CommentsBloc>.value(
        value: bloc,
        child: CommentsSheet(onSelect: onSelect),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    return BlocBuilder<CommentsBloc, CommentsState>(
      builder: (context, state) {
        final bloc = context.read<CommentsBloc>();
        Widget tile(CommentRow comment) => CommentTile(
          comment: comment,
          onSelect: () {
            Navigator.of(context).pop();
            onSelect(comment.id);
          },
          onResolve: (resolved) => bloc.add(
            CommentsEvent.resolved(commentId: comment.id, resolved: resolved),
          ),
          onEdit: () => CommentComposeSheet.show(
            context,
            title: l10n.commentEditTitle,
            action: l10n.commentSave,
            initial: comment.body,
            // A failure shows in the list, under its title.
            onSend: (words) async {
              bloc.add(
                CommentsEvent.edited(commentId: comment.id, body: words),
              );
              return null;
            },
          ),
          onDelete: () => bloc.add(CommentsEvent.deleted(comment.id)),
        );

        final error = state.error;
        final orphaned = state.orphaned;
        return Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.fromLTRB(
                KansoSpacing.md,
                KansoSpacing.sm,
                KansoSpacing.md,
                KansoSpacing.xs,
              ),
              child: Text(l10n.commentsTitle, style: typography.heading2),
            ),
            if (error != null)
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.md,
                  vertical: KansoSpacing.xs,
                ),
                child: KansoInlineMessage(message: error.message),
              ),
            if (state.status == LoadStatus.ready && !state.shared)
              KansoEmptyState(
                title: l10n.commentsNotShared,
                message: l10n.commentsNotSharedHint,
              )
            else if (state.status == LoadStatus.ready && state.comments.isEmpty)
              KansoEmptyState(
                title: l10n.commentsEmpty,
                message: state.mayComment
                    ? l10n.commentsEmptyHint
                    : l10n.commentsEmptyHintReader,
              )
            else
              Flexible(
                child: ListView(
                  shrinkWrap: true,
                  // The sheet is below the status bar: no room for it.
                  padding: EdgeInsets.zero,
                  children: <Widget>[
                    for (final comment in state.anchored) tile(comment),
                    if (orphaned.isNotEmpty) ...<Widget>[
                      Padding(
                        padding: const EdgeInsets.fromLTRB(
                          KansoSpacing.md,
                          KansoSpacing.sm,
                          KansoSpacing.md,
                          KansoSpacing.xxs,
                        ),
                        child: Text(
                          l10n.commentsOrphaned,
                          style: typography.sectionLabel,
                        ),
                      ),
                      for (final comment in orphaned) tile(comment),
                    ],
                  ],
                ),
              ),
            if (state.band.isNotEmpty) ...<Widget>[
              const KansoDivider(),
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.md,
                  vertical: KansoSpacing.xs,
                ),
                child: Text(state.band, style: typography.meta),
              ),
            ],
            const SizedBox(height: KansoSpacing.xs),
          ],
        );
      },
    );
  }
}

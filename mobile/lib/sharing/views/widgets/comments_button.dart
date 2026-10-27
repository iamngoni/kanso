// Kanso — comments_button.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../state/comments/comments_bloc.dart';
import 'comments_sheet.dart';

/// The button in a shared note's top bar that opens its comments, with how
/// many are open on it. A note that isn't shared has none, and no button.
/// It reads the `CommentsBloc` above it; [onSelect] selects a comment's
/// words in the editor.
class CommentsButton extends StatelessWidget {
  const CommentsButton({required this.onSelect, super.key});

  final ValueChanged<String> onSelect;

  @override
  Widget build(BuildContext context) {
    return BlocBuilder<CommentsBloc, CommentsState>(
      buildWhen: (before, after) =>
          before.shared != after.shared || before.open != after.open,
      builder: (context, state) {
        if (!state.shared) {
          return const SizedBox.shrink();
        }
        final colors = context.colors;
        final open = state.open;
        return Stack(
          clipBehavior: Clip.none,
          children: <Widget>[
            KansoIconButton(
              icon: KansoIcons.messageSquare,
              semanticLabel: context.l10n.commentsButton(open),
              onPressed: () => CommentsSheet.show(
                context,
                bloc: context.read<CommentsBloc>(),
                onSelect: onSelect,
              ),
            ),
            if (open > 0)
              PositionedDirectional(
                top: 6,
                end: 4,
                child: ExcludeSemantics(
                  child: Container(
                    constraints: const BoxConstraints(minWidth: 16),
                    padding: const EdgeInsets.symmetric(horizontal: 4),
                    decoration: BoxDecoration(
                      color: colors.accent,
                      borderRadius: KansoRadii.pillBorder,
                    ),
                    child: Text(
                      open > 99 ? '99+' : '$open',
                      textAlign: TextAlign.center,
                      style: context.typography.meta.copyWith(
                        color: colors.buttonText,
                        fontSize: 10,
                        height: 1.6,
                      ),
                    ),
                  ),
                ),
              ),
          ],
        );
      },
    );
  }
}

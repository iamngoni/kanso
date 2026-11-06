// Kanso — open_loops_page.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/repos/abs/digest_repository.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_list_scrollbar.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_swipe_host.dart';
import '../../../injection.dart';
import '../../utils/close_loop.dart';
import '../../utils/open_digest_note.dart';
import '../widgets/digest_ai_panel.dart';
import '../widgets/digest_loader.dart';
import '../widgets/loop_row.dart';

/// The open loops across the notes, in three groups: what I will do, what
/// I am waiting on and what I want to find out.
class OpenLoopsPage extends StatelessWidget {
  const OpenLoopsPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return DigestAiScope(
      child: DigestLoader<OpenLoopsBoard>(
        title: l10n.digestLoopsTitle,
        subtitle: (board) =>
            board.enabled ? l10n.digestLoopsCount(board.total) : null,
        load: getIt<DigestRepository>().openLoops,
        builder: (context, board, reload) {
          if (!board.enabled) {
            return KansoEmptyState(
              title: l10n.digestLoopsOffTitle,
              message: l10n.digestLoopsOffMessage,
            );
          }
          if (board.items.isEmpty) {
            return Column(
              children: <Widget>[
                Expanded(
                  child: KansoEmptyState(
                    title: l10n.digestLoopsEmptyTitle,
                    message: l10n.digestLoopsEmptyMessage,
                  ),
                ),
                const DigestAiPanel(find: true),
              ],
            );
          }
          return KansoSwipeHost(
            child: KansoListScrollbar(
              child: ListView(
                padding: digestListPadding,
                children: <Widget>[
                  for (final kind in OpenLoopKind.values)
                    ..._group(context, board, kind),
                  const DigestAiPanel(find: true),
                ],
              ),
            ),
          );
        },
      ),
    );
  }

  List<Widget> _group(
    BuildContext context,
    OpenLoopsBoard board,
    OpenLoopKind kind,
  ) {
    final items = board.items.where((item) => item.kind == kind).toList();
    if (items.isEmpty) {
      return const <Widget>[];
    }
    return <Widget>[
      KansoSectionHeader(label: loopKindLabel(context, kind)),
      for (final item in items)
        LoopRow(
          key: ValueKey<String>('${item.noteId}/${item.key}'),
          item: item,
          onAction: (action) => closeLoop(context, item, action),
          onOpen: () => openDigestNote(context, item.noteId, line: item.line),
        ),
    ];
  }
}

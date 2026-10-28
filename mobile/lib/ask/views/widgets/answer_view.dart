// Kanso — answer_view.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../models/data/answer_block.dart';
import '../../utils/parse_answer_blocks.dart';
import 'answer_block_view.dart';

/// An answer's Markdown, drawn: headings, paragraphs, lists, code,
/// quotations and rules, with a citation as a tappable chip that opens the
/// note.
///
/// It draws text and nothing else. An image is its description, a link to
/// anything but a note or a web page is plain text, and HTML is shown as
/// the characters it is made of.
class AnswerView extends StatelessWidget {
  const AnswerView({
    required this.markdown,
    required this.onOpenNote,
    required this.onOpenLink,
    required this.usedNoteIds,
    super.key,
  });

  final String markdown;

  /// Opens the note with this id.
  final ValueChanged<String> onOpenNote;

  /// Opens this web address.
  final ValueChanged<String> onOpenLink;
  final Set<String> usedNoteIds;

  @override
  Widget build(BuildContext context) {
    final blocks = parseAnswerBlocks(markdown);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        for (var i = 0; i < blocks.length; i++) ...<Widget>[
          if (i > 0)
            SizedBox(
              height: switch (blocks[i]) {
                AnswerHeading() => KansoSpacing.md,
                _ => KansoSpacing.sm,
              },
            ),
          AnswerBlockView(
            block: blocks[i],
            onOpenNote: onOpenNote,
            onOpenLink: onOpenLink,
            usedNoteIds: usedNoteIds,
          ),
        ],
      ],
    );
  }
}

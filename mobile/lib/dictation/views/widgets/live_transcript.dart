// Kanso — live_transcript.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/utils/context_extensions.dart';

/// What was heard so far: the finished phrases in the text colour and the
/// phrase being spoken, still changing, in the quiet one. Before anything is
/// heard it says where the words will appear. The newest words stay in view.
class LiveTranscript extends StatelessWidget {
  const LiveTranscript({
    required this.finished,
    required this.text,
    super.key,
  });

  /// The words that are final, as they are written.
  final String finished;

  /// Everything heard, as it is written: [finished] and what follows.
  final String text;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final style = context.typography.editorBody;
    // The rules may rewrite the end of the final words ("delete that"), so
    // what follows them is told by the prefix, not by the length.
    final settled = text.startsWith(finished) ? finished : '';
    final tail = text.substring(settled.length);
    if (text.isEmpty) {
      return Align(
        alignment: Alignment.topLeft,
        child: Text(
          context.l10n.dictationTranscriptEmpty,
          style: style.copyWith(color: colors.textMuted),
        ),
      );
    }
    return SingleChildScrollView(
      reverse: true,
      child: Align(
        alignment: Alignment.topLeft,
        child: Text.rich(
          TextSpan(
            children: <InlineSpan>[
              TextSpan(
                text: settled,
                style: style.copyWith(color: colors.text),
              ),
              TextSpan(
                text: tail,
                style: style.copyWith(color: colors.textSecondary),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

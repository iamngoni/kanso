// Kanso — comment_compose_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_text_input.dart';

/// A sheet to write a comment in: a new one on the selected words, or new
/// words for one of this account's. [onSend] sends the words and answers
/// with why they couldn't be sent, if they couldn't: the sheet says so and
/// stays; once they are sent it goes away.
class CommentComposeSheet extends StatefulWidget {
  const CommentComposeSheet({
    required this.title,
    required this.action,
    required this.onSend,
    this.initial = '',
    super.key,
  });

  /// "Comment on the selection" or "Edit comment".
  final String title;

  /// The button that sends: "Comment" or "Save".
  final String action;
  final Future<String?> Function(String words) onSend;
  final String initial;

  static Future<void> show(
    BuildContext context, {
    required String title,
    required String action,
    required Future<String?> Function(String words) onSend,
    String initial = '',
  }) {
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => CommentComposeSheet(
        title: title,
        action: action,
        onSend: onSend,
        initial: initial,
      ),
    );
  }

  @override
  State<CommentComposeSheet> createState() => _CommentComposeSheetState();
}

class _CommentComposeSheetState extends State<CommentComposeSheet> {
  late final TextEditingController words = TextEditingController(
    text: widget.initial,
  );
  final FocusNode focus = FocusNode(debugLabel: 'Comment');
  bool sending = false;
  String? problem;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        focus.requestFocus();
      }
    });
  }

  @override
  void dispose() {
    words.dispose();
    focus.dispose();
    super.dispose();
  }

  Future<void> send() async {
    final text = words.text.trim();
    if (text.isEmpty || sending) {
      return;
    }
    setState(() => sending = true);
    final failed = await widget.onSend(text);
    if (!mounted) {
      return;
    }
    if (failed == null) {
      Navigator.of(context).pop();
    } else {
      setState(() {
        sending = false;
        problem = failed;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    // The keyboard comes up under the sheet, which already has the home
    // indicator's room.
    final keyboard = math.max<double>(
      0,
      MediaQuery.viewInsetsOf(context).bottom -
          MediaQuery.paddingOf(context).bottom,
    );
    return Padding(
      padding: EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        KansoSpacing.md + keyboard,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Row(
            children: <Widget>[
              Expanded(
                child: Text(widget.title, style: context.typography.barTitle),
              ),
              KansoPressable(
                onTap: () => Navigator.of(context).maybePop(),
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.sm),
                  child: Text(
                    l10n.noteListCancel,
                    style: context.typography.body.copyWith(
                      color: colors.accent,
                    ),
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoTextInput(
            controller: words,
            focusNode: focus,
            hint: l10n.commentHint,
            semanticLabel: widget.title,
            textCapitalization: TextCapitalization.sentences,
            textInputAction: TextInputAction.send,
            onSubmitted: (_) => send(),
          ),
          if (problem case final problem?)
            Semantics(
              liveRegion: true,
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xs),
                child: Text(
                  problem,
                  style: context.typography.secondary.copyWith(
                    color: colors.danger,
                  ),
                ),
              ),
            )
          else
            const SizedBox(height: KansoSpacing.sm),
          ListenableBuilder(
            listenable: words,
            builder: (context, _) => KansoButton(
              label: widget.action,
              onPressed: words.text.trim().isEmpty || sending ? null : send,
            ),
          ),
        ],
      ),
    );
  }
}

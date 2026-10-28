// Kanso — ask_conversation.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../models/data/ask_phase.dart';
import '../../state/ask/ask_bloc.dart';
import 'ask_turn_view.dart';

/// The questions and answers so far, oldest first, kept scrolled to the end
/// while an answer is written.
class AskConversation extends StatefulWidget {
  const AskConversation({
    required this.state,
    required this.onOpenNote,
    required this.onOpenLink,
    required this.onRetry,
    this.usedExpanded = false,
    super.key,
  });

  final AskState state;
  final ValueChanged<String> onOpenNote;
  final ValueChanged<String> onOpenLink;
  final VoidCallback onRetry;

  /// Whether each list of notes used starts open.
  final bool usedExpanded;

  @override
  State<AskConversation> createState() => _AskConversationState();
}

class _AskConversationState extends State<AskConversation> {
  final ScrollController controller = ScrollController();

  @override
  void dispose() {
    controller.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(AskConversation oldWidget) {
    super.didUpdateWidget(oldWidget);
    final before = oldWidget.state.turns;
    final now = widget.state.turns;
    final grew =
        now.length != before.length ||
        (now.isNotEmpty &&
            before.isNotEmpty &&
            now.last.answer != before.last.answer);
    if (grew && widget.state.busy) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (controller.hasClients) {
          controller.jumpTo(controller.position.maxScrollExtent);
        }
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final turns = widget.state.turns;
    return ListView.separated(
      controller: controller,
      padding: const EdgeInsets.all(KansoSpacing.md),
      itemCount: turns.length,
      separatorBuilder: (context, index) =>
          const SizedBox(height: KansoSpacing.lg),
      itemBuilder: (context, index) {
        final last = index == turns.length - 1;
        return AskTurnView(
          turn: turns[index],
          phase: last ? widget.state.phase : AskPhase.idle,
          onOpenNote: widget.onOpenNote,
          onOpenLink: widget.onOpenLink,
          onRetry: last ? widget.onRetry : null,
          usedExpanded: widget.usedExpanded,
        );
      },
    );
  }
}

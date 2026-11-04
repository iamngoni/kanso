// Kanso — digest_ai_panel.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/ai_repository.dart';
import '../../../core/models/repos/abs/digest_repository.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/ai_trouble_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/sheet_header.dart';
import '../../../injection.dart';
import '../../state/digest_ai_cubit.dart';
import 'loop_row.dart';

/// How many days back "Find more open loops" reads (the desktop's).
const int digestFindDays = 14;

/// Gives the pages below it a [DigestAiCubit].
class DigestAiScope extends StatelessWidget {
  const DigestAiScope({required this.child, super.key});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<DigestAiCubit>(
      create: (_) =>
          DigestAiCubit(getIt<DigestRepository>(), getIt<AiRepository>())
            ..load(),
      child: BlocListener<DigestAiCubit, DigestAiState>(
        listenWhen: (before, after) =>
            before.found == null && after.found != null,
        listener: (context, state) =>
            unawaited(FoundLoopsSheet.show(context, context.read())),
        child: child,
      ),
    );
  }
}

/// The AI action of the open loops and what it says: "Find more open
/// loops…", what it found, and any failure. Nothing at all while AI is off
/// or not set up.
class DigestAiPanel extends StatelessWidget {
  const DigestAiPanel({
    this.find = false,
    super.key,
  });

  final bool find;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    return BlocBuilder<DigestAiCubit, DigestAiState>(
      builder: (context, state) {
        if (!state.ready) {
          return const SizedBox.shrink();
        }
        final cubit = context.read<DigestAiCubit>();
        final empty = state.empty;
        final kept = state.kept;
        return Padding(
          padding: const EdgeInsets.all(KansoSpacing.md),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              if (state.busy)
                Row(
                  children: <Widget>[
                    const KansoSpinner(),
                    const SizedBox(width: KansoSpacing.sm),
                    Expanded(
                      child: Text(
                        l10n.digestFinding,
                        style: typography.meta.copyWith(
                          color: colors.textMuted,
                        ),
                      ),
                    ),
                    KansoButton(
                      label: l10n.digestStop,
                      variant: KansoButtonVariant.quiet,
                      expanded: false,
                      compact: true,
                      onPressed: cubit.stop,
                    ),
                  ],
                )
              else ...<Widget>[
                if (find)
                  KansoButton(
                    label: l10n.digestFindLoops,
                    variant: KansoButtonVariant.secondary,
                    onPressed: () => unawaited(_ask(context, state.host)),
                  ),
              ],
              if (state.message case final message?)
                Padding(
                  padding: const EdgeInsets.only(top: KansoSpacing.sm),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: <Widget>[
                      KansoInlineMessage(message: aiTroubleText(message, l10n)),
                      const SizedBox(height: KansoSpacing.xs),
                      Wrap(
                        spacing: KansoSpacing.xs,
                        runSpacing: KansoSpacing.xs,
                        children: <Widget>[
                          KansoButton(
                            label: l10n.aiTroubleRetry,
                            variant: KansoButtonVariant.secondary,
                            expanded: false,
                            compact: true,
                            onPressed: () => unawaited(cubit.findLoops()),
                          ),
                          KansoButton(
                            label: l10n.aiTroubleSettings,
                            variant: KansoButtonVariant.quiet,
                            expanded: false,
                            compact: true,
                            onPressed: () =>
                                unawaited(context.push(KansoRoutes.settingsAi)),
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              if (empty != null)
                Padding(
                  padding: const EdgeInsets.only(top: KansoSpacing.sm),
                  child: Text(
                    <String>[
                      l10n.digestFoundNone,
                      ?empty.droppedText,
                    ].join(' '),
                    style: typography.meta.copyWith(color: colors.textMuted),
                  ),
                ),
              if (kept != null && kept > 0)
                Padding(
                  padding: const EdgeInsets.only(top: KansoSpacing.sm),
                  child: Text(
                    l10n.digestFoundKept(kept),
                    style: typography.meta.copyWith(color: colors.accent),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }

  /// Says what will be read and where it goes, then reads.
  Future<void> _ask(BuildContext context, String host) async {
    final cubit = context.read<DigestAiCubit>();
    final l10n = context.l10n;
    final yes = await KansoConfirmSheet.show(
      context,
      title: l10n.digestFindTitle,
      message: l10n.digestFindMessage(digestFindDays, host),
      confirmLabel: l10n.digestFindConfirm,
      destructive: false,
    );
    if (yes) {
      await cubit.findLoops();
    }
  }
}

String _label(BuildContext context, LoopProposal proposal) {
  final title = proposal.noteTitle.trim().isEmpty
      ? context.l10n.noteUntitled
      : proposal.noteTitle;
  return '${proposal.text}\n${loopKindLabel(context, proposal.kind)} · $title';
}

/// What "Find more open loops" found: each proposal with a tick, the note
/// it is from, and "Keep selected". Nothing is saved until then.
class FoundLoopsSheet extends StatelessWidget {
  const FoundLoopsSheet({super.key});

  static Future<void> show(BuildContext context, DigestAiCubit cubit) async {
    await showKansoBottomSheet<void>(
      context,
      builder: (_) => BlocProvider<DigestAiCubit>.value(
        value: cubit,
        child: const FoundLoopsSheet(),
      ),
    );
    // Closed any other way than Keep selected: nothing is kept.
    if (cubit.state.found != null) {
      cubit.discardFound();
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    return BlocBuilder<DigestAiCubit, DigestAiState>(
      builder: (context, state) {
        final scan = state.found;
        if (scan == null) {
          return const SizedBox.shrink();
        }
        return Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            SheetHeader(
              title: l10n.digestFoundTitle,
              actionLabel: l10n.searchCancel,
            ),
            Flexible(
              child: ListView(
                shrinkWrap: true,
                children: <Widget>[
                  for (final (index, proposal) in scan.proposals.indexed)
                    KansoCheckRow(
                      key: ValueKey<String>('found-$index'),
                      label: _label(context, proposal),
                      checked: state.picked.contains(index),
                      onTap: () => context.read<DigestAiCubit>().toggle(index),
                    ),
                ],
              ),
            ),
            if (<String>[?scan.coverage, ?scan.droppedText] case final lines
                when lines.isNotEmpty)
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.md,
                  vertical: KansoSpacing.xs,
                ),
                child: Text(
                  lines.join('. '),
                  style: typography.meta.copyWith(color: colors.textMuted),
                ),
              ),
            Padding(
              padding: const EdgeInsets.all(KansoSpacing.md),
              child: KansoButton(
                label: l10n.digestFoundKeep,
                onPressed: state.picked.isEmpty
                    ? null
                    : () {
                        unawaited(context.read<DigestAiCubit>().keepSelected());
                        Navigator.of(context).pop();
                      },
              ),
            ),
          ],
        );
      },
    );
  }
}

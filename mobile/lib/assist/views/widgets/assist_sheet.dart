// Kanso — assist_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/assist_action.dart';
import '../../../core/models/data/assist_action_info.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../models/data/assist_placement.dart';
import '../../models/data/assist_status.dart';
import '../../state/assist/assist_bloc.dart';
import 'assist_action_list.dart';

/// The assistant: the actions to choose from, then the suggestion as it is
/// written, to copy, to take or to throw away. It reads the `AssistBloc`
/// above it.
///
/// [selection] and [before] are the text it works on and the text just
/// before it. [onApply] puts a suggestion in the note and says whether it
/// did; it is null where there is nothing to put it in (a summary read from
/// the note's menu), and then the suggestion can only be copied. With
/// [summarizeNote] the sheet starts at once on the note's summary.
class AssistSheet extends StatelessWidget {
  const AssistSheet({
    required this.noteId,
    required this.selection,
    required this.hasSelection,
    required this.before,
    this.applyChanges,
    this.canApply,
    this.onApply,
    this.summarizeNote = false,
    super.key,
  });

  final String noteId;
  final String selection;

  /// Whether [selection] is text the person selected, and not the text
  /// before a caret.
  final bool hasSelection;
  final String before;
  final Listenable? applyChanges;
  final bool Function()? canApply;
  final bool Function(AssistPlacement placement, String suggestion)? onApply;
  final bool summarizeNote;

  @override
  Widget build(BuildContext context) {
    final changes = applyChanges;
    if (changes == null) {
      return buildSheet(context);
    }
    return AnimatedBuilder(
      animation: changes,
      builder: (context, _) => buildSheet(context),
    );
  }

  Widget buildSheet(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final bloc = context.read<AssistBloc>();
    return BlocBuilder<AssistBloc, AssistState>(
      builder: (context, state) {
        if (state.status == AssistStatus.idle && !summarizeNote) {
          return AssistActionList(
            actions: bloc.actions,
            hasSelection: hasSelection,
            hasText: selection.isNotEmpty,
            onChoose: (action) => bloc.add(
              AssistEvent.requested(
                noteId: noteId,
                action: action,
                before: before,
                selection: selection,
              ),
            ),
            onSummarizeNote: () =>
                bloc.add(AssistEvent.summaryRequested(noteId)),
          );
        }
        final info = infoFor(bloc.actions, state.action);
        final replaces = (info?.replaces ?? false) && !state.wholeNote;
        final title = state.wholeNote
            ? l10n.assistSummary
            : (info?.noun ?? l10n.assistTitle);
        final running =
            state.status == AssistStatus.running ||
            (summarizeNote && state.status == AssistStatus.idle);
        final failed = state.status == AssistStatus.failed;
        final applyCurrent = canApply?.call() ?? true;
        // The card says one thing at a time, beside its title.
        final status = switch (state.status) {
          _ when state.changed || !applyCurrent => l10n.assistChanged,
          AssistStatus.keptOut => l10n.assistKeptOut,
          AssistStatus.stopped =>
            replaces ? l10n.assistStopped : l10n.askStopped,
          AssistStatus.failed =>
            state.failure ??
                (state.suggestion.isEmpty
                    ? l10n.assistNothing
                    : l10n.askCutOff),
          _ when running && state.suggestion.isEmpty => l10n.assistThinking,
          _ => null,
        };
        final acceptable =
            state.status == AssistStatus.ready ||
            (state.status == AssistStatus.stopped &&
                !replaces &&
                state.suggestion.isNotEmpty);
        final placement = replaces
            ? AssistPlacement.replace
            : AssistPlacement.insertBelow;
        return Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.md,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              Row(
                children: <Widget>[
                  KansoIcon(
                    KansoIcons.sparkles,
                    size: KansoSizes.iconSmall,
                    color: colors.textMuted,
                  ),
                  const SizedBox(width: KansoSpacing.xs),
                  Text(title, style: typography.heading2),
                  if (status != null) ...<Widget>[
                    const SizedBox(width: KansoSpacing.xs),
                    Expanded(
                      child: Semantics(
                        liveRegion: true,
                        child: Text(
                          status,
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                          style: typography.secondary.copyWith(
                            color: failed
                                ? colors.danger
                                : colors.textSecondary,
                          ),
                        ),
                      ),
                    ),
                  ],
                ],
              ),
              if (state.suggestion.isNotEmpty) ...<Widget>[
                const SizedBox(height: KansoSpacing.sm),
                Flexible(
                  child: SingleChildScrollView(
                    child: Text(state.suggestion, style: typography.body),
                  ),
                ),
              ],
              const SizedBox(height: KansoSpacing.md),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: <Widget>[
                  if (running)
                    KansoButton(
                      label: l10n.assistStop,
                      variant: KansoButtonVariant.quiet,
                      expanded: false,
                      compact: true,
                      onPressed: () => bloc.add(const AssistEvent.stopped()),
                    ),
                  KansoButton(
                    label: state.status == AssistStatus.keptOut
                        ? l10n.assistClose
                        : l10n.assistDiscard,
                    variant: KansoButtonVariant.quiet,
                    expanded: false,
                    compact: true,
                    onPressed: () => discard(context, bloc),
                  ),
                  // Nothing to put the text in (a summary read from the
                  // note's menu): it can only be copied.
                  if (onApply == null && acceptable) ...<Widget>[
                    const SizedBox(width: KansoSpacing.xs),
                    KansoButton(
                      label: l10n.assistCopy,
                      expanded: false,
                      compact: true,
                      onPressed: () => Clipboard.setData(
                        ClipboardData(text: state.suggestion),
                      ),
                    ),
                  ] else if (onApply != null &&
                      state.status != AssistStatus.keptOut) ...<Widget>[
                    const SizedBox(width: KansoSpacing.xs),
                    KansoButton(
                      label: l10n.assistAccept,
                      expanded: false,
                      compact: true,
                      onPressed: acceptable && applyCurrent
                          ? () {
                              if (onApply!(placement, state.suggestion)) {
                                Navigator.of(context).maybePop();
                              } else {
                                bloc.add(const AssistEvent.changeNoticed());
                              }
                            }
                          : null,
                    ),
                  ],
                ],
              ),
            ],
          ),
        );
      },
    );
  }

  AssistActionInfo? infoFor(
    List<AssistActionInfo> actions,
    AssistAction? action,
  ) {
    for (final info in actions) {
      if (info.action == action) {
        return info;
      }
    }
    return null;
  }

  /// Back to the actions, or, for a summary or where there was nothing to
  /// choose, away.
  void discard(BuildContext context, AssistBloc bloc) {
    if (summarizeNote) {
      Navigator.of(context).maybePop();
    } else {
      bloc.add(const AssistEvent.discarded());
    }
  }
}

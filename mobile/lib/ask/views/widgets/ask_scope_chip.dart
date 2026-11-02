// Kanso — ask_scope_chip.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/ask_scope.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../state/ask/ask_bloc.dart';

/// What the next question searches, as a chip over the question field: "All
/// notes", a notebook or a tag. A tap opens the sheet that changes it.
class AskScopeChip extends StatelessWidget {
  const AskScopeChip({required this.scope, required this.enabled, super.key});

  final AskScope scope;
  final bool enabled;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final label = scope.isNarrow ? scope.label : l10n.askAllNotes;
    return KansoPressable(
      onTap: enabled ? () => showAskScopeSheet(context) : null,
      semanticLabel: '${l10n.askScopeTitle}: $label',
      child: Container(
        constraints: const BoxConstraints(minHeight: KansoSizes.touchTarget),
        padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.sm),
        alignment: AlignmentDirectional.centerStart,
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: colors.fill,
            borderRadius: KansoRadii.largeBorder,
          ),
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm,
              vertical: KansoSpacing.xxs,
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                Flexible(
                  child: Text(
                    label,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: context.typography.secondary,
                  ),
                ),
                const SizedBox(width: KansoSpacing.xxs),
                KansoIcon(
                  KansoIcons.chevronDown,
                  size: KansoSizes.iconSmall,
                  color: colors.textMuted,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// Opens the sheet that chooses what a question searches. The sheet is above
/// the page's navigator, so the [AskBloc] is handed to it.
Future<void> showAskScopeSheet(BuildContext context) {
  final bloc = context.read<AskBloc>()..add(const AskEvent.scopesRequested());
  return showKansoBottomSheet<void>(
    context,
    builder: (context) => BlocProvider<AskBloc>.value(
      value: bloc,
      child: const AskScopeSheet(),
    ),
  );
}

/// The choices: all notes, each notebook that is not locked, each tag.
class AskScopeSheet extends StatelessWidget {
  const AskScopeSheet({super.key});

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final l10n = context.l10n;
    return BlocBuilder<AskBloc, AskState>(
      builder: (context, state) {
        final choices = state.scopes.isEmpty
            ? <AskScope>[AskScope(label: l10n.askAllNotes)]
            : state.scopes;
        return Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.all(KansoSpacing.md),
              child: Text(l10n.askScopeTitle, style: typography.heading2),
            ),
            Flexible(
              child: ListView(
                shrinkWrap: true,
                children: <Widget>[
                  for (final choice in choices) ...<Widget>[
                    const KansoDivider(indent: KansoSpacing.md),
                    KansoPressable(
                      onTap: () {
                        context.read<AskBloc>().add(
                          AskEvent.scopeChosen(choice),
                        );
                        Navigator.of(context).maybePop();
                      },
                      child: ConstrainedBox(
                        constraints: const BoxConstraints(
                          minHeight: KansoSizes.touchTarget,
                        ),
                        child: Padding(
                          padding: const EdgeInsets.symmetric(
                            horizontal: KansoSpacing.md,
                          ),
                          child: Row(
                            children: <Widget>[
                              Expanded(
                                child: Text(
                                  choice.isNarrow
                                      ? choice.label
                                      : l10n.askAllNotes,
                                  style: typography.body,
                                ),
                              ),
                              if (choice.kind == state.scope.kind &&
                                  choice.value == state.scope.value)
                                KansoIcon(
                                  KansoIcons.check,
                                  color: colors.accent,
                                ),
                            ],
                          ),
                        ),
                      ),
                    ),
                  ],
                  const SizedBox(height: KansoSpacing.md),
                ],
              ),
            ),
          ],
        );
      },
    );
  }
}

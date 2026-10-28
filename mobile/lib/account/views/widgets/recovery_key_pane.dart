// Kanso — recovery_key_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/utils/account_error_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../../core/views/widgets/kanso_code_display.dart';
import '../../../core/views/widgets/kanso_copy_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';
import '../../state/recovery_key/recovery_key_bloc.dart';
import 'account_pane_frame.dart';

/// The recovery key, large, to write down: what it is for, a button to copy
/// it, the box "I've saved my recovery key somewhere safe" and Done, which
/// is pressed only once the box is checked. It needs a [RecoveryKeyBloc]
/// above it.
class RecoveryKeyPane extends StatelessWidget {
  const RecoveryKeyPane({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<RecoveryKeyBloc>();
    return AccountPaneFrame(
      title: l10n.accountRecoveryKeyTitle,
      leading: const SizedBox.shrink(),
      children: <Widget>[
        BlocBuilder<RecoveryKeyBloc, RecoveryKeyState>(
          builder: (context, state) {
            final key = state.key;
            return switch (state.status) {
              LoadStatus.loading => Semantics(
                label: l10n.accountLoading,
                child: const Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: <Widget>[
                    KansoSkeleton(height: 60),
                    SizedBox(height: KansoSpacing.md),
                    KansoSkeleton(height: 96),
                  ],
                ),
              ),
              LoadStatus.failed => Center(
                child: KansoMessageCard(
                  title: l10n.accountRecoveryKeyFailedTitle,
                  message: l10n.accountRecoveryKeyFailedMessage,
                  actions: <Widget>[
                    KansoButton(
                      label: l10n.accountRetry,
                      onPressed: () =>
                          bloc.add(const RecoveryKeyEvent.started()),
                    ),
                  ],
                ),
              ),
              LoadStatus.ready when key == null => KansoEmptyState(
                title: l10n.accountRecoveryKeyNoneTitle,
                message: l10n.accountRecoveryKeyNoneMessage,
                action: KansoButton(
                  label: l10n.accountDone,
                  expanded: false,
                  onPressed: () => bloc.add(const RecoveryKeyEvent.done()),
                ),
              ),
              LoadStatus.ready => Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: <Widget>[
                  Text(
                    l10n.accountRecoveryKeyText,
                    style: context.typography.secondary,
                  ),
                  const SizedBox(height: KansoSpacing.md),
                  KansoCodeDisplay(text: key!.key, fontSize: 22),
                  const SizedBox(height: KansoSpacing.sm),
                  KansoCopyButton(
                    text: key.key,
                    label: l10n.accountRecoveryKeyCopy,
                  ),
                  const SizedBox(height: KansoSpacing.md),
                  KansoPressable(
                    onTap: () => bloc.add(
                      RecoveryKeyEvent.confirmedChanged(
                        confirmed: !state.confirmed,
                      ),
                    ),
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        vertical: KansoSpacing.xs,
                      ),
                      child: Row(
                        children: <Widget>[
                          IgnorePointer(
                            child: KansoCheckbox(value: state.confirmed),
                          ),
                          const SizedBox(width: KansoSpacing.sm),
                          Expanded(
                            child: Text(
                              l10n.accountRecoveryKeyConfirm,
                              style: context.typography.body,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                  const SizedBox(height: KansoSpacing.md),
                  if (state.error != null) ...<Widget>[
                    KansoInlineMessage(
                      message: l10n.accountError(state.error!),
                    ),
                    const SizedBox(height: KansoSpacing.sm),
                  ],
                  KansoButton(
                    label: l10n.accountDone,
                    onPressed: state.confirmed && !state.saving
                        ? () => bloc.add(const RecoveryKeyEvent.done())
                        : null,
                  ),
                ],
              ),
            };
          },
        ),
      ],
    );
  }
}

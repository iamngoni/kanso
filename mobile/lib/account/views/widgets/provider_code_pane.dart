// Kanso — provider_code_pane.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_code_display.dart';
import '../../../core/views/widgets/kanso_copy_button.dart';
import '../../models/data/provider_code.dart';
import '../../state/sign_in/sign_in_bloc.dart';
import 'account_pane_frame.dart';

/// The code GitHub asks for, large, to type at its page: a button to copy
/// it, a button to open GitHub again, a quiet line that the flow waits, and
/// Cancel. It needs a [SignInBloc] above it.
class ProviderCodePane extends StatelessWidget {
  const ProviderCodePane({required this.code, super.key});

  final ProviderCode code;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<SignInBloc>();
    return AccountPaneFrame(
      title: l10n.accountGithubTitle,
      leading: const SizedBox.shrink(),
      children: <Widget>[
        Text(l10n.accountGithubExplain, style: context.typography.body),
        const SizedBox(height: KansoSpacing.md),
        KansoCodeDisplay(
          text: code.userCode,
          semanticLabel: l10n.accountApprovalCode(
            code.userCode.split('').join(' '),
          ),
        ),
        const SizedBox(height: KansoSpacing.sm),
        KansoCopyButton(text: code.userCode, label: l10n.accountCopy),
        const SizedBox(height: KansoSpacing.xs),
        KansoButton(
          label: l10n.accountGithubOpen,
          onPressed: () => bloc.add(const SignInEvent.githubOpened()),
        ),
        const SizedBox(height: KansoSpacing.md),
        Text(
          l10n.accountGithubWaiting,
          textAlign: TextAlign.center,
          style: context.typography.meta,
        ),
        const SizedBox(height: KansoSpacing.md),
        KansoButton(
          label: l10n.accountCancel,
          variant: KansoButtonVariant.quiet,
          onPressed: () => bloc.add(const SignInEvent.providerCancelled()),
        ),
      ],
    );
  }
}

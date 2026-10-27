// Kanso — account_linking_card.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_card.dart';
import '../../../core/views/widgets/kanso_code_display.dart';
import '../../../core/views/widgets/kanso_copy_button.dart';
import '../../models/data/provider_code.dart';
import '../../state/account_details/account_details_bloc.dart';
import '../../utils/account_labels.dart';

/// What the Account page shows while it links a provider: which one, the
/// code to type when it is GitHub, and Cancel. It needs an
/// [AccountDetailsBloc] above it.
class AccountLinkingCard extends StatelessWidget {
  const AccountLinkingCard({required this.provider, this.code, super.key});

  final ProviderKind provider;
  final ProviderCode? code;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final typography = context.typography;
    final shown = code;
    return KansoCard(
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(
            l10n.accountLinkTitle(l10n.providerName(provider)),
            style: typography.heading2,
          ),
          const SizedBox(height: KansoSpacing.xs),
          Text(
            shown == null
                ? l10n.accountProviderWaiting
                : l10n.accountGithubExplain,
            style: typography.secondary,
          ),
          if (shown != null) ...<Widget>[
            const SizedBox(height: KansoSpacing.md),
            KansoCodeDisplay(text: shown.userCode, fontSize: 28),
            const SizedBox(height: KansoSpacing.sm),
            KansoCopyButton(text: shown.userCode),
          ],
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: l10n.accountCancel,
            variant: KansoButtonVariant.quiet,
            onPressed: () => context.read<AccountDetailsBloc>().add(
              const AccountDetailsEvent.providerLinkCancelled(),
            ),
          ),
        ],
      ),
    );
  }
}

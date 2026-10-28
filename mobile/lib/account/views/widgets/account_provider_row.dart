// Kanso — account_provider_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_detail_row.dart';
import '../../utils/account_labels.dart';

/// Apple, Google or GitHub as a way into the account: whether it is linked,
/// and Link or Unlink, which [onPressed] carries out.
class AccountProviderRow extends StatelessWidget {
  const AccountProviderRow({
    required this.provider,
    required this.linked,
    this.onPressed,
    super.key,
  });

  final ProviderKind provider;
  final bool linked;

  /// Links or unlinks; null while a change is being made.
  final VoidCallback? onPressed;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoDetailRow(
      icon: provider == ProviderKind.github
          ? KansoIcons.github
          : KansoIcons.link,
      title: l10n.providerName(provider),
      subtitle: linked
          ? l10n.accountProviderLinked
          : l10n.accountProviderNotLinked,
      trailing: KansoButton(
        label: linked ? l10n.accountProviderUnlink : l10n.accountProviderLink,
        variant: KansoButtonVariant.quiet,
        expanded: false,
        compact: true,
        onPressed: onPressed,
      ),
    );
  }
}

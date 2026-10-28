// Kanso — account_passkey_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_detail_row.dart';
import '../../utils/account_labels.dart';

/// A passkey of the account: its name, when it was added and last used,
/// whether it can unlock a new device, and Remove, which [onRemove] carries
/// out.
class AccountPasskeyRow extends StatelessWidget {
  const AccountPasskeyRow({required this.passkey, this.onRemove, super.key});

  final PasskeyRow passkey;

  /// Removes the passkey; null while a change is being made.
  final VoidCallback? onRemove;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final used = passkey.lastUsedMs;
    final added = accountDate(context, passkey.addedMs);
    final when = used == null
        ? l10n.accountPasskeyAdded(added)
        : l10n.accountPasskeyUsed(added, accountDate(context, used));
    final unlock = passkey.canUnlock
        ? l10n.accountPasskeyCanUnlock
        : l10n.accountPasskeySignsInOnly;
    return KansoDetailRow(
      icon: KansoIcons.fingerprint,
      title: passkey.label,
      subtitle: '$when\n$unlock',
      trailing: KansoButton(
        label: l10n.accountPasskeyRemove,
        variant: KansoButtonVariant.quiet,
        expanded: false,
        compact: true,
        onPressed: onRemove,
      ),
    );
  }
}

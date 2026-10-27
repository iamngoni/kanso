// Kanso — account_device_row.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_detail_row.dart';
import '../../utils/account_labels.dart';

/// A device of the account: its name, "This device" or when it was last
/// seen, and for the others a Sign out that [onSignOut] carries out.
class AccountDeviceRow extends StatelessWidget {
  const AccountDeviceRow({required this.device, this.onSignOut, super.key});

  final DeviceRow device;

  /// Signs the device out; null while a change is being made.
  final VoidCallback? onSignOut;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final seen = device.lastSeenMs;
    return KansoDetailRow(
      icon: KansoIcons.monitor,
      title: device.title,
      subtitle: device.thisDevice
          ? l10n.accountThisDevice
          : seen == null
          ? null
          : l10n.accountLastSeen(accountDate(context, seen)),
      trailing: device.thisDevice
          ? null
          : KansoButton(
              label: l10n.accountDeviceSignOut,
              variant: KansoButtonVariant.quiet,
              expanded: false,
              compact: true,
              onPressed: onSignOut,
            ),
    );
  }
}

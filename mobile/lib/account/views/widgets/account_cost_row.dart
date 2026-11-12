// Kanso — account_cost_row.dart
// Created 2026-11-12.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_detail_row.dart';

/// Opens [uri] outside the app.
typedef OpenLink = Future<void> Function(Uri uri);

Future<void> _launch(Uri uri) async {
  try {
    await launchUrl(uri, mode: LaunchMode.externalApplication);
  } on Object {
    // The row still reads if no browser can open the Price page.
  }
}

/// "What Kanso costs": Free, the sync space used, and a link to the Price
/// page. The words and the numbers are the library's ([cost]), so the phone
/// says what the desktop says.
class AccountCostRow extends StatelessWidget {
  const AccountCostRow({required this.cost, this.onOpen = _launch, super.key});

  final CostSummary cost;

  /// Opens the Price page; the system browser by default.
  final OpenLink onOpen;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoDetailRow(
      icon: KansoIcons.coins,
      title: l10n.accountCost,
      subtitle: '${cost.headline}\n${cost.spaceLine}',
      trailing: KansoButton(
        label: l10n.accountCostLearnMore,
        variant: KansoButtonVariant.quiet,
        expanded: false,
        compact: true,
        onPressed: () => onOpen(Uri.parse(cost.learnMoreUrl)),
      ),
    );
  }
}

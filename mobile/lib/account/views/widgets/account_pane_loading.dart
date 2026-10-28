// Kanso — account_pane_loading.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';

/// Stands where the account's rows will be while it is read.
class AccountPaneLoading extends StatelessWidget {
  const AccountPaneLoading({super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: context.l10n.accountLoading,
      child: const Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          KansoSkeleton(height: 22, width: 160),
          SizedBox(height: KansoSpacing.xs),
          KansoSkeleton(width: 120),
          SizedBox(height: KansoSpacing.lg),
          KansoSkeleton(height: 96),
          SizedBox(height: KansoSpacing.lg),
          KansoSkeleton(height: 144),
        ],
      ),
    );
  }
}

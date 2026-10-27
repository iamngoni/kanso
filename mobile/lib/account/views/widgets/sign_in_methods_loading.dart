// Kanso — sign_in_methods_loading.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_skeleton.dart';

/// Stands where the ways in will be while the server is asked.
class SignInMethodsLoading extends StatelessWidget {
  const SignInMethodsLoading({super.key});

  @override
  Widget build(BuildContext context) {
    return Semantics(
      label: context.l10n.accountServerChecking,
      child: const Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          KansoSkeleton(height: 48),
          SizedBox(height: KansoSpacing.xs),
          KansoSkeleton(height: 48),
          SizedBox(height: KansoSpacing.lg),
          KansoSkeleton(height: 44),
          SizedBox(height: KansoSpacing.sm),
          KansoSkeleton(height: 44),
        ],
      ),
    );
  }
}

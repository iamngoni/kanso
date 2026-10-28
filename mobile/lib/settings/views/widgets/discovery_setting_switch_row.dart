// Kanso — discovery_setting_switch_row.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_switch.dart';

/// A Discovery preference with its explanation and the app's standard switch.
class DiscoverySettingSwitchRow extends StatelessWidget {
  const DiscoverySettingSwitchRow({
    required this.title,
    required this.detail,
    required this.value,
    required this.onChanged,
    super.key,
  });

  final String title;
  final String detail;
  final bool value;
  final ValueChanged<bool>? onChanged;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    return Padding(
      padding: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.md,
        vertical: KansoSpacing.sm,
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                Text(title, style: typography.body),
                const SizedBox(height: KansoSpacing.xxs),
                Text(
                  detail,
                  style: typography.secondary.copyWith(
                    color: colors.textSecondary,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: KansoSpacing.md),
          KansoSwitch(value: value, onChanged: onChanged),
        ],
      ),
    );
  }
}

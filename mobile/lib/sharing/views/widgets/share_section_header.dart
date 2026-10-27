// Kanso — share_section_header.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';

/// The label above a group of the share sheet: People, Asked to join,
/// Invite links. It has no indent of its own: the sheet's content has one.
class ShareSectionHeader extends StatelessWidget {
  const ShareSectionHeader({required this.label, super.key});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.only(
        top: KansoSpacing.md,
        bottom: KansoSpacing.xxs,
      ),
      child: Semantics(
        header: true,
        child: Text(label, style: context.typography.sectionLabel),
      ),
    );
  }
}

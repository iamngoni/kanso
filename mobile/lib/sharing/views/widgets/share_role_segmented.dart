// Kanso — share_role_segmented.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../models/data/share_choices.dart';
import '../../utils/share_labels.dart';

/// The three roles a person can be given (view, comment, edit) as one
/// choice, for an invitation or a link.
class ShareRoleSegmented extends StatelessWidget {
  const ShareRoleSegmented({
    required this.selected,
    required this.onChanged,
    required this.semanticLabel,
    super.key,
  });

  final ShareRole selected;
  final ValueChanged<ShareRole> onChanged;
  final String semanticLabel;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return KansoSegmented<ShareRole>(
      expanded: true,
      semanticLabel: semanticLabel,
      selected: selected,
      onChanged: onChanged,
      segments: <KansoSegment<ShareRole>>[
        for (final role in shareableRoles)
          KansoSegment<ShareRole>(value: role, label: l10n.shareRoleName(role)),
      ],
    );
  }
}

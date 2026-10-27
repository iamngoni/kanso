// Kanso — library_header.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_mark.dart';

/// The top of the library: the app's mark and name, and the way to
/// settings.
class LibraryHeader extends StatelessWidget {
  const LibraryHeader({required this.onSettings, super.key});

  final VoidCallback onSettings;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return SafeArea(
      bottom: false,
      child: SizedBox(
        height: KansoSizes.barHeight,
        child: Padding(
          padding: const EdgeInsetsDirectional.only(
            start: KansoSpacing.md,
            end: KansoSpacing.xxs,
          ),
          child: Row(
            children: <Widget>[
              const KansoMark(),
              const SizedBox(width: KansoSpacing.sm),
              Expanded(
                child: Text(l10n.appName, style: context.typography.barTitle),
              ),
              KansoIconButton(
                icon: KansoIcons.settings,
                semanticLabel: l10n.openSettings,
                onPressed: onSettings,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// Kanso — accessory_strip.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:kanso/core/configs/kanso_icons.dart';
import 'package:kanso/core/configs/kanso_sizes.dart';
import 'package:kanso/core/configs/kanso_spacing.dart';
import 'package:kanso/core/utils/context_extensions.dart';
import 'package:kanso/core/views/widgets/kanso_divider.dart';
import 'package:kanso/core/views/widgets/kanso_icon.dart';
import 'package:kanso/dictation/views/widgets/dictation_pill.dart';
import 'package:kanso/editor_extras/models/data/editor_accessory_extra.dart';

/// The editor's accessory bar as step 6 draws it (a key is a 44 point
/// square with a 20 point icon in the secondary colour), with dictation's key
/// [extra] last, and the pill above it while [pill] is given. The real bar
/// is the editor's; this is what dictation's key and pill look like in it.
class AccessoryStrip extends StatelessWidget {
  const AccessoryStrip({required this.extra, this.pill, super.key});

  final EditorAccessoryExtra extra;
  final DictationPill? pill;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final lit = extra.active?.value ?? false;
    final keys = <KansoIcons>[
      KansoIcons.hash,
      KansoIcons.list,
      KansoIcons.listChecks,
      KansoIcons.bold,
      KansoIcons.link,
      KansoIcons.image,
      KansoIcons.undo,
    ];
    return ColoredBox(
      color: colors.editor,
      child: Column(
        mainAxisAlignment: MainAxisAlignment.end,
        children: <Widget>[
          if (pill != null)
            Padding(
              padding: const EdgeInsets.all(KansoSpacing.sm),
              child: pill,
            ),
          ColoredBox(
            color: colors.sidebar,
            child: Column(
              children: <Widget>[
                const KansoDivider(),
                SizedBox(
                  height: KansoSizes.touchTarget,
                  child: Row(
                    children: <Widget>[
                      for (final key in keys)
                        Expanded(
                          child: Center(
                            child: KansoIcon(key, color: colors.textSecondary),
                          ),
                        ),
                      Expanded(
                        child: Center(
                          child: Container(
                            width: 36,
                            height: 36,
                            alignment: Alignment.center,
                            decoration: BoxDecoration(
                              borderRadius: BorderRadius.circular(8),
                              color: lit
                                  ? colors.accent.withValues(alpha: 0.18)
                                  : null,
                            ),
                            child: KansoIcon(
                              extra.icon,
                              color: lit ? colors.accent : colors.textSecondary,
                            ),
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

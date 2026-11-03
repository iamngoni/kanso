// Kanso — home_receipts.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../ask/utils/open_cited_note.dart';
import '../../../core/bridge/api/people_types.dart';
import '../../../core/configs/kanso_radii.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_pressable.dart';

/// The lines an AI sentence came from as small numbered chips, `[1] [2]`;
/// each opens its note at those lines. [first] is the number of the first.
class HomeReceipts extends StatelessWidget {
  const HomeReceipts({required this.sources, this.first = 1, super.key});

  final List<PageSource> sources;
  final int first;

  @override
  Widget build(BuildContext context) {
    if (sources.isEmpty) {
      return const SizedBox.shrink();
    }
    final colors = context.colors;
    return Wrap(
      spacing: KansoSpacing.xxs,
      runSpacing: KansoSpacing.xxs,
      children: <Widget>[
        for (var i = 0; i < sources.length; i++)
          KansoPressable(
            semanticLabel: sources[i].label,
            onTap: () => context.push(
              citedLinesLocation(
                sources[i].noteId,
                sources[i].start,
                sources[i].end,
              ),
            ),
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: colors.fill,
                borderRadius: KansoRadii.smallBorder,
              ),
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.xs,
                  vertical: 2,
                ),
                child: Text(
                  '[${first + i}]',
                  style: context.typography.meta.copyWith(color: colors.link),
                ),
              ),
            ),
          ),
      ],
    );
  }
}

// Kanso — meaning_search_results.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_divider.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';

/// Semantic matches above the ordinary text search results.
class MeaningSearchResults extends StatelessWidget {
  const MeaningSearchResults({required this.rows, super.key});

  final List<RelatedRow> rows;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return ConstrainedBox(
      constraints: BoxConstraints(
        maxHeight: MediaQuery.sizeOf(context).height * 0.38,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              KansoSpacing.sm,
              KansoSpacing.md,
              KansoSpacing.xs,
            ),
            child: Text(
              l10n.searchMeaningTitle,
              style: context.typography.sectionLabel,
            ),
          ),
          Flexible(
            child: ListView.separated(
              shrinkWrap: true,
              itemCount: rows.length,
              separatorBuilder: (context, index) =>
                  const KansoDivider(indent: KansoSpacing.md),
              itemBuilder: (context, index) {
                final row = rows[index];
                return KansoSettingsRow(
                  label: row.title,
                  value: row.label,
                  chevron: true,
                  onTap: () => GoRouter.of(
                    context,
                  ).push('${KansoRoutes.noteById}/${row.id}'),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

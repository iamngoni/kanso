// Kanso — review_stats_view.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/cards_types.dart';
import '../../../core/bridge/api/library.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/kanso_error_mapping.dart';
import '../../../injection.dart';
import '../../utils/flashcards_text.dart';

/// The stats of the cards in [scope]: counts, reviews a day, retention, the
/// next week's forecast and the hardest cards. Worked out on this device
/// from the log.
class ReviewStatsView extends StatefulWidget {
  const ReviewStatsView({required this.scope, super.key});

  final CardScopeArg scope;

  @override
  State<ReviewStatsView> createState() => _ReviewStatsViewState();
}

class _ReviewStatsViewState extends State<ReviewStatsView> {
  late final Future<ReviewStats> _stats = getIt<KansoLibrary>().cards().stats(
    scope: widget.scope,
    utcOffsetMinutes: DateTime.now().timeZoneOffset.inMinutes,
  );

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<ReviewStats>(
      future: _stats,
      builder: (context, snapshot) {
        if (snapshot.hasError) {
          return Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: Text(applicationErrorOf(snapshot.error!).message),
          );
        }
        final stats = snapshot.data;
        if (stats == null) {
          return const Center(child: CircularProgressIndicator());
        }
        return _StatsBody(stats: stats);
      },
    );
  }
}

class _StatsBody extends StatelessWidget {
  const _StatsBody({required this.stats});

  final ReviewStats stats;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final type = context.typography;
    final counts = stats.counts;
    final retention = stats.retention;
    return ListView(
      padding: const EdgeInsets.all(KansoSpacing.md),
      children: <Widget>[
        Row(
          children: <Widget>[
            _Figure(label: FlashcardsText.statsTotal, value: counts.total),
            _Figure(label: FlashcardsText.statsDue, value: counts.due),
            _Figure(label: FlashcardsText.statsNew, value: counts.newCards),
            _Figure(
              label: FlashcardsText.statsSuspended,
              value: counts.suspended,
            ),
          ],
        ),
        const SizedBox(height: KansoSpacing.lg),
        Text(FlashcardsText.statsPerDay, style: type.sectionLabel),
        const SizedBox(height: KansoSpacing.xs),
        _Bars(values: stats.perDay, color: colors.accent),
        const SizedBox(height: KansoSpacing.lg),
        Text(FlashcardsText.statsRetention, style: type.sectionLabel),
        const SizedBox(height: KansoSpacing.xs),
        Text(
          retention == null
              ? FlashcardsText.statsNone
              : '${(retention * 100).round()}%',
          style: type.heading1,
        ),
        const SizedBox(height: KansoSpacing.lg),
        Text(FlashcardsText.statsForecast, style: type.sectionLabel),
        const SizedBox(height: KansoSpacing.xs),
        _Bars(values: stats.forecast, color: colors.success),
        if (stats.hardest.isNotEmpty) ...<Widget>[
          const SizedBox(height: KansoSpacing.lg),
          Text(FlashcardsText.statsHardest, style: type.sectionLabel),
          for (final card in stats.hardest)
            ListTile(
              contentPadding: EdgeInsets.zero,
              title: Text(card.question, style: type.body),
              subtitle: Text(
                card.noteTitle,
                style: type.meta.copyWith(color: colors.textSecondary),
              ),
              trailing: Text('×${card.agains}', style: type.label),
              onTap: () => context.push(
                '${KansoRoutes.noteById}/${Uri.encodeComponent(card.noteId)}',
              ),
            ),
        ],
      ],
    );
  }
}

class _Figure extends StatelessWidget {
  const _Figure({required this.label, required this.value});

  final String label;
  final int value;

  @override
  Widget build(BuildContext context) {
    final type = context.typography;
    return Expanded(
      child: Column(
        children: <Widget>[
          Text('$value', style: type.heading1),
          Text(
            label,
            style: type.meta.copyWith(color: context.colors.textSecondary),
            textAlign: TextAlign.center,
          ),
        ],
      ),
    );
  }
}

class _Bars extends StatelessWidget {
  const _Bars({required this.values, required this.color});

  final List<int> values;
  final Color color;

  @override
  Widget build(BuildContext context) {
    final top = values.fold<int>(0, math.max);
    return SizedBox(
      height: 72,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: <Widget>[
          for (final v in values)
            Expanded(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 1),
                child: FractionallySizedBox(
                  alignment: Alignment.bottomCenter,
                  heightFactor: top == 0 ? 0.02 : math.max(0.02, v / top),
                  child: DecoratedBox(
                    decoration: BoxDecoration(
                      color: v == 0 ? color.withValues(alpha: 0.25) : color,
                      borderRadius: BorderRadius.circular(2),
                    ),
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

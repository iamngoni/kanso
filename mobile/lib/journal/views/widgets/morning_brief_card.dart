// Kanso — morning_brief_card.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/digest_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/debounce.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../injection.dart';

/// The id of the Journal note of the day [time] falls on, as the core
/// names it (`note:daily-YYYY-MM-DD`).
String dailyNoteIdOf(DateTime time) {
  final local = time.toLocal();
  return 'note:daily-'
      '${local.year.toString().padLeft(4, '0')}-'
      '${local.month.toString().padLeft(2, '0')}-'
      '${local.day.toString().padLeft(2, '0')}';
}

/// "Your morning brief": a line over today's Journal note with what is due
/// and how many loops are open, that opens the brief. Nothing over any other
/// note, over a day that isn't today, or with the brief turned off in
/// Settings. One line that gives way (the words are cut, never the card), so
/// it fits the narrowest pane.
class MorningBriefCard extends StatefulWidget {
  const MorningBriefCard({required this.noteId, super.key});

  /// The note open under the card.
  final String noteId;

  @override
  State<MorningBriefCard> createState() => _MorningBriefCardState();
}

class _MorningBriefCardState extends State<MorningBriefCard> {
  BriefView? brief;
  StreamSubscription<List<String>>? changes;
  int generation = 0;

  bool get isToday => widget.noteId == dailyNoteIdOf(clock.now());

  /// Whether the core's brief is there to ask (a page built without the
  /// library, as some tests do, shows no card).
  bool get available =>
      getIt.isRegistered<DigestRepository>() &&
      getIt.isRegistered<LibraryRepository>();

  @override
  void initState() {
    super.initState();
    if (!available) {
      return;
    }
    changes = debounceStream(
      getIt<LibraryRepository>().changes(),
      const Duration(milliseconds: 400),
    ).listen((_) => unawaited(read()));
    unawaited(read());
  }

  @override
  void didUpdateWidget(MorningBriefCard oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.noteId != widget.noteId) {
      unawaited(read());
    }
  }

  @override
  void dispose() {
    unawaited(changes?.cancel());
    super.dispose();
  }

  Future<void> read() async {
    final mine = ++generation;
    if (!available || !isToday) {
      if (brief != null && mounted) {
        setState(() => brief = null);
      }
      return;
    }
    final digest = getIt<DigestRepository>();
    final prefs = (await digest.prefs()).fold((_) => null, (it) => it);
    final result = prefs != null && prefs.brief
        ? (await digest.brief()).fold((_) => null, (it) => it)
        : null;
    if (mounted && mine == generation) {
      setState(() => brief = result);
    }
  }

  @override
  Widget build(BuildContext context) {
    final shown = brief;
    if (!isToday || shown == null) {
      return const SizedBox.shrink();
    }
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    return KansoPressable(
      onTap: () => unawaited(context.push(KansoRoutes.brief)),
      child: Semantics(
        button: true,
        child: Container(
          decoration: BoxDecoration(
            border: Border(bottom: BorderSide(color: colors.divider)),
          ),
          padding: const EdgeInsets.symmetric(
            horizontal: KansoSpacing.md,
            vertical: KansoSpacing.sm,
          ),
          child: Row(
            children: <Widget>[
              KansoIcon(KansoIcons.sun, color: colors.accent),
              const SizedBox(width: KansoSpacing.sm),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  mainAxisSize: MainAxisSize.min,
                  children: <Widget>[
                    Text(
                      l10n.digestMorningBrief,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: typography.body.copyWith(color: colors.text),
                    ),
                    Text(
                      l10n.digestMorningBriefCounts(
                        shown.due.length,
                        shown.loopsTotal,
                      ),
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: typography.meta.copyWith(
                        color: colors.textMuted,
                      ),
                    ),
                  ],
                ),
              ),
              KansoIcon(KansoIcons.chevronRight, color: colors.textMuted),
            ],
          ),
        ),
      ),
    );
  }
}

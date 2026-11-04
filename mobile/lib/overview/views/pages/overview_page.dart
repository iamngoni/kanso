// Kanso — overview_page.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/overview_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/ai_repository.dart';
import '../../../core/models/repos/abs/digest_repository.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_menu.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../../core/views/widgets/kanso_swipe_host.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../core/views/widgets/kanso_undo_notice.dart';
import '../../../digest/utils/open_digest_note.dart';
import '../../../injection.dart';
import '../../state/overview_cubit.dart';
import '../widgets/overview_ai_block.dart';
import '../widgets/overview_card.dart';
import '../widgets/overview_rows.dart';
import '../widgets/revisit_sheet.dart';

/// Overview, Kanso's home: Today and This week. It draws the last Today kept
/// at once, works the page out again off the UI isolate, and shows only the
/// sections that have something real in them, so a quiet day is a short page.
class OverviewPage extends StatelessWidget {
  const OverviewPage({this.week = false, super.key});

  /// Open on This week.
  final bool week;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<OverviewCubit>(
      create: (_) =>
          OverviewCubit(
            getIt<DigestRepository>(),
            getIt<LibraryRepository>(),
            getIt<AiRepository>(),
          )..load(
            perspective: week
                ? OverviewPerspective.week
                : OverviewPerspective.today,
          ),
      child: const _OverviewView(),
    );
  }
}

class _OverviewView extends StatelessWidget {
  const _OverviewView();

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return BlocConsumer<OverviewCubit, OverviewViewState>(
      listenWhen: (before, after) =>
          before.notice != after.notice && after.notice != null,
      listener: (context, state) {
        final day = state.notice;
        if (day == null) {
          return;
        }
        ScaffoldMessenger.maybeOf(
          context,
        )?.showSnackBar(SnackBar(content: Text(l10n.overviewRevisitSet(day))));
        context.read<OverviewCubit>().clearNotice();
      },
      builder: (context, state) {
        final cubit = context.read<OverviewCubit>();
        final week = state.perspective == OverviewPerspective.week;
        return KansoPage(
          backgroundColor: colors.app,
          body: Stack(
            children: <Widget>[
              Column(
                children: <Widget>[
                  KansoTopBar(
                    title: l10n.overviewTitle,
                    backgroundColor: colors.app,
                    bordered: false,
                    leading: KansoIconButton(
                      icon: KansoIcons.files,
                      semanticLabel: l10n.overviewNotes,
                      onPressed: () => context.go(KansoRoutes.notes),
                    ),
                    trailing: <Widget>[
                      if (state.aiReady)
                        KansoIconButton(
                          icon: KansoIcons.sparkles,
                          semanticLabel: l10n.overviewSummarise,
                          color: colors.accent,
                          onPressed: state.aiWorking ? null : cubit.summarise,
                        ),
                      Builder(
                        builder: (context) => KansoIconButton(
                          icon: KansoIcons.ellipsis,
                          semanticLabel: l10n.overviewMenu,
                          onPressed: () => unawaited(_menu(context, week)),
                        ),
                      ),
                    ],
                  ),
                  Expanded(child: _Body(state: state)),
                ],
              ),
              if (state.undo case final undo?)
                Positioned(
                  left: KansoSpacing.md,
                  right: KansoSpacing.md,
                  bottom: KansoSpacing.md,
                  child: SafeArea(
                    child: KansoUndoNotice(
                      message: l10n.overviewTicked(undo.title),
                      onUndo: () => unawaited(cubit.undoTick()),
                    ),
                  ),
                ),
            ],
          ),
        );
      },
    );
  }

  Future<void> _menu(BuildContext context, bool week) async {
    final l10n = context.l10n;
    final box = context.findRenderObject() as RenderBox?;
    final at = box == null
        ? Offset.zero
        : box.localToGlobal(Offset(box.size.width - 200, box.size.height));
    final cubit = context.read<OverviewCubit>();
    final router = GoRouter.of(context);
    final chosen = await showKansoMenu<String>(
      context,
      at,
      <KansoMenuEntry<String>>[
        KansoMenuEntry<String>('today', l10n.overviewOpenTodayNote),
        if (week) KansoMenuEntry<String>('save', l10n.overviewSaveWeek),
        KansoMenuEntry<String>('loops', l10n.digestLoopsTitle),
        KansoMenuEntry<String>('settings', l10n.overviewSettings),
      ],
    );
    switch (chosen) {
      case 'today':
        unawaited(router.push<void>(KansoRoutes.daily));
      case 'save':
        final from = cubit.state.week?.from;
        if (from != null) {
          final saved = await getIt<DigestRepository>().saveReview(
            from: from,
            replace: true,
          );
          saved.fold(
            (_) {},
            (it) => unawaited(router.push<void>(KansoRoutes.notes)),
          );
          unawaited(cubit.refresh());
        }
      case 'loops':
        unawaited(router.push<void>(KansoRoutes.loops));
      case 'settings':
        unawaited(router.push<void>(KansoRoutes.settings));
    }
  }
}

class _Body extends StatelessWidget {
  const _Body({required this.state});

  final OverviewViewState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final cubit = context.read<OverviewCubit>();
    final today = state.today;
    final week = state.week;
    final isWeek = state.perspective == OverviewPerspective.week;
    if (isWeek ? week == null : today == null) {
      if (state.failed) {
        return _withSwitch(
          KansoEmptyState(
            title: l10n.digestError,
            message: '',
            action: KansoButton(
              label: l10n.digestRetry,
              expanded: false,
              onPressed: () => unawaited(cubit.refresh()),
            ),
          ),
        );
      }
      return _withSwitch(const Center(child: KansoSpinner()));
    }
    return LayoutBuilder(
      builder: (context, box) => KansoSwipeHost(
        child: ListView(
          padding: overviewPagePadding(
            box.maxWidth,
          ).add(const EdgeInsets.only(bottom: overviewBottomSpace)),
          children: isWeek
              ? _weekChildren(context, state, week!)
              : _todayChildren(context, state, today!),
        ),
      ),
    );
  }
}

// MARK: Shared

/// The page with the Today | This week switch above it, for when the
/// masthead (which carries the switch) isn't there yet.
Widget _withSwitch(Widget child) => Column(
  crossAxisAlignment: CrossAxisAlignment.stretch,
  children: <Widget>[
    const Padding(
      padding: EdgeInsets.fromLTRB(
        KansoSpacing.lg,
        KansoSpacing.md,
        KansoSpacing.lg,
        0,
      ),
      child: _PerspectiveSwitch(),
    ),
    Expanded(child: child),
  ],
);

/// The Today | This week switch. It sits under the date and greeting.
class _PerspectiveSwitch extends StatelessWidget {
  const _PerspectiveSwitch();

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final cubit = context.read<OverviewCubit>();
    final perspective = context.select<OverviewCubit, OverviewPerspective>(
      (it) => it.state.perspective,
    );
    return KansoSegmented<OverviewPerspective>(
      expanded: true,
      semanticLabel: l10n.overviewTitle,
      selected: perspective,
      onChanged: (it) => unawaited(cubit.showPerspective(it)),
      segments: <KansoSegment<OverviewPerspective>>[
        KansoSegment<OverviewPerspective>(
          value: OverviewPerspective.today,
          label: l10n.overviewToday,
        ),
        KansoSegment<OverviewPerspective>(
          value: OverviewPerspective.week,
          label: l10n.overviewThisWeek,
        ),
      ],
    );
  }
}

void _open(BuildContext context, String noteId, {int? line}) {
  if (noteId.isNotEmpty) {
    openDigestNote(context, noteId, line: line);
  }
}

Future<void> _revisit(BuildContext context, String noteId) async {
  final cubit = context.read<OverviewCubit>();
  final pick = await pickRevisit(context);
  if (pick != null) {
    unawaited(cubit.revisit(noteId, pick));
  }
}

Widget _masthead(
  BuildContext context, {
  required String overline,
  required String title,
  required String lede,
  Widget? beforeTitle,
}) {
  final colors = context.colors;
  final typography = context.typography;
  return Padding(
    padding: const EdgeInsets.fromLTRB(
      KansoSpacing.lg,
      KansoSpacing.md,
      KansoSpacing.lg,
      0,
    ),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        ?beforeTitle,
        Text(
          overline,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: typography.sectionLabel.copyWith(color: colors.textMuted),
        ),
        const SizedBox(height: KansoSpacing.xxs),
        Text(title, style: typography.largeTitle.copyWith(color: colors.text)),
        const SizedBox(height: KansoSpacing.xs),
        Text(
          lede,
          style: typography.body.copyWith(
            color: colors.textSecondary,
            height: 1.4,
          ),
        ),
        const SizedBox(height: KansoSpacing.md),
        const _PerspectiveSwitch(),
      ],
    ),
  );
}

Widget? _ai(BuildContext context, OverviewViewState state, OverviewAi? ai) {
  final cubit = context.read<OverviewCubit>();
  final l10n = context.l10n;
  final colors = context.colors;
  final typography = context.typography;
  if (state.aiWorking) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.lg,
        KansoSpacing.lg,
        KansoSpacing.md,
        0,
      ),
      child: Row(
        children: <Widget>[
          const KansoSpinner(),
          const SizedBox(width: KansoSpacing.sm),
          Expanded(
            child: Text(
              l10n.digestSummarising,
              style: typography.meta.copyWith(color: colors.textMuted),
            ),
          ),
          KansoPressable(
            onTap: cubit.stopSummary,
            child: Text(
              l10n.digestStop,
              style: typography.label.copyWith(color: colors.link),
            ),
          ),
        ],
      ),
    );
  }
  if (state.aiMessage case final message?) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.lg,
        KansoSpacing.lg,
        KansoSpacing.md,
        0,
      ),
      child: Text(
        message,
        style: typography.meta.copyWith(color: colors.danger),
      ),
    );
  }
  if (ai == null || state.aiHidden) {
    return null;
  }
  return OverviewAiBlock(
    ai: ai,
    more: state.aiMore,
    onOpenNote: (id) => _open(context, id),
    onMore: cubit.moreAi,
    onRewrite: () => unawaited(cubit.summarise()),
    onHide: cubit.hideAi,
  );
}

// MARK: Today

List<Widget> _todayChildren(
  BuildContext context,
  OverviewViewState state,
  OverviewToday t,
) {
  final l10n = context.l10n;
  final cubit = context.read<OverviewCubit>();
  final colors = context.colors;
  final typography = context.typography;
  if (t.state == OverviewState.new_) {
    return <Widget>[
      _masthead(
        context,
        overline: t.overline,
        title: t.greeting,
        lede: l10n.overviewNewLede,
      ),
      Padding(
        padding: const EdgeInsets.all(KansoSpacing.lg),
        child: KansoButton(
          label: l10n.overviewWriteToday,
          onPressed: () => unawaited(context.push(KansoRoutes.daily)),
        ),
      ),
    ];
  }
  final ai = _ai(context, state, t.ai);
  final earlierOpen = state.expanded.contains('earlier');
  return <Widget>[
    _masthead(context, overline: t.overline, title: t.greeting, lede: t.lede),
    ?ai,
    if (t.agenda.isNotEmpty || t.earlier.isNotEmpty)
      OverviewSection(
        label: l10n.overviewToday,
        footer: t.agendaMore > 0
            ? OverviewMoreLine(
                label: l10n.overviewAgendaMore(t.agendaMore, t.taskTotal),
                onTap: () => unawaited(context.push(KansoRoutes.tasks)),
              )
            : null,
        children: <Widget>[
          if (t.earlier.isNotEmpty)
            KansoPressable(
              onTap: () => cubit.toggleExpanded('earlier'),
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.md,
                  vertical: KansoSpacing.sm,
                ),
                child: Row(
                  children: <Widget>[
                    Expanded(
                      child: Text(
                        l10n.overviewEarlier(t.earlier.length),
                        style: typography.meta.copyWith(
                          color: colors.textSecondary,
                        ),
                      ),
                    ),
                    Icon(
                      earlierOpen
                          ? Icons.keyboard_arrow_up
                          : Icons.keyboard_arrow_down,
                      size: 16,
                      color: colors.textMuted,
                    ),
                  ],
                ),
              ),
            ),
          if (earlierOpen)
            for (final row in t.earlier)
              _agendaTile(context, row, dimmed: true),
          for (final row in t.agenda) _agendaTile(context, row),
        ],
      ),
    for (final list in t.lists) _listSection(context, state, list),
    if (t.also.isNotEmpty)
      OverviewSection(
        label: l10n.overviewAlso,
        children: <Widget>[for (final line in t.also) _alsoTile(context, line)],
      ),
    if (t.past case final past?)
      CodaCard(
        past: past,
        onOpen: () => _open(context, past.noteId),
        onRevisit: () => unawaited(_revisit(context, past.noteId)),
        onDismiss: () => unawaited(cubit.dismissPast(past)),
      ),
    if (t.footer.isNotEmpty)
      Padding(
        padding: const EdgeInsets.fromLTRB(
          KansoSpacing.lg,
          KansoSpacing.xl,
          KansoSpacing.lg,
          0,
        ),
        child: Text(
          t.footer,
          style: typography.meta.copyWith(color: colors.textSecondary),
        ),
      ),
  ];
}

Widget _agendaTile(
  BuildContext context,
  OverviewAgendaRow row, {
  bool dimmed = false,
}) {
  final cubit = context.read<OverviewCubit>();
  return AgendaTile(
    key: ValueKey<String>('agenda/${row.key}/${row.line}'),
    row: row,
    dimmed: dimmed,
    onTick: () => unawaited(cubit.tick(row)),
    onOpen: () => _open(context, row.noteId, line: row.line),
    onOpenMeeting: row.meetingNoteId.isEmpty
        ? null
        : () => _open(context, row.meetingNoteId),
    onRevisit: row.noteId.isEmpty
        ? null
        : () => unawaited(_revisit(context, row.noteId)),
  );
}

Widget _listSection(
  BuildContext context,
  OverviewViewState state,
  OverviewList list,
) {
  final cubit = context.read<OverviewCubit>();
  final l10n = context.l10n;
  // The last `more` rows wait behind `N more`, which shows them in place.
  final open = state.expanded.contains('more-${list.kind.name}');
  final rows = open || list.more == 0
      ? list.rows
      : list.rows.take(list.rows.length - list.more).toList();
  return OverviewSection(
    label: switch (list.kind) {
      OverviewListKind.pickUp => l10n.overviewPickUp,
      OverviewListKind.waiting => l10n.overviewWaiting,
      OverviewListKind.activity => l10n.overviewActivity,
    },
    footer: list.more > 0 && !open
        ? OverviewMoreLine(
            label: l10n.overviewMoreRows(list.more),
            onTap: () => cubit.expand('more-${list.kind.name}'),
          )
        : null,
    children: <Widget>[
      for (final row in rows)
        switch (list.kind) {
          OverviewListKind.pickUp => PickUpTile(
            key: ValueKey<String>('pick/${row.noteId}'),
            row: row,
            onOpen: () => _open(context, row.noteId, line: row.line),
            onHide: () => unawaited(cubit.hidePlace(row)),
            onRevisit: () => unawaited(_revisit(context, row.noteId)),
          ),
          OverviewListKind.waiting => WaitingTile(
            key: ValueKey<String>('wait/${row.key}'),
            row: row,
            onOpen: () => _open(context, row.noteId, line: row.line),
            onDone: () => unawaited(cubit.doneWaiting(row)),
            onLater: () => unawaited(cubit.snoozeWaiting(row)),
          ),
          OverviewListKind.activity => ActivityTile(
            row: row,
            onOpen: () => row.agent
                ? unawaited(context.push(KansoRoutes.review))
                : _open(context, row.noteId, line: row.line),
          ),
        },
    ],
  );
}

Widget _alsoTile(BuildContext context, OverviewLine line) {
  final l10n = context.l10n;
  String? action;
  VoidCallback? onAction;
  switch (line.kind) {
    case OverviewLineKind.captured:
      action = l10n.overviewTriage;
      onAction = () => unawaited(context.push(KansoRoutes.triage));
    case OverviewLineKind.promises:
      action = l10n.overviewSeeAll;
      onAction = () => unawaited(context.push(KansoRoutes.loops));
    case OverviewLineKind.revisit:
    case OverviewLineKind.themes:
    case OverviewLineKind.meetings:
    case OverviewLineKind.activity:
    case OverviewLineKind.waiting:
    case OverviewLineKind.pickUp:
      break;
  }
  return AlsoTile(
    line: line,
    action: action,
    onAction: onAction,
    onOpenItem: (item) => _open(context, item.noteId, line: item.line),
  );
}

// MARK: This week

List<Widget> _weekChildren(
  BuildContext context,
  OverviewViewState state,
  OverviewWeek w,
) {
  final l10n = context.l10n;
  final cubit = context.read<OverviewCubit>();
  final colors = context.colors;
  final typography = context.typography;
  final ai = _ai(context, state, w.ai);
  return <Widget>[
    _masthead(
      context,
      overline: w.rangeLabel,
      title: w.isThisWeek ? l10n.overviewThisWeek : l10n.overviewThatWeek,
      lede: w.lede,
      beforeTitle: Padding(
        padding: const EdgeInsets.only(bottom: KansoSpacing.xs),
        child: Row(
          children: <Widget>[
            KansoIconButton(
              icon: KansoIcons.chevronLeft,
              semanticLabel: l10n.overviewPreviousWeek,
              onPressed: () => unawaited(cubit.stepWeek(-1)),
            ),
            KansoIconButton(
              icon: KansoIcons.chevronRight,
              semanticLabel: l10n.overviewNextWeek,
              onPressed: () => unawaited(cubit.stepWeek(1)),
            ),
            if (!w.isThisWeek)
              KansoPressable(
                onTap: () => unawaited(cubit.thisWeek()),
                child: Padding(
                  padding: const EdgeInsets.all(KansoSpacing.xs),
                  child: Text(
                    l10n.overviewThisWeek,
                    style: typography.label.copyWith(color: colors.link),
                  ),
                ),
              ),
            const Spacer(),
            Text(
              w.weekLabel,
              style: typography.meta.copyWith(color: colors.textMuted),
            ),
          ],
        ),
      ),
    ),
    ?ai,
    if (w.projects.isNotEmpty)
      OverviewSection(
        label: l10n.overviewProjects,
        children: <Widget>[
          for (final p in w.projects)
            KansoPressable(
              onTap: () => unawaited(
                context.push('${KansoRoutes.notebookById}/${p.id}'),
              ),
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.md,
                  vertical: KansoSpacing.sm,
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: <Widget>[
                    Text(
                      p.name,
                      style: typography.body.copyWith(
                        color: colors.text,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                    Text(
                      p.lastTitle.isEmpty
                          ? l10n.overviewNotesChanged(p.notesChanged)
                          : '${l10n.overviewNotesChanged(p.notesChanged)} '
                                '· ${l10n.overviewLast(p.lastTitle)}',
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: typography.meta.copyWith(
                        color: colors.textSecondary,
                      ),
                    ),
                    if (p.nextTask.isNotEmpty)
                      Text(
                        p.nextDue.isEmpty
                            ? l10n.overviewNext(p.nextTask)
                            : '${l10n.overviewNext(p.nextTask)} '
                                  '· ${p.nextDue}',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: typography.secondary.copyWith(
                          color: colors.text,
                        ),
                      ),
                  ],
                ),
              ),
            ),
        ],
      ),
    if (w.finished.isNotEmpty)
      OverviewSection(
        label: l10n.overviewFinished,
        count: w.finished.length + w.finishedMore,
        footer: w.finishedMore > 0
            ? Padding(
                padding: const EdgeInsets.all(KansoSpacing.md),
                child: Text(
                  l10n.overviewMoreRows(w.finishedMore),
                  style: typography.meta.copyWith(color: colors.textSecondary),
                ),
              )
            : null,
        children: <Widget>[
          for (final f in w.finished)
            KansoPressable(
              onTap: () => _open(context, f.noteId),
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: KansoSpacing.md,
                  vertical: KansoSpacing.sm,
                ),
                child: Row(
                  children: <Widget>[
                    Icon(Icons.check, size: 16, color: colors.success),
                    const SizedBox(width: KansoSpacing.sm),
                    Expanded(
                      child: Text(
                        f.text,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                        style: typography.body.copyWith(
                          color: colors.textSecondary,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
        ],
      ),
    if (w.waiting.isNotEmpty)
      OverviewSection(
        label: l10n.overviewWaiting,
        children: <Widget>[
          for (final row in w.waiting)
            WaitingTile(
              key: ValueKey<String>('wait/${row.key}'),
              row: row,
              onOpen: () => _open(context, row.noteId, line: row.line),
              onDone: () => unawaited(cubit.doneWaiting(row)),
              onLater: () => unawaited(cubit.snoozeWaiting(row)),
            ),
        ],
      ),
    if (w.stillOpen.isNotEmpty)
      OverviewSection(
        label: l10n.overviewStillOpen,
        count: w.stillOpenTotal,
        footer: w.stillOpenTotal > w.stillOpen.length
            ? OverviewMoreLine(
                label: l10n.overviewAllOpenTasks(w.stillOpenTotal),
                onTap: () => unawaited(context.push(KansoRoutes.tasks)),
              )
            : null,
        children: <Widget>[
          for (final row in w.stillOpen) _agendaTile(context, row),
        ],
      ),
    if (w.ahead.isNotEmpty)
      OverviewSection(
        label: l10n.overviewAhead,
        children: <Widget>[
          for (final day in w.ahead) ...<Widget>[
            Padding(
              padding: EdgeInsets.fromLTRB(
                KansoSpacing.md,
                KansoSpacing.sm,
                KansoSpacing.md,
                day.rows.isEmpty ? KansoSpacing.sm : 0,
              ),
              child: Text(
                day.summary,
                style: typography.meta.copyWith(
                  color: colors.textSecondary,
                  fontWeight: day.rows.isEmpty
                      ? FontWeight.w400
                      : FontWeight.w600,
                ),
              ),
            ),
            for (final row in day.rows) _agendaTile(context, row),
          ],
        ],
      ),
    if (w.also.isNotEmpty)
      OverviewSection(
        label: l10n.overviewAlso,
        children: <Widget>[for (final line in w.also) _alsoTile(context, line)],
      ),
    Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.lg,
        KansoSpacing.xl,
        KansoSpacing.lg,
        0,
      ),
      child: w.savedAs.isEmpty
          ? KansoButton(
              label: l10n.overviewSaveWeek,
              variant: KansoButtonVariant.secondary,
              onPressed: () => unawaited(_saveWeek(context, w)),
            )
          : Text(
              l10n.overviewSavedAs(w.savedAs),
              style: typography.meta.copyWith(color: colors.textSecondary),
            ),
    ),
  ];
}

Future<void> _saveWeek(BuildContext context, OverviewWeek w) async {
  final cubit = context.read<OverviewCubit>();
  final saved = await getIt<DigestRepository>().saveReview(
    from: w.from,
    replace: false,
  );
  if (!context.mounted) {
    return;
  }
  saved.fold((_) {}, (it) => _open(context, it.noteId));
  unawaited(cubit.refresh());
}

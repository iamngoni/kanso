// Kanso — overview_rows.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';

import '../../../core/bridge/api/overview_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/kanso_swipe_action.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_swipe_actions.dart';
import 'overview_card.dart';

/// The hex colour `#rrggbb` of a calendar, or null.
Color? calendarColor(String hex) {
  if (hex.length != 7 || !hex.startsWith('#')) {
    return null;
  }
  final value = int.tryParse(hex.substring(1), radix: 16);
  return value == null ? null : Color(0xFF000000 | value);
}

/// A row of the agenda: the time in a gutter at the leading edge, a tick
/// for a task or a calendar's dot for an event, the title, and the line
/// under it. A late task's time is warm; an event on now is accented.
class AgendaTile extends StatelessWidget {
  const AgendaTile({
    required this.row,
    required this.onTick,
    required this.onOpen,
    this.onOpenMeeting,
    this.onRevisit,
    this.dimmed = false,
    super.key,
  });

  final OverviewAgendaRow row;
  final VoidCallback onTick;
  final VoidCallback onOpen;
  final VoidCallback? onOpenMeeting;
  final VoidCallback? onRevisit;
  final bool dimmed;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final late = row.kind == OverviewRowKind.overdue;
    final isEvent = row.kind == OverviewRowKind.event;
    final timeColor = late
        ? colors.warning
        : row.now
        ? colors.accent
        : row.free
        ? colors.textMuted
        : colors.textSecondary;
    final dot = calendarColor(row.color) ?? colors.accent;
    final tile = KansoPressable(
      onTap: onOpen,
      onLongPress: onRevisit,
      child: Container(
        decoration: row.now
            ? BoxDecoration(
                border: BorderDirectional(
                  start: BorderSide(color: colors.accent, width: 3),
                ),
              )
            : null,
        // The Now bar takes its width out of the padding, so every row's
        // glyph and title line up.
        padding: EdgeInsetsDirectional.only(
          start: KansoSpacing.md - (row.now ? 3 : 0),
          end: KansoSpacing.md,
          top: KansoSpacing.sm,
          bottom: KansoSpacing.sm,
        ),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            // One slot for the event's square and the task's tick, so
            // titles start at the same place whatever the row is.
            if (isEvent)
              Padding(
                padding: const EdgeInsetsDirectional.only(
                  top: 6,
                  end: KansoSpacing.sm,
                ),
                child: SizedBox(
                  width: KansoSizes.checkbox,
                  child: Center(
                    child: Container(
                      width: 9,
                      height: 9,
                      decoration: BoxDecoration(
                        color: dot,
                        borderRadius: BorderRadius.circular(3),
                      ),
                    ),
                  ),
                ),
              )
            else
              KansoPressable(
                onTap: onTick,
                semanticLabel: l10n.overviewTick(row.title),
                child: const Padding(
                  padding: EdgeInsetsDirectional.only(
                    end: KansoSpacing.sm,
                    bottom: KansoSpacing.xxs,
                  ),
                  child: IgnorePointer(child: KansoCheckbox(value: false)),
                ),
              ),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: <Widget>[
                  Text(
                    row.title,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: typography.body.copyWith(
                      color: colors.text,
                      fontWeight: isEvent ? FontWeight.w600 : FontWeight.w400,
                    ),
                  ),
                  if (row.context.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(top: 1),
                      child: Text(
                        row.context,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: typography.meta.copyWith(
                          color: colors.textSecondary,
                        ),
                      ),
                    ),
                ],
              ),
            ),
            if (onOpenMeeting != null)
              KansoPressable(
                onTap: onOpenMeeting,
                child: Padding(
                  padding: const EdgeInsetsDirectional.only(
                    start: KansoSpacing.sm,
                    top: 2,
                  ),
                  child: Text(
                    l10n.overviewMeetingNote,
                    style: typography.meta.copyWith(color: colors.link),
                  ),
                ),
              ),
            // The time trails on the right, in tabular figures.
            Padding(
              padding: const EdgeInsetsDirectional.only(
                start: KansoSpacing.sm,
                top: 2,
              ),
              child: Text(
                row.gutter,
                maxLines: 1,
                style: typography.meta.copyWith(
                  color: timeColor,
                  fontWeight: late || row.now
                      ? FontWeight.w600
                      : FontWeight.w400,
                  fontFeatures: const <FontFeature>[
                    FontFeature.tabularFigures(),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
    return Semantics(
      container: true,
      label: '${row.gutter}, ${row.title}',
      customSemanticsActions: <CustomSemanticsAction, VoidCallback>{
        if (!isEvent)
          CustomSemanticsAction(label: l10n.overviewTickOff): onTick,
        CustomSemanticsAction(label: l10n.overviewOpenNote): onOpen,
      },
      child: Opacity(
        opacity: dimmed ? 0.6 : 1,
        child: ExcludeSemantics(child: tile),
      ),
    );
  }
}

/// A *Pick up where you left off* row: the note and where it is, how long
/// ago, and the words just before the caret.
class PickUpTile extends StatelessWidget {
  const PickUpTile({
    required this.row,
    required this.onOpen,
    required this.onHide,
    required this.onRevisit,
    super.key,
  });

  final OverviewListRow row;
  final VoidCallback onOpen;
  final VoidCallback onHide;
  final VoidCallback onRevisit;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final where = row.place.isEmpty
        ? row.whenLabel
        : '${row.place} · ${row.whenLabel}';
    return KansoSwipeActions(
      id: 'pick/${row.noteId}',
      endActions: <KansoSwipeAction>[
        KansoSwipeAction(
          icon: KansoIcons.clock,
          label: l10n.overviewRevisit,
          background: colors.accent,
          onPressed: onRevisit,
        ),
        KansoSwipeAction(
          icon: KansoIcons.close,
          label: l10n.overviewNotNow,
          background: colors.warning,
          onPressed: onHide,
        ),
      ],
      child: ColoredBox(
        color: context.groupedSurface,
        child: KansoPressable(
          onTap: onOpen,
          onLongPress: onRevisit,
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.md,
              vertical: KansoSpacing.sm,
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: <Widget>[
                Row(
                  crossAxisAlignment: CrossAxisAlignment.baseline,
                  textBaseline: TextBaseline.alphabetic,
                  children: <Widget>[
                    Expanded(
                      child: Text(
                        row.title.trim().isEmpty
                            ? l10n.noteUntitled
                            : row.title,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: typography.body.copyWith(
                          color: colors.text,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                    const SizedBox(width: KansoSpacing.xs),
                    Flexible(
                      child: Text(
                        where,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: typography.meta.copyWith(
                          color: colors.textMuted,
                        ),
                      ),
                    ),
                  ],
                ),
                if (row.detail.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(top: 3),
                    child: Text.rich(
                      TextSpan(
                        children: <InlineSpan>[
                          TextSpan(text: row.detail),
                          if (row.caret)
                            WidgetSpan(
                              alignment: PlaceholderAlignment.middle,
                              child: Container(
                                margin: const EdgeInsets.only(left: 1),
                                width: 2,
                                height: 16,
                                decoration: BoxDecoration(
                                  color: colors.accent,
                                  borderRadius: BorderRadius.circular(1),
                                ),
                              ),
                            ),
                        ],
                      ),
                      maxLines: 3,
                      overflow: TextOverflow.ellipsis,
                      style: typography.secondary.copyWith(
                        color: colors.textSecondary,
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// A *Waiting on others* row: who, what, how long they have been quiet.
/// Swipe to mark it done or ask again later.
class WaitingTile extends StatelessWidget {
  const WaitingTile({
    required this.row,
    required this.onOpen,
    required this.onDone,
    required this.onLater,
    super.key,
  });

  final OverviewListRow row;
  final VoidCallback onOpen;
  final VoidCallback onDone;
  final VoidCallback onLater;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    return KansoSwipeActions(
      id: 'wait/${row.key}',
      startActions: <KansoSwipeAction>[
        KansoSwipeAction(
          icon: KansoIcons.check,
          label: l10n.overviewDone,
          background: colors.success,
          onPressed: onDone,
        ),
      ],
      endActions: <KansoSwipeAction>[
        KansoSwipeAction(
          icon: KansoIcons.alarmClock,
          label: l10n.overviewNotYet,
          background: colors.warning,
          onPressed: onLater,
        ),
      ],
      onCommitStart: onDone,
      child: ColoredBox(
        color: context.groupedSurface,
        child: KansoPressable(
          onTap: onOpen,
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.md,
              vertical: KansoSpacing.sm,
            ),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: <Widget>[
                      Text(
                        row.title,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        style: typography.body.copyWith(
                          color: colors.text,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                      Text(
                        row.detail,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                        style: typography.secondary.copyWith(
                          color: colors.textSecondary,
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: KansoSpacing.sm),
                Padding(
                  padding: const EdgeInsets.only(top: 2),
                  child: Text(
                    l10n.overviewQuietDays(row.quietDays),
                    style: typography.meta.copyWith(
                      color: row.warn ? colors.warning : colors.textMuted,
                      fontWeight: row.warn ? FontWeight.w600 : FontWeight.w400,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// An agent or shared activity row.
class ActivityTile extends StatelessWidget {
  const ActivityTile({required this.row, required this.onOpen, super.key});

  final OverviewListRow row;
  final VoidCallback onOpen;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    return KansoPressable(
      onTap: onOpen,
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.md,
          vertical: KansoSpacing.sm,
        ),
        child: Row(
          children: <Widget>[
            KansoIcon(
              row.agent ? KansoIcons.sparkles : KansoIcons.users,
              size: 16,
              color: row.agent ? colors.accent : colors.textMuted,
            ),
            const SizedBox(width: KansoSpacing.sm),
            Expanded(
              child: Text(
                row.title,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: typography.body.copyWith(color: colors.text),
              ),
            ),
            Text(
              row.agent ? l10n.overviewReview : row.whenLabel,
              style: typography.meta.copyWith(
                color: row.agent ? colors.link : colors.textSecondary,
                fontWeight: row.agent ? FontWeight.w600 : FontWeight.w400,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// A Line of *Also*: its name, what it holds, and one action.
class AlsoTile extends StatelessWidget {
  const AlsoTile({
    required this.line,
    required this.onOpenItem,
    this.action,
    this.onAction,
    super.key,
  });

  final OverviewLine line;
  final void Function(OverviewItem item) onOpenItem;
  final String? action;
  final VoidCallback? onAction;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final lead = switch (line.kind) {
      OverviewLineKind.captured => l10n.overviewInbox(line.count),
      OverviewLineKind.promises => l10n.overviewPromisesCount(line.count),
      OverviewLineKind.waiting => l10n.overviewWaitingCount(line.count),
      _ => null,
    };
    final cards = line.kind == OverviewLineKind.revisit && line.cards > 0
        ? l10n.overviewFlashcards(line.cards)
        : null;
    return Padding(
      padding: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.md,
        vertical: KansoSpacing.sm,
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          SizedBox(
            width: 84,
            child: Text(
              line.label,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: typography.label.copyWith(
                color: colors.textSecondary,
                fontWeight: FontWeight.w600,
              ),
            ),
          ),
          Expanded(
            child: Wrap(
              spacing: KansoSpacing.xs,
              runSpacing: 2,
              children: <Widget>[
                if (lead != null)
                  Text(
                    lead,
                    style: typography.secondary.copyWith(
                      color: colors.textSecondary,
                    ),
                  ),
                for (final item in line.items)
                  KansoPressable(
                    onTap: () => onOpenItem(item),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: <Widget>[
                        if (item.voiceSecs > 0)
                          Padding(
                            padding: const EdgeInsets.only(right: 3),
                            child: KansoIcon(
                              KansoIcons.mic,
                              size: 13,
                              color: colors.textSecondary,
                            ),
                          ),
                        ConstrainedBox(
                          constraints: const BoxConstraints(maxWidth: 220),
                          child: Text(
                            line.kind == OverviewLineKind.revisit
                                ? item.text
                                : '“${item.text}”',
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: typography.secondary.copyWith(
                              color: colors.text,
                            ),
                          ),
                        ),
                        if (item.voiceSecs > 0)
                          Text(
                            ' ${_clockOf(item.voiceSecs)}',
                            style: typography.meta.copyWith(
                              color: colors.textMuted,
                            ),
                          ),
                      ],
                    ),
                  ),
                if (cards != null)
                  Text(
                    cards,
                    style: typography.secondary.copyWith(
                      color: colors.textSecondary,
                    ),
                  ),
                if (line.kind == OverviewLineKind.themes)
                  Text(
                    line.detail.join(' · '),
                    style: typography.secondary.copyWith(color: colors.link),
                  ),
              ],
            ),
          ),
          if (action != null && onAction != null)
            KansoPressable(
              onTap: onAction,
              child: Padding(
                padding: const EdgeInsetsDirectional.only(
                  start: KansoSpacing.xs,
                ),
                child: Text(
                  action!,
                  style: typography.label.copyWith(
                    color: colors.link,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

/// The one note from the past: a pull-quote and why it is here.
class CodaCard extends StatelessWidget {
  const CodaCard({
    required this.past,
    required this.onOpen,
    required this.onRevisit,
    required this.onDismiss,
    super.key,
  });

  final OverviewPast past;
  final VoidCallback onOpen;
  final VoidCallback onRevisit;
  final VoidCallback onDismiss;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.lg,
        KansoSpacing.xl,
        KansoSpacing.lg,
        0,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(
            l10n.overviewFromThePast(past.why).toUpperCase(),
            style: typography.sectionLabel.copyWith(color: colors.textMuted),
          ),
          const SizedBox(height: KansoSpacing.xs),
          KansoPressable(
            onTap: onOpen,
            child: Text(
              past.title.trim().isEmpty ? l10n.noteUntitled : past.title,
              style: typography.heading2.copyWith(color: colors.text),
            ),
          ),
          if (past.excerpt.isNotEmpty)
            Container(
              margin: const EdgeInsets.only(top: KansoSpacing.xs),
              padding: const EdgeInsetsDirectional.only(start: KansoSpacing.sm),
              decoration: BoxDecoration(
                border: BorderDirectional(
                  start: BorderSide(color: colors.divider, width: 2),
                ),
              ),
              child: Text(
                '“${past.excerpt}”',
                maxLines: 5,
                overflow: TextOverflow.ellipsis,
                style: typography.body.copyWith(color: colors.textSecondary),
              ),
            ),
          const SizedBox(height: KansoSpacing.xs),
          Wrap(
            spacing: KansoSpacing.md,
            children: <Widget>[
              for (final (label, tap, color) in <(String, VoidCallback, Color)>[
                (l10n.overviewOpen, onOpen, colors.link),
                (l10n.overviewRevisit, onRevisit, colors.link),
                (l10n.overviewNotUseful, onDismiss, colors.textSecondary),
              ])
                KansoPressable(
                  onTap: tap,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(vertical: 6),
                    child: Text(
                      label,
                      style: typography.label.copyWith(color: color),
                    ),
                  ),
                ),
            ],
          ),
        ],
      ),
    );
  }
}

/// The space the page leaves under its last card.
const double overviewBottomSpace = KansoSpacing.xxl + KansoSpacing.lg;

/// A page of Overview's wide gutter on a tablet, for a column.
EdgeInsets overviewPagePadding(double width) {
  final side = width > overviewColumnWidth + 2 * KansoSpacing.lg
      ? (width - overviewColumnWidth) / 2
      : 0.0;
  return EdgeInsets.symmetric(horizontal: side);
}

/// `1:05` from seconds.
String _clockOf(int seconds) =>
    '${seconds ~/ 60}:${(seconds % 60).toString().padLeft(2, '0')}';

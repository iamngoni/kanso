// Kanso — page_section_view.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../ask/utils/open_cited_note.dart';
import '../../../core/bridge/api/people_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/ai_trouble_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_menu.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import 'people_menu.dart';

/// What the menu of a section can do.
enum PageSectionAction { refresh, keep, copy, hide }

/// Builds the row of one item of a section.
typedef PageItemBuilder =
    Widget Function(BuildContext context, PageItem item, int index);

/// One section of a person's, a project's or a notebook's page: a header
/// that folds it and has its menu, a stale line, what went wrong, its rows
/// (with "Show all" and the section's note under them).
///
/// It holds nothing and does nothing itself: the page that owns the data
/// says what to do for each tap, so the people pages and the notebook
/// homepages draw one section the same way.
class PageSectionView extends StatelessWidget {
  const PageSectionView({
    required this.section,
    required this.aiOn,
    required this.collapsed,
    required this.onToggle,
    required this.onAction,
    this.working = false,
    this.trouble,
    this.onRetry,
    this.onShowAll,
    this.itemBuilder,
    this.body,
    this.action,
    this.copyLabel,
    this.canCopy,
    this.emptyFallback = '',
    super.key,
  });

  final PageSection section;
  final bool aiOn;
  final bool collapsed;
  final VoidCallback onToggle;
  final ValueChanged<PageSectionAction> onAction;

  /// Whether the section is being rewritten now.
  final bool working;

  /// The model's own words when the rewrite failed.
  final String? trouble;
  final VoidCallback? onRetry;
  final VoidCallback? onShowAll;

  /// Draws each item; the default is a [PageItemRow] that opens its note.
  final PageItemBuilder? itemBuilder;

  /// Drawn instead of the rows, when the section is not a list of them.
  final Widget? body;

  /// Drawn under an empty section, such as the button that writes it.
  final Widget? action;

  /// The words of "Copy into the note"; the people ones when null.
  final String? copyLabel;

  /// Whether the menu can copy the section; when it has rows, if null.
  final bool? canCopy;

  /// What an empty section says when it has no line of its own.
  final String emptyFallback;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final s = section;
    final pinned = s.state == PageSectionState.pinned;
    final entries = <KansoMenuEntry<PageSectionAction>>[
      if (s.ai && aiOn)
        KansoMenuEntry<PageSectionAction>(
          PageSectionAction.refresh,
          l10n.peopleRefresh,
          icon: KansoIcons.refreshCw,
        ),
      if (s.ai)
        KansoMenuEntry<PageSectionAction>(
          PageSectionAction.keep,
          pinned ? l10n.peopleUnkeep : l10n.peopleKeep,
          icon: KansoIcons.pin,
        ),
      if (canCopy ?? s.items.isNotEmpty)
        KansoMenuEntry<PageSectionAction>(
          PageSectionAction.copy,
          copyLabel ?? l10n.peopleCopyInto,
          icon: KansoIcons.copy,
        ),
      KansoMenuEntry<PageSectionAction>(
        PageSectionAction.hide,
        l10n.peopleHide,
        icon: KansoIcons.eye,
      ),
    ];
    String? group;
    final empty = body == null && s.items.isEmpty;
    return Padding(
      padding: const EdgeInsets.only(top: KansoSpacing.md),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Row(
            children: <Widget>[
              Expanded(
                child: KansoPressable(
                  onTap: onToggle,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      vertical: KansoSpacing.xs,
                    ),
                    child: Row(
                      children: <Widget>[
                        KansoIcon(
                          collapsed
                              ? KansoIcons.chevronRight
                              : KansoIcons.chevronDown,
                          size: 14,
                          color: colors.textMuted,
                        ),
                        const SizedBox(width: KansoSpacing.xxs),
                        if (s.ai) ...<Widget>[
                          KansoIcon(
                            KansoIcons.sparkles,
                            size: 14,
                            color: colors.textMuted,
                          ),
                          const SizedBox(width: KansoSpacing.xxs),
                        ],
                        Flexible(
                          child: Text(
                            s.title,
                            style: typography.label,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        if (pinned) ...<Widget>[
                          const SizedBox(width: KansoSpacing.xxs),
                          KansoIcon(
                            KansoIcons.pin,
                            size: 12,
                            color: colors.textMuted,
                          ),
                        ],
                      ],
                    ),
                  ),
                ),
              ),
              if (working)
                const SizedBox.square(
                  dimension: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
              PeopleMenuButton<PageSectionAction>(
                semanticLabel: s.title,
                entries: entries,
                onChosen: onAction,
              ),
            ],
          ),
          if (!collapsed) ...<Widget>[
            if (s.staleLine.isNotEmpty)
              Text(
                s.staleLine,
                style: typography.meta.copyWith(color: colors.warning),
              ),
            if (trouble != null)
              _TroubleCard(trouble: trouble!, onRetry: onRetry),
            if (empty && trouble == null) ...<Widget>[
              if (s.emptyLine.isNotEmpty || emptyFallback.isNotEmpty)
                Text(
                  s.emptyLine.isNotEmpty ? s.emptyLine : emptyFallback,
                  style: typography.secondary.copyWith(
                    color: colors.textMuted,
                  ),
                ),
              if (action != null) action!,
            ],
            if (body != null) body!,
            if (body == null)
              for (var i = 0; i < s.items.length; i++) ...<Widget>[
                if (s.items[i].group.isNotEmpty && s.items[i].group != group)
                  Padding(
                    padding: const EdgeInsets.only(top: KansoSpacing.xs),
                    child: Text(
                      group = s.items[i].group,
                      style: typography.meta.copyWith(color: colors.textMuted),
                    ),
                  ),
                itemBuilder?.call(context, s.items[i], i) ??
                    PageItemRow(section: s, item: s.items[i], index: i),
              ],
            if (s.total > s.items.length && onShowAll != null)
              Align(
                alignment: AlignmentDirectional.centerStart,
                child: KansoButton(
                  label: l10n.peopleShowAll(s.total),
                  variant: KansoButtonVariant.quiet,
                  compact: true,
                  expanded: false,
                  onPressed: onShowAll,
                ),
              ),
            if (s.note.isNotEmpty)
              Text(
                s.note,
                style: typography.meta.copyWith(color: colors.textMuted),
              ),
          ],
        ],
      ),
    );
  }
}

class _TroubleCard extends StatelessWidget {
  const _TroubleCard({required this.trouble, required this.onRetry});

  final String trouble;
  final VoidCallback? onRetry;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xs),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          Text(
            aiTroubleText(trouble, l10n),
            style: context.typography.secondary.copyWith(
              color: context.colors.danger,
            ),
          ),
          Row(
            children: <Widget>[
              if (onRetry != null) ...<Widget>[
                KansoButton(
                  label: l10n.aiTroubleRetry,
                  variant: KansoButtonVariant.secondary,
                  compact: true,
                  expanded: false,
                  onPressed: onRetry,
                ),
                const SizedBox(width: KansoSpacing.xs),
              ],
              KansoButton(
                label: l10n.aiTroubleSettings,
                variant: KansoButtonVariant.quiet,
                compact: true,
                expanded: false,
                onPressed: () => context.push(KansoRoutes.settingsAi),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// One item of a section: its text, what it is about, the lines it came
/// from (each one opens its note at those lines), a box to tick when it is
/// a task. A long press on an AI item says it is not right.
class PageItemRow extends StatelessWidget {
  const PageItemRow({
    required this.section,
    required this.item,
    required this.index,
    this.onOpen,
    this.onNotRight,
    this.onTick,
    super.key,
  });

  final PageSection section;
  final PageItem item;
  final int index;

  /// What a tap does; opens the item's note, or its first source, if null.
  final VoidCallback? onOpen;
  final VoidCallback? onNotRight;
  final ValueChanged<bool>? onTick;

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final typography = context.typography;
    final l10n = context.l10n;
    final task = item.task;
    final who = switch (item.who) {
      'you' => l10n.peopleYouPromised,
      'them' => l10n.peopleTheyPromised,
      _ => '',
    };
    final source = item.sources.isEmpty ? null : item.sources.first;
    void openSource(PageSource s) =>
        context.push(citedLinesLocation(s.noteId, s.start, s.end));
    final open =
        onOpen ??
        (item.target.isNotEmpty
            ? () => context.push(citedNoteLocation(item.target))
            : source == null
            ? null
            : () => openSource(source));
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: <Widget>[
          if (task != null)
            Padding(
              padding: const EdgeInsets.only(right: KansoSpacing.xs),
              child: KansoCheckbox(
                value: task.checked,
                onChanged: onTick == null ? null : (v) => onTick!(v),
              ),
            ),
          Expanded(
            child: KansoPressable(
              onTap: open,
              onLongPress: section.ai ? onNotRight : null,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  if (who.isNotEmpty)
                    Text(
                      who,
                      style: typography.meta.copyWith(color: colors.textMuted),
                    ),
                  Text(item.text, style: typography.body),
                  if (item.meta.isNotEmpty || item.tag.isNotEmpty)
                    Text(
                      <String>[
                        if (item.tag.isNotEmpty) item.tag,
                        if (item.meta.isNotEmpty) item.meta,
                        if (item.shared) l10n.peopleSharedNote,
                      ].join(' · '),
                      style: typography.meta.copyWith(color: colors.textMuted),
                    ),
                  for (final src in item.sources)
                    if (section.ai)
                      KansoPressable(
                        onTap: () => openSource(src),
                        child: Text(
                          src.label,
                          style: typography.meta.copyWith(color: colors.link),
                        ),
                      ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

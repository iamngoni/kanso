// Kanso — page_panel.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../ask/utils/open_cited_note.dart';
import '../../../core/bridge/api/library.dart';
import '../../../core/bridge/api/people_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_menu.dart';
import '../../../injection.dart';
import '../../models/repos/abs/people_repository.dart';
import 'page_section_view.dart';
import 'people_menu.dart';
import 'people_name_sheet.dart';

/// The live part of a person's or project's page, drawn below the note's
/// text: computed sections and AI sections whose every line cites the lines
/// it came from. It draws nothing for a note that is not a page, so the
/// note's layout does not move.
class PeoplePagePanel extends StatefulWidget {
  const PeoplePagePanel({required this.noteId, super.key});

  final String noteId;

  @override
  State<PeoplePagePanel> createState() => _PeoplePagePanelState();
}

enum _PageAction { refreshAll, merge, alias, showHidden }

class _PeoplePagePanelState extends State<PeoplePagePanel> {
  final PeopleRepository _people = getIt<PeopleRepository>();
  PagePanel? _panel;

  /// Section keys being rewritten now.
  final Set<String> _working = <String>{};

  /// Section keys that failed to rewrite, with the model's own words.
  final Map<String, String> _trouble = <String, String>{};
  final Set<String> _collapsed = <String>{};

  @override
  void initState() {
    super.initState();
    unawaited(_open());
  }

  @override
  void didUpdateWidget(PeoplePagePanel old) {
    super.didUpdateWidget(old);
    if (old.noteId != widget.noteId) {
      _panel = null;
      _working.clear();
      _trouble.clear();
      unawaited(_open());
    }
  }

  Future<void> _open() async {
    final id = widget.noteId;
    final read = (await _people.page(id)).toOption().toNullable();
    if (!mounted || id != widget.noteId) {
      return;
    }
    setState(() => _panel = read);
    // Sections the person's refresh setting wants rewritten now, one by one
    // so a failure names its own section.
    for (final key in read?.due ?? const <String>[]) {
      if (!mounted || id != widget.noteId) {
        return;
      }
      await _refresh(key);
    }
  }

  Future<void> _refresh(String key) async {
    setState(() {
      _working.add(key);
      _trouble.remove(key);
    });
    final done = await _people.refreshSection(widget.noteId, key);
    if (!mounted) {
      return;
    }
    done.fold(
      (error) => setState(() {
        _working.remove(key);
        _trouble[key] = error.message;
      }),
      (panel) => setState(() {
        _working.remove(key);
        _panel = panel ?? _panel;
      }),
    );
  }

  Future<void> _refreshAll() async {
    final panel = _panel;
    if (panel == null) {
      return;
    }
    for (final s in panel.sections.where((s) => s.ai)) {
      if (s.state != PageSectionState.pinned) {
        await _refresh(s.key);
      }
    }
  }

  void _take(PagePanel? panel) {
    if (mounted && panel != null) {
      setState(() => _panel = panel);
    }
  }

  Future<void> _sectionAction(PageSection s, PageSectionAction action) async {
    final id = widget.noteId;
    switch (action) {
      case PageSectionAction.refresh:
        await _refresh(s.key);
      case PageSectionAction.keep:
        final on = s.state != PageSectionState.pinned;
        _take((await _people.pin(id, s.key, on: on)).toOption().toNullable());
      case PageSectionAction.copy:
        final copied = (await _people.copyIntoNote(
          id,
          s.key,
        )).toOption().toNullable();
        if ((copied ?? false) && mounted) {
          ScaffoldMessenger.maybeOf(context)?.showSnackBar(
            SnackBar(content: Text(context.l10n.peopleCopiedInto)),
          );
        }
      case PageSectionAction.hide:
        _take(
          (await _people.hide(id, s.key, hidden: true)).toOption().toNullable(),
        );
    }
  }

  Future<void> _pageAction(_PageAction action) async {
    final id = widget.noteId;
    switch (action) {
      case _PageAction.refreshAll:
        await _refreshAll();
      case _PageAction.showHidden:
        _take((await _people.showHidden(id)).toOption().toNullable());
      case _PageAction.alias:
        final answer = await PeopleNameSheet.alias(context);
        if (answer == null) {
          return;
        }
        await _people.addAlias(id, answer);
        await _reload();
      case _PageAction.merge:
        await _merge();
    }
  }

  Future<void> _reload() async {
    final read = (await _people.page(widget.noteId)).toOption().toNullable();
    _take(read);
  }

  Future<void> _merge() async {
    final l10n = context.l10n;
    final pages = (await _people.pages()).toOption().toNullable() ?? const [];
    final header = _panel?.header;
    final others = pages
        .where((p) => p.id != widget.noteId && p.kind == header?.kind)
        .toList();
    if (!mounted) {
      return;
    }
    if (others.isEmpty) {
      ScaffoldMessenger.maybeOf(
        context,
      )?.showSnackBar(SnackBar(content: Text(l10n.peopleMergeNone)));
      return;
    }
    final target = await PeopleNameSheet.pick(context, others);
    if (target == null || !mounted) {
      return;
    }
    final ok = await KansoConfirmSheet.show(
      context,
      title: l10n.peopleMergeConfirmTitle(target.title),
      message: l10n.peopleMergeConfirmMessage(target.title),
      confirmLabel: l10n.peopleMergeConfirm,
    );
    if (!ok || !mounted) {
      return;
    }
    // This page folds into the one picked; the note opens there.
    final merged = await _people.merge(target.id, widget.noteId);
    if (mounted) {
      merged.fold((_) {}, (m) => context.go(citedNoteLocation(m.into)));
    }
  }

  Future<void> _notRight(PageSection s, int index) async {
    final l10n = context.l10n;
    final ok = await KansoConfirmSheet.show(
      context,
      title: l10n.peopleNotRight,
      message: s.items[index].text,
      confirmLabel: l10n.peopleNotRight,
    );
    if (ok) {
      _take(
        (await _people.notRight(
          widget.noteId,
          s.key,
          index,
        )).toOption().toNullable(),
      );
    }
  }

  Future<void> _tick(PageItem item, bool checked) async {
    final task = item.task;
    if (task == null) {
      return;
    }
    await getIt<KansoLibrary>().setTask(
      noteId: task.noteId,
      line: task.line,
      text: task.text,
      checked: checked,
    );
    await _reload();
  }

  Future<void> _showAll(PageSection s) async {
    final all = (await _people.showAll(
      widget.noteId,
      s.key,
    )).toOption().toNullable();
    final panel = _panel;
    if (all == null || panel == null || !mounted) {
      return;
    }
    setState(
      () => _panel = PagePanel(
        header: panel.header,
        sections: <PageSection>[
          for (final x in panel.sections) x.key == s.key ? all : x,
        ],
        hidden: panel.hidden,
        aiOn: panel.aiOn,
        due: panel.due,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final panel = _panel;
    if (panel == null) {
      return const SizedBox.shrink();
    }
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    return Padding(
      padding: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.lg,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Container(height: 1, color: colors.divider),
          const SizedBox(height: KansoSpacing.sm),
          Row(
            children: <Widget>[
              Expanded(
                child: Text(
                  panel.header.line,
                  style: typography.meta.copyWith(color: colors.textMuted),
                ),
              ),
              PeopleMenuButton<_PageAction>(
                semanticLabel: l10n.peopleTitle,
                entries: <KansoMenuEntry<_PageAction>>[
                  if (panel.aiOn)
                    KansoMenuEntry<_PageAction>(
                      _PageAction.refreshAll,
                      l10n.peopleRefreshAll,
                      icon: KansoIcons.refreshCw,
                    ),
                  if (panel.hidden.isNotEmpty)
                    KansoMenuEntry<_PageAction>(
                      _PageAction.showHidden,
                      l10n.peopleShowHidden,
                      icon: KansoIcons.eye,
                    ),
                  KansoMenuEntry<_PageAction>(
                    _PageAction.alias,
                    l10n.peopleAddAlias,
                    icon: KansoIcons.plus,
                  ),
                  KansoMenuEntry<_PageAction>(
                    _PageAction.merge,
                    l10n.peopleMergeWith,
                    icon: KansoIcons.users,
                  ),
                ],
                onChosen: (a) => unawaited(_pageAction(a)),
              ),
            ],
          ),
          for (final s in panel.sections) _section(context, panel, s),
          if (panel.hidden.isNotEmpty)
            Align(
              alignment: AlignmentDirectional.centerStart,
              child: KansoButton(
                label: l10n.peopleShowHidden,
                variant: KansoButtonVariant.quiet,
                compact: true,
                expanded: false,
                onPressed: () => unawaited(_pageAction(_PageAction.showHidden)),
              ),
            ),
        ],
      ),
    );
  }

  Widget _section(BuildContext context, PagePanel panel, PageSection s) {
    return PageSectionView(
      section: s,
      aiOn: panel.aiOn,
      collapsed: _collapsed.contains(s.key),
      working: _working.contains(s.key),
      trouble: _trouble[s.key],
      emptyFallback: context.l10n.peopleDismissedNone,
      onToggle: () => setState(() {
        if (!_collapsed.add(s.key)) {
          _collapsed.remove(s.key);
        }
      }),
      onAction: (a) => unawaited(_sectionAction(s, a)),
      onRetry: () => unawaited(_refresh(s.key)),
      onShowAll: () => unawaited(_showAll(s)),
      itemBuilder: (context, item, index) => PageItemRow(
        section: s,
        item: item,
        index: index,
        onNotRight: () => unawaited(_notRight(s, index)),
        onTick: (v) => unawaited(_tick(item, v)),
      ),
    );
  }
}

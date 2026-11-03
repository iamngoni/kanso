// Kanso — home_body.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../ask/utils/open_cited_note.dart';
import '../../../core/bridge/api/notebook_home_types.dart';
import '../../../core/bridge/api/people_types.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../people/views/widgets/page_section_view.dart';
import '../../state/notebook_home/notebook_home_bloc.dart';
import 'home_map_card.dart';
import 'home_receipts.dart';

/// The sections of one notebook's homepage in a single column, in the order
/// the core gave them, each one foldable. Every word on it (the section
/// titles, the lines, the footer, the button that writes a section) is the
/// core's, so the desktop and the phone say the same things.
class HomeBody extends StatefulWidget {
  const HomeBody({
    required this.panel,
    required this.words,
    required this.state,
    required this.name,
    this.onChanged,
    super.key,
  });

  final HomePanel panel;
  final HomeWords words;
  final NotebookHomeState state;

  /// The notebook's name, for the map's spoken label.
  final String name;

  /// Called after the person came back from editing a note.
  final VoidCallback? onChanged;

  @override
  State<HomeBody> createState() => _HomeBodyState();
}

class _HomeBodyState extends State<HomeBody> {
  final Set<String> _collapsed = <String>{};

  NotebookHomeBloc get _bloc => context.read<NotebookHomeBloc>();
  String get _bookId => widget.state.bookId;

  Future<void> _openNote(String id) async {
    await context.push(citedNoteLocation(id));
    if (mounted) {
      _bloc.add(const NotebookHomeEvent.reopened());
    }
  }

  void _openNotebook(String id) {
    context.go(BrowseLocation.place(ListPlace.notebook(id: id)).location);
  }

  void _openGraph() {
    context.push(
      '${KansoRoutes.graph}?${KansoRoutes.graphBookParameter}='
      '${Uri.encodeQueryComponent(_bookId)}',
    );
  }

  Future<void> _notRight(PageSection s, int index) async {
    final l10n = context.l10n;
    final ok = await KansoConfirmSheet.show(
      context,
      title: l10n.peopleNotRight,
      message: s.items[index].text,
      confirmLabel: l10n.peopleNotRight,
    );
    if (ok && mounted) {
      _bloc.add(NotebookHomeEvent.notRight(s.key, index));
    }
  }

  void _action(PageSection s, PageSectionAction action) {
    switch (action) {
      case PageSectionAction.refresh:
        _bloc.add(NotebookHomeEvent.sectionRefreshRequested(s.key));
      case PageSectionAction.keep:
        _bloc.add(
          NotebookHomeEvent.pinned(
            s.key,
            on: s.state != PageSectionState.pinned,
          ),
        );
      case PageSectionAction.copy:
        _bloc.add(NotebookHomeEvent.copiedIntoHome(s.key));
      case PageSectionAction.hide:
        _bloc.add(NotebookHomeEvent.hidden(s.key, hidden: true));
    }
  }

  @override
  Widget build(BuildContext context) {
    final panel = widget.panel;
    final colors = context.colors;
    final typography = context.typography;
    return ListView(
      padding: EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.lg + MediaQuery.paddingOf(context).bottom,
      ),
      children: <Widget>[
        if (panel.header.name.isNotEmpty)
          Text(panel.header.name, style: typography.heading1),
        if (panel.header.line.isNotEmpty)
          Text(
            panel.header.line,
            style: typography.meta.copyWith(color: colors.textMuted),
          ),
        for (final s in panel.sections) _section(context, s),
        if (panel.hidden.isNotEmpty)
          Align(
            alignment: AlignmentDirectional.centerStart,
            child: KansoButton(
              label: context.l10n.peopleShowHidden,
              variant: KansoButtonVariant.quiet,
              compact: true,
              expanded: false,
              onPressed: () => _bloc.add(const NotebookHomeEvent.hiddenShown()),
            ),
          ),
        if (panel.footer.isNotEmpty)
          Padding(
            padding: const EdgeInsets.only(top: KansoSpacing.md),
            child: Text(
              panel.footer,
              style: typography.meta.copyWith(color: colors.textMuted),
            ),
          ),
      ],
    );
  }

  Widget _section(BuildContext context, PageSection s) {
    final panel = widget.panel;
    final words = widget.words;
    final state = widget.state;
    final unwritten = s.ai && s.state == PageSectionState.unwritten;
    return PageSectionView(
      key: ValueKey<String>('home-section-${s.key}'),
      section: s,
      aiOn: panel.aiOn,
      collapsed: _collapsed.contains(s.key),
      working: state.working.contains(s.key),
      trouble: state.trouble[s.key],
      copyLabel: context.l10n.notebookHomeCopyInto,
      canCopy: s.items.isNotEmpty && s.key != 'home_note' && s.key != 'map',
      emptyFallback: s.key == 'open_tasks' ? words.noOpenTasks : '',
      onToggle: () => setState(() {
        if (!_collapsed.add(s.key)) {
          _collapsed.remove(s.key);
        }
      }),
      onAction: (a) => _action(s, a),
      onRetry: () =>
          _bloc.add(NotebookHomeEvent.sectionRefreshRequested(s.key)),
      onShowAll: () => _bloc.add(NotebookHomeEvent.allShown(s.key)),
      action: unwritten && panel.aiOn
          ? Align(
              alignment: AlignmentDirectional.centerStart,
              child: KansoButton(
                label: words.firstVisitButton,
                icon: KansoIcons.sparkles,
                variant: KansoButtonVariant.secondary,
                compact: true,
                expanded: false,
                onPressed: () => _bloc.add(
                  NotebookHomeEvent.sectionRefreshRequested(s.key),
                ),
              ),
            )
          : null,
      body: _body(context, s),
      itemBuilder: (context, item, index) => _item(context, s, item, index),
    );
  }

  Widget? _body(BuildContext context, PageSection s) {
    switch (s.key) {
      case 'map':
        return Padding(
          padding: const EdgeInsets.only(top: KansoSpacing.xs),
          child: HomeMapCard(
            map: widget.state.map,
            semanticLabel: context.l10n.notebookHomeMapLabel(widget.name),
            onTap: _openGraph,
          ),
        );
      case 'home_note':
        if (s.items.isEmpty) {
          return null;
        }
        final item = s.items.first;
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(item.text, style: context.typography.previewBody),
            if (item.target.isNotEmpty)
              KansoButton(
                label: context.l10n.notebookHomeEdit,
                icon: KansoIcons.pencil,
                variant: KansoButtonVariant.quiet,
                compact: true,
                expanded: false,
                onPressed: () => unawaited(_openNote(item.target)),
              ),
          ],
        );
      case 'about':
        if (s.items.isEmpty) {
          return null;
        }
        final sources = <PageSource>[
          for (final item in s.items) ...item.sources,
        ];
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(
              s.items.map((i) => i.text).join(' '),
              style: context.typography.body,
            ),
            if (sources.isNotEmpty) ...<Widget>[
              const SizedBox(height: KansoSpacing.xs),
              HomeReceipts(sources: sources),
            ],
          ],
        );
    }
    return null;
  }

  Widget _item(
    BuildContext context,
    PageSection s,
    PageItem item,
    int index,
  ) {
    final colors = context.colors;
    final typography = context.typography;
    switch (s.key) {
      case 'open_tasks':
        return PageItemRow(
          section: s,
          item: item,
          index: index,
          onOpen: item.target.isEmpty ? null : () => _openNote(item.target),
          onTick: (v) {
            final task = item.task;
            if (task != null) {
              _bloc.add(NotebookHomeEvent.taskTicked(task, checked: v));
            }
          },
        );
      case 'decisions':
        return Padding(
          padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Text('• ${item.text}', style: typography.body),
              Row(
                children: <Widget>[
                  if (item.meta.isNotEmpty)
                    Padding(
                      padding: const EdgeInsets.only(right: KansoSpacing.xs),
                      child: Text(
                        item.meta,
                        style: typography.meta.copyWith(
                          color: colors.textMuted,
                        ),
                      ),
                    ),
                  Expanded(child: HomeReceipts(sources: item.sources)),
                  KansoPressable(
                    onTap: () => unawaited(_notRight(s, index)),
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.xxs),
                      child: Text(
                        context.l10n.peopleNotRight,
                        style: typography.meta.copyWith(color: colors.link),
                      ),
                    ),
                  ),
                ],
              ),
            ],
          ),
        );
      case 'inside':
      case 'people':
      case 'key_notes':
      case 'recent':
        final locked = item.tag == 'locked';
        final open = item.target.isEmpty
            ? null
            : s.key == 'inside'
            ? () => _openNotebook(item.target)
            : () => unawaited(_openNote(item.target));
        return KansoPressable(
          onTap: open,
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xxs),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                if (locked)
                  Padding(
                    padding: const EdgeInsets.only(right: KansoSpacing.xs),
                    child: KansoIcon(
                      KansoIcons.lock,
                      size: 14,
                      color: colors.textMuted,
                    ),
                  ),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: <Widget>[
                      Text(item.text, style: typography.body),
                      // A locked notebook shows its name and nothing else.
                      if (!locked && item.meta.isNotEmpty)
                        Text(
                          item.meta,
                          style: typography.meta.copyWith(
                            color: colors.textMuted,
                          ),
                        ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
    }
    return PageItemRow(
      section: s,
      item: item,
      index: index,
      onNotRight: () => unawaited(_notRight(s, index)),
    );
  }
}

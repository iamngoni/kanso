// Kanso — notebook_home_page.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../ask/utils/open_cited_note.dart';
import '../../../core/bridge/api/notebook_home_types.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_menu.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../../lock/state/lock_status/lock_status_bloc.dart';
import '../../../note_list/views/widgets/notebook_gate_host.dart';
import '../../../people/views/widgets/people_menu.dart';
import '../../../shell/views/widgets/shell_back_button.dart';
import '../../models/repos/abs/notebook_home_repository.dart';
import '../../state/notebook_home/notebook_home_bloc.dart';
import '../widgets/home_body.dart';
import '../widgets/home_sheets.dart';

enum _HomeAction { chooseHome, newHome, customise, showHidden }

/// A notebook's homepage: what is in it, what it is about, and what is open
/// in it, on a page of its own (a note pane on a tablet, [inPane]).
///
/// A locked notebook draws no homepage until it is open: the unlock gate
/// asks, and the page says nothing of the notes behind it meanwhile.
class NotebookHomePage extends StatelessWidget {
  const NotebookHomePage({
    required this.bookId,
    this.inPane = false,
    super.key,
  });

  final String bookId;

  /// Whether it is the note pane of a wide window: there is no way back.
  final bool inPane;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<NotebookHomeBloc>(
      key: ValueKey<String>(bookId),
      create: (_) =>
          getIt<NotebookHomeBloc>()..add(NotebookHomeEvent.started(bookId)),
      child: NotebookGateHost(
        place: ListPlace.notebook(id: bookId),
        child: BlocListener<LockStatusBloc, LockStatusState>(
          listenWhen: (before, after) => before.unlocked != after.unlocked,
          listener: (context, _) => context.read<NotebookHomeBloc>().add(
            const NotebookHomeEvent.reopened(),
          ),
          child: _HomeView(bookId: bookId, inPane: inPane),
        ),
      ),
    );
  }
}

class _HomeView extends StatelessWidget {
  const _HomeView({required this.bookId, required this.inPane});

  final String bookId;
  final bool inPane;

  Future<void> _action(
    BuildContext context,
    NotebookHomeState state,
    _HomeAction action,
  ) async {
    final bloc = context.read<NotebookHomeBloc>();
    final repo = getIt<NotebookHomeRepository>();
    switch (action) {
      case _HomeAction.chooseHome:
        final found = (await repo.candidates(bookId)).toOption().toNullable();
        if (!context.mounted) {
          return;
        }
        final picked = await showHomeCandidatesSheet(
          context,
          found ?? const <HomeCandidate>[],
        );
        if (picked != null) {
          bloc.add(NotebookHomeEvent.homeChosen(picked));
        }
      case _HomeAction.newHome:
        bloc.add(const NotebookHomeEvent.newHomeRequested());
      case _HomeAction.customise:
        final found = (await repo.sections(bookId)).toOption().toNullable();
        if (found == null || !context.mounted) {
          return;
        }
        final choice = await showHomeCustomiseSheet(context, found);
        if (choice == null) {
          return;
        }
        bloc.add(
          choice.reset
              ? const NotebookHomeEvent.layoutReset()
              : NotebookHomeEvent.customised(choice.order, choice.hidden),
        );
      case _HomeAction.showHidden:
        bloc.add(const NotebookHomeEvent.hiddenShown());
    }
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    final l10n = context.l10n;
    final indexName = context.select<LibraryIndexBloc, String?>(
      (index) => index.state.notebookName(bookId),
    );
    return BlocConsumer<NotebookHomeBloc, NotebookHomeState>(
      listenWhen: (before, after) =>
          (!before.copied && after.copied) ||
          (before.madeNoteId == null && after.madeNoteId != null),
      listener: (context, state) async {
        final bloc = context.read<NotebookHomeBloc>();
        if (state.copied) {
          ScaffoldMessenger.maybeOf(context)?.showSnackBar(
            SnackBar(content: Text(l10n.notebookHomeCopiedInto)),
          );
        }
        final made = state.madeNoteId;
        bloc.add(const NotebookHomeEvent.noticeTaken());
        if (made != null) {
          // The new Home note opens; coming back reads the page again.
          await context.push(citedNoteLocation(made));
          if (context.mounted) {
            bloc.add(const NotebookHomeEvent.reopened());
          }
        }
      },
      builder: (context, state) {
        final panel = state.panel;
        final words = state.words;
        final name = (panel?.header.name.isNotEmpty ?? false)
            ? panel!.header.name
            : indexName ?? '';
        final open = panel != null && !panel.locked;
        final entries = <KansoMenuEntry<_HomeAction>>[
          if (open) ...<KansoMenuEntry<_HomeAction>>[
            KansoMenuEntry<_HomeAction>(
              _HomeAction.chooseHome,
              l10n.notebookHomeChooseHome,
              icon: KansoIcons.fileText,
            ),
            KansoMenuEntry<_HomeAction>(
              _HomeAction.newHome,
              l10n.notebookHomeNewHome,
              icon: KansoIcons.plus,
            ),
            KansoMenuEntry<_HomeAction>(
              _HomeAction.customise,
              l10n.notebookHomeCustomise,
              icon: KansoIcons.list,
            ),
            if (panel.hidden.isNotEmpty)
              KansoMenuEntry<_HomeAction>(
                _HomeAction.showHidden,
                l10n.peopleShowHidden,
                icon: KansoIcons.eye,
              ),
          ],
        ];
        return KansoPage(
          backgroundColor: colors.app,
          body: Column(
            children: <Widget>[
              KansoTopBar(
                backgroundColor: colors.app,
                title: name,
                leading: inPane ? null : const ShellBackButton(),
                trailing: <Widget>[
                  if (open && panel.aiOn && words != null)
                    KansoPressable(
                      semanticLabel: words.askButton,
                      onTap: () => context.push(
                        KansoRoutes.askIn(bookId, name),
                      ),
                      child: Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: KansoSpacing.sm,
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: <Widget>[
                            KansoIcon(
                              KansoIcons.sparkles,
                              size: 14,
                              color: colors.accent,
                            ),
                            const SizedBox(width: KansoSpacing.xxs),
                            Text(
                              words.askButton,
                              style: context.typography.label.copyWith(
                                color: colors.accent,
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                  if (open && panel.aiOn)
                    KansoIconButton(
                      icon: KansoIcons.refreshCw,
                      semanticLabel: l10n.peopleRefreshAll,
                      onPressed: () => context.read<NotebookHomeBloc>().add(
                        const NotebookHomeEvent.refreshAllRequested(),
                      ),
                    ),
                  if (entries.isNotEmpty)
                    PeopleMenuButton<_HomeAction>(
                      semanticLabel: name,
                      entries: entries,
                      onChosen: (a) => unawaited(_action(context, state, a)),
                    ),
                ],
              ),
              Expanded(child: _content(context, state, name)),
            ],
          ),
        );
      },
    );
  }

  Widget _content(BuildContext context, NotebookHomeState state, String name) {
    final panel = state.panel;
    final words = state.words;
    if (state.status == LoadStatus.failed) {
      return Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            Text(
              context.l10n.notebookHomeFailed,
              style: context.typography.secondary,
            ),
            KansoButton(
              label: context.l10n.retry,
              expanded: false,
              onPressed: () => context.read<NotebookHomeBloc>().add(
                const NotebookHomeEvent.reopened(),
              ),
            ),
          ],
        ),
      );
    }
    if (panel == null || words == null) {
      return const Center(child: KansoSpinner());
    }
    if (panel.locked) {
      // Nothing of the notebook is drawn: the gate asks for the password.
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(KansoSpacing.lg),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              KansoIcon(
                KansoIcons.lock,
                size: 28,
                color: context.colors.textMuted,
              ),
              const SizedBox(height: KansoSpacing.sm),
              Text(
                words.locked,
                textAlign: TextAlign.center,
                style: context.typography.secondary.copyWith(
                  color: context.colors.textMuted,
                ),
              ),
            ],
          ),
        ),
      );
    }
    return HomeBody(panel: panel, words: words, state: state, name: name);
  }
}

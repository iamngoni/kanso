// Kanso — replace_page.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../models/data/replace_option.dart';
import '../../models/data/replace_preview_status.dart';
import '../../models/data/replace_problem.dart';
import '../../state/replace_notes/replace_notes_bloc.dart';

/// Finds and replaces text across the library, with a single undoable batch.
class ReplacePage extends StatefulWidget {
  const ReplacePage({super.key});

  @override
  State<ReplacePage> createState() => _ReplacePageState();
}

class _ReplacePageState extends State<ReplacePage> {
  late final TextEditingController findController;
  late final TextEditingController replacementController;

  @override
  void initState() {
    super.initState();
    findController = TextEditingController();
    replacementController = TextEditingController();
  }

  @override
  void dispose() {
    findController.dispose();
    replacementController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocProvider<ReplaceNotesBloc>(
      create: (_) => getIt<ReplaceNotesBloc>(),
      child: Scaffold(
        backgroundColor: context.colors.app,
        body: Column(
          children: <Widget>[
            KansoTopBar(
              title: l10n.replaceTitle,
              leading: IconButton(
                tooltip: l10n.replaceClose,
                onPressed: () => context.pop(),
                icon: const Icon(Icons.close),
              ),
            ),
            Expanded(
              child: BlocBuilder<ReplaceNotesBloc, ReplaceNotesState>(
                builder: (context, state) {
                  final bloc = context.read<ReplaceNotesBloc>();
                  return ListView(
                    padding: const EdgeInsets.all(KansoSpacing.md),
                    children: <Widget>[
                      TextField(
                        controller: findController,
                        onChanged: (text) => bloc.add(
                          ReplaceNotesEvent.findChanged(text),
                        ),
                        decoration: InputDecoration(
                          labelText: l10n.replaceFind,
                        ),
                      ),
                      const SizedBox(height: KansoSpacing.sm),
                      TextField(
                        controller: replacementController,
                        onChanged: (text) => bloc.add(
                          ReplaceNotesEvent.replacementChanged(text),
                        ),
                        decoration: InputDecoration(
                          labelText: l10n.replaceWith,
                        ),
                      ),
                      const SizedBox(height: KansoSpacing.sm),
                      Text(
                        l10n.replaceOptionsLabel,
                        style: context.typography.heading2,
                      ),
                      for (final option in ReplaceOption.values)
                        SwitchListTile(
                          contentPadding: EdgeInsets.zero,
                          title: Text(switch (option) {
                            ReplaceOption.matchCase => l10n.replaceMatchCase,
                            ReplaceOption.wholeWord => l10n.replaceWholeWord,
                            ReplaceOption.regex => l10n.replaceRegex,
                            ReplaceOption.skipCode => l10n.replaceSkipCode,
                            ReplaceOption.includeTrash =>
                              l10n.replaceIncludeTrash,
                          }),
                          value: switch (option) {
                            ReplaceOption.matchCase => state.options.matchCase,
                            ReplaceOption.wholeWord => state.options.wholeWord,
                            ReplaceOption.regex => state.options.regex,
                            ReplaceOption.skipCode => state.options.skipCode,
                            ReplaceOption.includeTrash =>
                              state.options.includeTrash,
                          },
                          onChanged: (_) => bloc.add(
                            ReplaceNotesEvent.optionToggled(option),
                          ),
                        ),
                      const Divider(),
                      if (state.problem != null)
                        Text(switch (state.problem!) {
                          ReplaceProblem.badPattern =>
                            l10n.replaceProblemBadPattern,
                          ReplaceProblem.applyFailed =>
                            l10n.replaceProblemApply,
                          ReplaceProblem.undoFailed => l10n.replaceProblemUndo,
                        }, style: TextStyle(color: context.colors.danger)),
                      if (state.outcome case final outcome?) ...<Widget>[
                        Text(l10n.replaceDone(outcome.replaced, outcome.notes)),
                        if (outcome.skipped > 0)
                          Text(l10n.replaceSkipped(outcome.skipped)),
                        if (state.undone == null)
                          TextButton(
                            onPressed: state.undoing
                                ? null
                                : () => bloc.add(
                                    const ReplaceNotesEvent.undoRequested(),
                                  ),
                            child: Text(
                              state.undoing
                                  ? l10n.replaceUndoing
                                  : l10n.replaceUndo,
                            ),
                          )
                        else
                          Text(l10n.replaceUndone(state.undone!.restored)),
                      ],
                      if (state.previewStatus == ReplacePreviewStatus.loading)
                        Semantics(
                          label: l10n.replaceSearching,
                          child: const LinearProgressIndicator(),
                        )
                      else if (state.options.find.isEmpty)
                        Text(l10n.replaceTypeToFind)
                      else if (state.previewStatus ==
                          ReplacePreviewStatus.failed)
                        Text(
                          l10n.replaceProblemSearch,
                          style: TextStyle(color: context.colors.danger),
                        )
                      else if (state.preview case final preview?) ...<Widget>[
                        Text(
                          l10n.replaceFound(
                            preview.totalMatches,
                            preview.notes.length,
                          ),
                          style: context.typography.heading2,
                        ),
                        for (final note in preview.notes)
                          CheckboxListTile(
                            contentPadding: EdgeInsets.zero,
                            value: !state.excluded.contains(note.id),
                            title: Text(
                              note.title.isEmpty
                                  ? l10n.replaceUntitled
                                  : note.title,
                            ),
                            subtitle: Column(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: <Widget>[
                                if (note.trashed) Text(l10n.replaceInTrash),
                                for (final match in note.matches.take(3))
                                  Text(l10n.replaceLine(match.lineNumber)),
                                if (note.matches.length > 3)
                                  Text(
                                    l10n.replaceMore(note.matches.length - 3),
                                  ),
                              ],
                            ),
                            onChanged: (_) => bloc.add(
                              ReplaceNotesEvent.noteToggled(note.id),
                            ),
                          ),
                        FilledButton(
                          onPressed: state.planned.isEmpty || state.applying
                              ? null
                              : () => bloc.add(
                                  const ReplaceNotesEvent.applyRequested(),
                                ),
                          child: Text(
                            state.applying
                                ? l10n.replaceApplying
                                : l10n.replaceApply(
                                    state.matches,
                                    state.chosen.length,
                                  ),
                          ),
                        ),
                      ] else if (state.previewStatus ==
                          ReplacePreviewStatus.ready)
                        Text(l10n.replaceNoMatches),
                    ],
                  );
                },
              ),
            ),
          ],
        ),
      ),
    );
  }
}

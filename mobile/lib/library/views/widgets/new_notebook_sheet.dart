// Kanso — new_notebook_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_text_input.dart';
import '../../../injection.dart';
import '../../models/data/new_notebook_failure.dart';
import '../../state/new_notebook/new_notebook_bloc.dart';

/// A sheet with a name to type, that makes a notebook at the top of the
/// library and goes to it.
class NewNotebookSheet extends StatefulWidget {
  const NewNotebookSheet({super.key});

  /// Shows the sheet over the whole app; when a notebook is made, closes
  /// the library's drawer and goes to the notebook's list.
  static Future<void> open(BuildContext context) async {
    final router = GoRouter.of(context);
    final drawer = KansoDrawerScope.maybeOf(context);
    final made = await showKansoBottomSheet<NotebookRow>(
      context,
      builder: (_) => const NewNotebookSheet(),
    );
    if (made != null) {
      drawer?.close();
      router.go(BrowseLocation.place(ListPlace.notebook(id: made.id)).location);
    }
  }

  @override
  State<NewNotebookSheet> createState() => _NewNotebookSheetState();
}

class _NewNotebookSheetState extends State<NewNotebookSheet> {
  late final NewNotebookBloc bloc;
  final TextEditingController name = TextEditingController();
  final FocusNode focus = FocusNode(debugLabel: 'New notebook name');

  @override
  void initState() {
    super.initState();
    bloc = getIt<NewNotebookBloc>();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        focus.requestFocus();
      }
    });
  }

  @override
  void dispose() {
    name.dispose();
    focus.dispose();
    unawaited(bloc.close());
    super.dispose();
  }

  void submit(String value) => bloc.add(NewNotebookEvent.submitted(value));

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    // The keyboard comes up under the sheet, which already has the home
    // indicator's room.
    final keyboard = math.max<double>(
      0,
      MediaQuery.viewInsetsOf(context).bottom -
          MediaQuery.paddingOf(context).bottom,
    );
    return BlocProvider<NewNotebookBloc>.value(
      value: bloc,
      child: BlocListener<NewNotebookBloc, NewNotebookState>(
        listenWhen: (before, now) => now.created != null,
        listener: (context, state) => Navigator.of(context).pop(state.created),
        child: Padding(
          padding: EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            KansoSpacing.md + keyboard,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              Row(
                children: <Widget>[
                  Expanded(
                    child: Text(
                      l10n.newNotebookTitle,
                      style: context.typography.barTitle,
                    ),
                  ),
                  KansoPressable(
                    onTap: () => Navigator.of(context).maybePop(),
                    child: Padding(
                      padding: const EdgeInsets.all(KansoSpacing.sm),
                      child: Text(
                        l10n.noteListCancel,
                        style: context.typography.body.copyWith(
                          color: colors.accent,
                        ),
                      ),
                    ),
                  ),
                ],
              ),
              const SizedBox(height: KansoSpacing.xs),
              KansoTextInput(
                controller: name,
                focusNode: focus,
                hint: l10n.newNotebookNameHint,
                semanticLabel: l10n.newNotebookNameHint,
                textCapitalization: TextCapitalization.words,
                onSubmitted: submit,
              ),
              BlocBuilder<NewNotebookBloc, NewNotebookState>(
                builder: (context, state) {
                  final failure = state.failure;
                  if (failure == null) {
                    return const SizedBox(height: KansoSpacing.sm);
                  }
                  return Semantics(
                    liveRegion: true,
                    child: Padding(
                      padding: const EdgeInsets.symmetric(
                        vertical: KansoSpacing.xs,
                      ),
                      child: Text(
                        switch (failure) {
                          NewNotebookFailure.invalidName =>
                            l10n.newNotebookInvalid,
                          NewNotebookFailure.failed => l10n.newNotebookFailed,
                        },
                        style: context.typography.secondary.copyWith(
                          color: colors.danger,
                        ),
                      ),
                    ),
                  );
                },
              ),
              ListenableBuilder(
                listenable: name,
                builder: (context, _) {
                  return KansoButton(
                    label: l10n.newNotebookCreate,
                    onPressed: name.text.trim().isEmpty
                        ? null
                        : () => submit(name.text),
                  );
                },
              ),
            ],
          ),
        ),
      ),
    );
  }
}

// Kanso — new_notebook_sheet.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_drawer_controller.dart';
import '../../../core/views/widgets/kanso_drawer_scope.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_text_input.dart';
import '../../../injection.dart';
import '../../models/data/new_notebook_failure.dart';
import '../../state/new_notebook/new_notebook_bloc.dart';
import 'new_notebook_kind_picker.dart';

/// A sheet with a name to type and a starter to choose, that makes a notebook
/// at the top of the library, or inside [parent], and goes to it.
class NewNotebookSheet extends StatefulWidget {
  const NewNotebookSheet({this.parent, super.key});

  /// The notebook the new one is made inside, if any.
  final NotebookRow? parent;

  /// Shows the sheet over the whole app; when a notebook is made, closes
  /// the library's drawer and goes to the notebook's list. Made inside
  /// [parent], [index] is told that the parent changed. A [drawer] given
  /// is the one to close, for a [context] that is not inside it.
  static Future<void> open(
    BuildContext context, {
    NotebookRow? parent,
    LibraryIndexBloc? index,
    KansoDrawerController? drawer,
  }) async {
    final router = GoRouter.of(context);
    final closing = drawer ?? KansoDrawerScope.maybeOf(context);
    final made = await showKansoBottomSheet<MadeNotebook>(
      context,
      builder: (_) => NewNotebookSheet(parent: parent),
    );
    if (made != null) {
      if (parent != null) {
        index?.add(LibraryIndexEvent.changesReceived(<String>[parent.id]));
      }
      closing?.close();
      router.go(
        BrowseLocation.place(
          ListPlace.notebook(id: made.notebook.id),
        ).location,
      );
    }
  }

  @override
  State<NewNotebookSheet> createState() => _NewNotebookSheetState();
}

class _NewNotebookSheetState extends State<NewNotebookSheet> {
  late final NewNotebookBloc bloc;
  final TextEditingController name = TextEditingController();
  final FocusNode focus = FocusNode(debugLabel: 'New notebook name');
  NewNotebookKind kind = NewNotebookKind.blank;

  @override
  void initState() {
    super.initState();
    bloc = getIt<NewNotebookBloc>()
      ..add(NewNotebookEvent.opened(parentId: widget.parent?.id));
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

  void submit(String value) =>
      bloc.add(NewNotebookEvent.submitted(value, kind: kind));

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return BlocProvider<NewNotebookBloc>.value(
      value: bloc,
      child: BlocListener<NewNotebookBloc, NewNotebookState>(
        listenWhen: (before, now) => now.created != null,
        listener: (context, state) => Navigator.of(context).pop(state.created),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            KansoSpacing.md,
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
                          NewNotebookFailure.lockedParent =>
                            l10n.newNotebookStarterLocked,
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
              // The starters are many and tall, and with the keyboard up on a
              // phone they don't fit: they scroll, with the name above and
              // Create below in view.
              Flexible(
                child: BlocBuilder<NewNotebookBloc, NewNotebookState>(
                  buildWhen: (before, now) =>
                      before.starters != now.starters ||
                      before.parentLocked != now.parentLocked,
                  builder: (context, state) => SingleChildScrollView(
                    child: NewNotebookKindPicker(
                      starters: state.starters,
                      kind: kind,
                      lockedParent: state.parentLocked,
                      onChanged: (chosen) => setState(() => kind = chosen),
                    ),
                  ),
                ),
              ),
              const SizedBox(height: KansoSpacing.md),
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

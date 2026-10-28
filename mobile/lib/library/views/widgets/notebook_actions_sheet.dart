// Kanso — notebook_actions_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../injection.dart';
import '../../../lock/state/lock_status/lock_status_bloc.dart';
import '../../../publish/views/widgets/publish_sheet.dart';
import 'library_actions.dart';
import 'library_color_sheets.dart';
import 'library_text_prompt_sheet.dart';
import 'notebook_icon_sheet.dart';
import 'notebook_lock_actions.dart';

/// One action of a notebook's menu.
class NotebookSheetAction {
  const NotebookSheetAction({
    required this.label,
    required this.icon,
    required this.onTap,
    this.destructive = false,
  });

  final String label;
  final KansoIcons icon;
  final VoidCallback onTap;

  /// Whether it is drawn in the danger colour.
  final bool destructive;
}

/// A notebook's menu, opened by a long press in the library: its name and
/// the desktop's actions: its lock (when it has one to add or take off),
/// a notebook inside it, renaming it, its icon and colour, and deleting it.
class NotebookActionsSheet extends StatelessWidget {
  const NotebookActionsSheet({
    required this.notebook,
    required this.index,
    required this.host,
    super.key,
  });

  final NotebookRow notebook;

  /// The library's index, which reads again after a change.
  final LibraryIndexBloc index;

  /// The root navigator's context, which outlives the menu: the sheets a
  /// choice shows are opened from it, after the menu has closed.
  final BuildContext host;

  /// Shows the menu of [notebook] over the whole app.
  static Future<void> show(BuildContext context, NotebookRow notebook) {
    final index = context.read<LibraryIndexBloc>();
    final status = context.read<LockStatusBloc>();
    final host = Navigator.of(context, rootNavigator: true).overlay!.context;
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => MultiBlocProvider(
        providers: <BlocProvider<dynamic>>[
          BlocProvider<LibraryIndexBloc>.value(value: index),
          BlocProvider<LockStatusBloc>.value(value: status),
        ],
        child: NotebookActionsSheet(
          notebook: notebook,
          index: index,
          host: host,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final locks = notebookLockActions(context, notebook);

    void closeThen(Future<void> Function() action) {
      Navigator.of(context).pop();
      action();
    }

    Future<void> newInside() async {
      final name = await LibraryTextPromptSheet.show(
        host,
        title: l10n.newNotebookTitle,
        confirmLabel: l10n.newNotebookCreate,
        hint: l10n.libraryNotebookNameHint,
      );
      if (name == null) {
        return;
      }
      await runLibraryAction(
        host,
        index: index,
        action: () => getIt<LibraryRepository>().createNotebook(
          name,
          parentId: notebook.id,
        ),
        changed: <String>[notebook.id],
      );
    }

    Future<void> rename() async {
      final name = await LibraryTextPromptSheet.show(
        host,
        title: l10n.libraryRenameNotebookTitle,
        confirmLabel: l10n.librarySaveAction,
        initial: notebook.name,
        hint: l10n.libraryNotebookNameHint,
      );
      if (name == null || name == notebook.name) {
        return;
      }
      await runLibraryAction(
        host,
        index: index,
        action: () =>
            getIt<LibraryRepository>().renameNotebook(notebook.id, name),
        changed: <String>[notebook.id],
      );
    }

    Future<void> reicon() async {
      final pick = await NotebookIconSheet.show(host, current: notebook.icon);
      if (pick == null) {
        return;
      }
      await runLibraryAction(
        host,
        index: index,
        action: () =>
            getIt<LibraryRepository>().setNotebookIcon(notebook.id, pick.name),
        changed: <String>[notebook.id],
      );
    }

    Future<void> recolor() async {
      final pick = await NotebookColorSheet.show(host, current: notebook.color);
      if (pick == null) {
        return;
      }
      await runLibraryAction(
        host,
        index: index,
        action: () =>
            getIt<LibraryRepository>().setNotebookColor(notebook.id, pick.hex),
        changed: <String>[notebook.id],
      );
    }

    Future<void> delete() async {
      final yes = await KansoConfirmSheet.show(
        host,
        title: l10n.libraryDeleteNotebookTitle(notebook.name),
        message: l10n.libraryDeleteNotebookMessage,
        confirmLabel: l10n.libraryDeleteAction,
      );
      if (!yes) {
        return;
      }
      await runLibraryAction(
        host,
        index: index,
        action: () => getIt<LibraryRepository>().deleteNotebook(notebook.id),
        changed: <String>[notebook.id],
      );
    }

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.fromSTEB(
            KansoSpacing.md,
            KansoSpacing.sm,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Text(
            notebook.name,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: context.typography.barTitle,
          ),
        ),
        if (locks.isNotEmpty)
          KansoSheetGroup(
            dividerIndent: KansoSettingsRow.iconInset,
            children: <Widget>[
              for (final action in locks)
                KansoSettingsRow(
                  icon: action.icon,
                  label: action.label,
                  danger: action.destructive,
                  onTap: () => closeThen(() async => action.onTap()),
                ),
            ],
          ),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.externalLink,
              label: l10n.publishNotebookMenu,
              onTap: () => closeThen(
                () => PublishSheet.showNotebook(host, notebook.id),
              ),
            ),
          ],
        ),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.plus,
              label: l10n.libraryNewSubNotebook,
              onTap: () => closeThen(newInside),
            ),
            KansoSettingsRow(
              icon: KansoIcons.pencil,
              label: l10n.libraryRenameNotebook,
              onTap: () => closeThen(rename),
            ),
            KansoSettingsRow(
              icon: KansoIcons.shapes,
              label: l10n.libraryChangeNotebookIcon,
              onTap: () => closeThen(reicon),
            ),
            KansoSettingsRow(
              icon: KansoIcons.palette,
              label: l10n.libraryChangeNotebookColor,
              onTap: () => closeThen(recolor),
            ),
          ],
        ),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.trash,
              label: l10n.libraryDeleteNotebook,
              danger: true,
              onTap: () => closeThen(delete),
            ),
          ],
        ),
        const SizedBox(height: KansoSpacing.sm),
      ],
    );
  }
}

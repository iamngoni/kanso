// Kanso — tag_actions_sheet.dart
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
import 'library_actions.dart';
import 'library_color_sheets.dart';
import 'library_text_prompt_sheet.dart';

/// A tag's menu, opened by a long press on its chip or its row in the tree:
/// its name and the desktop's actions, Rename…, Color and Delete Tag…, with
/// the questions they ask.
class TagActionsSheet extends StatelessWidget {
  const TagActionsSheet({
    required this.tag,
    required this.index,
    required this.host,
    super.key,
  });

  final TagRow tag;

  /// The library's index, which reads again after a change.
  final LibraryIndexBloc index;

  /// The root navigator's context, which outlives the menu: the sheets a
  /// choice shows are opened from it, after the menu has closed.
  final BuildContext host;

  /// Shows the menu of [tag] over the whole app.
  static Future<void> show(BuildContext context, TagRow tag) {
    final index = context.read<LibraryIndexBloc>();
    final host = Navigator.of(context, rootNavigator: true).overlay!.context;
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => BlocProvider<LibraryIndexBloc>.value(
        value: index,
        child: TagActionsSheet(tag: tag, index: index, host: host),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final library = getIt<LibraryRepository>();
    Future<void> rename() async {
      final name = await LibraryTextPromptSheet.show(
        host,
        title: l10n.libraryRenameTagTitle,
        confirmLabel: l10n.librarySaveAction,
        initial: tag.name,
        hint: l10n.libraryTagNameHint,
      );
      if (name == null || name == tag.name) {
        return;
      }
      await runLibraryAction(
        host,
        index: index,
        action: () => library.renameTag(tag.id, name),
        changed: <String>[tag.id],
      );
    }

    Future<void> recolor() async {
      final color = await TagColorSheet.show(host, current: tag.color);
      if (color == null) {
        return;
      }
      await runLibraryAction(
        host,
        index: index,
        action: () => library.setTagColor(tag.id, color),
        changed: <String>[tag.id],
      );
    }

    Future<void> delete() async {
      final yes = await KansoConfirmSheet.show(
        host,
        title: l10n.libraryDeleteTagTitle(tag.name),
        message: l10n.libraryDeleteTagMessage,
        confirmLabel: l10n.libraryDeleteAction,
      );
      if (!yes) {
        return;
      }
      await runLibraryAction(
        host,
        index: index,
        action: () => library.deleteTag(tag.id),
        changed: <String>[tag.id],
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
            '#${tag.name}',
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: context.typography.barTitle,
          ),
        ),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.pencil,
              label: l10n.libraryRenameTag,
              onTap: () {
                Navigator.of(context).pop();
                rename();
              },
            ),
            KansoSettingsRow(
              icon: KansoIcons.palette,
              label: l10n.libraryTagColor,
              chevron: true,
              onTap: () {
                Navigator.of(context).pop();
                recolor();
              },
            ),
          ],
        ),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            KansoSettingsRow(
              icon: KansoIcons.trash,
              label: l10n.libraryDeleteTag,
              danger: true,
              onTap: () {
                Navigator.of(context).pop();
                delete();
              },
            ),
          ],
        ),
        const SizedBox(height: KansoSpacing.sm),
      ],
    );
  }
}

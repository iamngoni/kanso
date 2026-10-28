// Kanso — note_actions_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/repos/abs/library_repository.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_check_row.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../injection.dart';
import '../../../publish/views/widgets/publish_sheet.dart';
import '../../state/note_list/note_list_bloc.dart';

/// What can be done with the notes chosen, beyond the bar's Pin, Move and
/// Trash, as the desktop's note menu has it: their status, a copy, their
/// links and, in the trash, restoring them or deleting them for good.
class NoteActionsSheet extends StatelessWidget {
  const NoteActionsSheet({required this.ids, super.key});

  /// The notes the menu acts on.
  final List<String> ids;

  /// Shows the menu for [ids] over the whole app. [bloc] is the list the
  /// notes are in.
  static Future<void> show(
    BuildContext context, {
    required NoteListBloc bloc,
    required List<String> ids,
  }) {
    final index = context.read<LibraryIndexBloc>();
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => MultiBlocProvider(
        providers: <BlocProvider<dynamic>>[
          BlocProvider<NoteListBloc>.value(value: bloc),
          BlocProvider<LibraryIndexBloc>.value(value: index),
        ],
        child: NoteActionsSheet(ids: ids),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = context.read<NoteListBloc>();
    final state = bloc.state;
    final library = getIt<LibraryRepository>();
    final chosen = <NoteRow>[
      for (final row in state.rows)
        if (ids.contains(row.id)) row,
    ];
    final single = chosen.length == 1 ? chosen.single : null;
    final inTrash = state.inTrash;
    final host = Navigator.of(context, rootNavigator: true).overlay!.context;

    void say(String message) {
      ScaffoldMessenger.maybeOf(host)?.showSnackBar(
        SnackBar(content: Text(message)),
      );
    }

    void change(NoteChange change) {
      bloc.add(NoteListEvent.changeRequested(change, ids));
    }

    void closeThen(Future<void> Function() action) {
      Navigator.of(context).pop();
      action();
    }

    Future<void> status() async {
      final picked = await NoteStatusSheet.show(host, current: single?.status);
      if (picked == null) {
        return;
      }
      change(NoteChange.setStatus(status: picked));
    }

    Future<void> duplicate() async {
      final one = single;
      if (one == null) {
        return;
      }
      final result = await library.duplicate(one.id);
      result.fold(
        (error) => say(l10n.noteListChangeFailed),
        (saved) => bloc.add(NoteListEvent.changesReceived(<String>[saved.id])),
      );
    }

    Future<void> copyLink({required bool kanso}) async {
      final one = single;
      if (one == null) {
        return;
      }
      final result = await library.noteLink(one.id, kanso: kanso);
      await result.fold(
        (error) async => say(l10n.noteListChangeFailed),
        (link) async {
          await Clipboard.setData(ClipboardData(text: link.text));
          say(link.message);
        },
      );
    }

    Future<void> deleteForever() async {
      Navigator.of(context).pop();
      final yes = await KansoConfirmSheet.show(
        host,
        title: l10n.noteActionDeleteForeverTitle,
        message: l10n.noteActionDeleteForeverMessage,
        confirmLabel: l10n.noteActionDeleteForever,
      );
      if (yes) {
        change(const NoteChange.deleteForever());
      }
    }

    final title = single?.title.trim() ?? '';
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
            single == null || title.isEmpty
                ? l10n.noteListSelected(ids.length)
                : title,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: context.typography.barTitle,
          ),
        ),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            if (inTrash)
              KansoSettingsRow(
                icon: KansoIcons.undo,
                label: l10n.noteActionRestore,
                onTap: () {
                  Navigator.of(context).pop();
                  change(const NoteChange.restore());
                },
              )
            else
              KansoSettingsRow(
                icon: KansoIcons.circleDot,
                label: l10n.noteActionStatus,
                chevron: true,
                onTap: () => closeThen(status),
              ),
          ],
        ),
        if (single != null)
          KansoSheetGroup(
            dividerIndent: KansoSettingsRow.iconInset,
            children: <Widget>[
              KansoSettingsRow(
                icon: KansoIcons.copy,
                label: l10n.noteActionDuplicate,
                onTap: () => closeThen(duplicate),
              ),
              KansoSettingsRow(
                icon: KansoIcons.link,
                label: l10n.noteActionCopyLink,
                onTap: () => closeThen(() => copyLink(kanso: false)),
              ),
              KansoSettingsRow(
                icon: KansoIcons.hash,
                label: l10n.noteActionCopyKansoLink,
                onTap: () => closeThen(() => copyLink(kanso: true)),
              ),
              if (!inTrash)
                KansoSettingsRow(
                  icon: KansoIcons.externalLink,
                  label: l10n.publishNoteMenu,
                  onTap: () => closeThen(
                    () => PublishSheet.showNote(host, single.id),
                  ),
                ),
            ],
          ),
        if (inTrash)
          KansoSheetGroup(
            dividerIndent: KansoSettingsRow.iconInset,
            children: <Widget>[
              KansoSettingsRow(
                icon: KansoIcons.trash,
                label: l10n.noteActionDeleteForever,
                danger: true,
                onTap: deleteForever,
              ),
            ],
          ),
        const SizedBox(height: KansoSpacing.sm),
      ],
    );
  }
}

/// The statuses a note can be given, with the one it has now checked.
class NoteStatusSheet extends StatelessWidget {
  const NoteStatusSheet({this.current, super.key});

  /// The status the note has now, if it is one note.
  final String? current;

  /// Shows the statuses over the whole app; the status chosen, or null when
  /// it was closed without a choice.
  static Future<String?> show(BuildContext context, {String? current}) {
    final index = context.read<LibraryIndexBloc>();
    return showKansoBottomSheet<String>(
      context,
      builder: (_) => BlocProvider<LibraryIndexBloc>.value(
        value: index,
        child: NoteStatusSheet(current: current),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final statuses = context.select<LibraryIndexBloc, List<StatusChoice>>(
      (index) => index.state.statuses,
    );
    return ListView(
      shrinkWrap: true,
      padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
      children: <Widget>[
        Padding(
          padding: const EdgeInsetsDirectional.only(
            start: KansoSpacing.md,
            top: KansoSpacing.xs,
          ),
          child: Row(
            children: <Widget>[
              Expanded(
                child: Text(
                  l10n.noteActionStatus,
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
        ),
        KansoSheetGroup(
          children: <Widget>[
            for (final choice in statuses)
              KansoCheckRow(
                label: choice.label,
                checked: choice.status == current,
                onTap: () => Navigator.of(context).pop(choice.status),
              ),
          ],
        ),
      ],
    );
  }
}

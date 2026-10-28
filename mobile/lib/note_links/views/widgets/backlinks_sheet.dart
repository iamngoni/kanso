// Kanso — backlinks_sheet.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/linking_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_empty_state.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_section_header.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../injection.dart';
import '../../models/repos/abs/note_links_repository.dart';

/// The notes that name the open note, and the ones that name it without
/// linking to it, in a sheet. A line opens the note it is on.
class BacklinksSheet extends StatefulWidget {
  const BacklinksSheet({required this.noteId, this.links, super.key});

  final String noteId;

  /// What reads the links; the app's own when not given.
  final NoteLinksRepository? links;

  /// Shows the links to [noteId] over the whole app.
  static Future<void> show(BuildContext context, {required String noteId}) {
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => BacklinksSheet(noteId: noteId),
    );
  }

  @override
  State<BacklinksSheet> createState() => _BacklinksSheetState();
}

class _BacklinksSheetState extends State<BacklinksSheet> {
  List<LinkRef>? linked;
  List<LinkRef>? unlinked;
  ApplicationError? problem;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  Future<void> _load() async {
    final repository = widget.links ?? getIt<NoteLinksRepository>();
    final backs = await repository.backlinks(widget.noteId);
    final mentions = await repository.unlinkedMentions(widget.noteId);
    if (!mounted) {
      return;
    }
    backs.fold(
      (error) => setState(() => problem = error),
      (rows) => setState(() => linked = rows),
    );
    mentions.fold(
      (error) => setState(() => problem = error),
      (rows) => setState(() => unlinked = rows),
    );
  }

  void _open(BuildContext context, LinkRef ref) {
    Navigator.of(context).pop();
    final router = GoRouter.of(context);
    unawaited(
      router.push(
        '${KansoRoutes.notes}/note/${Uri.encodeComponent(ref.id)}'
        '?${KansoRoutes.taskLineParameter}=${ref.line}',
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final problem = this.problem;
    final linked = this.linked;
    final unlinked = this.unlinked;
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
          child: Text(l10n.noteBacklinks, style: context.typography.barTitle),
        ),
        Flexible(
          child: switch ((problem, linked, unlinked)) {
            (final ApplicationError error, _, _) => Padding(
              padding: const EdgeInsets.all(KansoSpacing.md),
              child: KansoInlineMessage(message: error.message),
            ),
            (_, null, _) || (_, _, null) => const SizedBox(
              height: KansoSpacing.md,
            ),
            (_, final List<LinkRef> links, final List<LinkRef> mentions)
                when links.isEmpty && mentions.isEmpty =>
              Padding(
                padding: const EdgeInsets.all(KansoSpacing.md),
                child: KansoEmptyState(
                  title: l10n.noteBacklinksEmptyTitle,
                  message: l10n.noteBacklinksEmptyMessage,
                ),
              ),
            (_, final List<LinkRef> links, final List<LinkRef> mentions) =>
              ListView(
                shrinkWrap: true,
                padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
                children: <Widget>[
                  if (links.isNotEmpty) ...<Widget>[
                    KansoSectionHeader(label: l10n.noteBacklinksLinked),
                    KansoSheetGroup(
                      dividerIndent: KansoSettingsRow.iconInset,
                      children: <Widget>[
                        for (final ref in links)
                          KansoSettingsRow(
                            label: ref.title,
                            value: ref.context,
                            onTap: () => _open(context, ref),
                          ),
                      ],
                    ),
                  ],
                  if (mentions.isNotEmpty) ...<Widget>[
                    KansoSectionHeader(label: l10n.noteBacklinksUnlinked),
                    KansoSheetGroup(
                      dividerIndent: KansoSettingsRow.iconInset,
                      children: <Widget>[
                        for (final ref in mentions)
                          KansoSettingsRow(
                            label: ref.title,
                            value: ref.context,
                            onTap: () => _open(context, ref),
                          ),
                      ],
                    ),
                  ],
                ],
              ),
          },
        ),
      ],
    );
  }
}

// Kanso — related_notes_sheet.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart' hide State;
import 'package:flutter/material.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../discovery/models/repos/abs/discovery_repository.dart';
import '../../../injection.dart';
import '../../../journal/utils/open_made_note.dart';

/// Related notes the open note does not link to yet.
class RelatedNotesSheet extends StatefulWidget {
  const RelatedNotesSheet({required this.noteId, this.discovery, super.key});

  final String noteId;

  /// What reads related notes; the app's own when not given.
  final DiscoveryRepository? discovery;

  /// Shows the suggestions for [noteId] over the whole app.
  static Future<void> show(BuildContext context, {required String noteId}) {
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => RelatedNotesSheet(noteId: noteId),
    );
  }

  @override
  State<RelatedNotesSheet> createState() => _RelatedNotesSheetState();
}

class _RelatedNotesSheetState extends State<RelatedNotesSheet> {
  late final Future<
    (
      Either<ApplicationError, bool>,
      Either<ApplicationError, List<RelatedRow>>?,
    )
  > _related;

  @override
  void initState() {
    super.initState();
    _related = _load();
  }

  Future<
    (
      Either<ApplicationError, bool>,
      Either<ApplicationError, List<RelatedRow>>?,
    )
  > _load() async {
    final repository = widget.discovery ?? getIt<DiscoveryRepository>();
    final enabled = await repository.relatedEnabled();
    final on = enabled.fold((_) => false, (value) => value);
    final Either<ApplicationError, List<RelatedRow>>? related = on
        ? await repository.relatedNotes(widget.noteId, limit: 20)
        : null;
    return (enabled, related);
  }

  void _open(BuildContext context, RelatedRow row) {
    Navigator.of(context).pop();
    openNoteById(context, row.id);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
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
            l10n.noteRelatedTitle,
            style: context.typography.barTitle,
          ),
        ),
        Flexible(
          child: FutureBuilder<
            (
              Either<ApplicationError, bool>,
              Either<ApplicationError, List<RelatedRow>>?,
            )
          >(
            future: _related,
            builder: (context, snapshot) {
              if (!snapshot.hasData) {
                return const SizedBox(
                  height: KansoSpacing.xl,
                  child: Center(child: CircularProgressIndicator()),
                );
              }
              final (enabled, related) = snapshot.requireData;
              return enabled.fold(
                (error) => _message(KansoInlineMessage(message: error.message)),
                (on) {
                  if (!on) {
                    return _message(Text(l10n.noteRelatedTurnOn));
                  }
                  return related!.fold(
                    (error) => _message(
                      KansoInlineMessage(message: error.message),
                    ),
                    (rows) => rows.isEmpty
                        ? _message(Text(l10n.noteRelatedEmpty))
                        : ListView(
                            shrinkWrap: true,
                            padding: const EdgeInsets.only(
                              bottom: KansoSpacing.sm,
                            ),
                            children: <Widget>[
                              KansoSheetGroup(
                                dividerIndent: KansoSettingsRow.iconInset,
                                children: <Widget>[
                                  for (final row in rows)
                                    KansoSettingsRow(
                                      label: row.title,
                                      value: row.label,
                                      chevron: true,
                                      onTap: () => _open(context, row),
                                    ),
                                ],
                              ),
                            ],
                          ),
                  );
                },
              );
            },
          ),
        ),
        const SizedBox(height: KansoSpacing.sm),
      ],
    );
  }

  Widget _message(Widget child) => Padding(
    padding: const EdgeInsets.all(KansoSpacing.md),
    child: child,
  );
}

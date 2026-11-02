// Kanso — related_notes_sheet.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart' hide State;
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_inline_message.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../../discovery/models/repos/abs/discovery_repository.dart';
import '../../../injection.dart';
import '../../../journal/utils/open_made_note.dart';
import '../../state/resurface/resurface_bloc.dart';
import 'resurface_item.dart';

/// Related notes: the ones about the paragraph being written (with a way to
/// link to each and to say it is not related), "On this day" on a daily
/// note, and the ones the open note does not link to yet.
class RelatedNotesSheet extends StatefulWidget {
  const RelatedNotesSheet({
    required this.noteId,
    this.discovery,
    this.resurface,
    this.onLink,
    super.key,
  });

  final String noteId;

  /// What reads related notes; the app's own when not given.
  final DiscoveryRepository? discovery;

  /// What found the notes about the paragraph; none shows only the notes
  /// about the whole note.
  final ResurfaceBloc? resurface;

  /// Puts a link to the note with this title at the caret; without it, the
  /// notes have no Link action.
  final ValueChanged<String>? onLink;

  /// Shows the suggestions for [noteId] over the whole app.
  static Future<void> show(
    BuildContext context, {
    required String noteId,
    ResurfaceBloc? resurface,
    ValueChanged<String>? onLink,
  }) {
    return showKansoBottomSheet<void>(
      context,
      builder: (_) => RelatedNotesSheet(
        noteId: noteId,
        resurface: resurface,
        onLink: onLink,
      ),
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
  >
  _related;

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
  >
  _load() async {
    final repository = widget.discovery ?? getIt<DiscoveryRepository>();
    final enabled = await repository.relatedEnabled();
    final on = enabled.fold((_) => false, (value) => value);
    final Either<ApplicationError, List<RelatedRow>>? related = on
        ? await repository.relatedNotes(widget.noteId, limit: 20)
        : null;
    return (enabled, related);
  }

  void _open(BuildContext context, String id) {
    Navigator.of(context).pop();
    openNoteById(context, id);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final bloc = widget.resurface;
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
          child: ListView(
            shrinkWrap: true,
            padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
            children: <Widget>[
              if (bloc != null)
                BlocBuilder<ResurfaceBloc, ResurfaceState>(
                  bloc: bloc,
                  builder: (context, state) => _paragraph(context, state, bloc),
                ),
              _aboutNote(context),
            ],
          ),
        ),
        const SizedBox(height: KansoSpacing.sm),
      ],
    );
  }

  /// "On this day", and the notes about the paragraph.
  Widget _paragraph(
    BuildContext context,
    ResurfaceState state,
    ResurfaceBloc bloc,
  ) {
    final l10n = context.l10n;
    final widgets = <Widget>[
      if (state.day.isNotEmpty) ...<Widget>[
        _heading(context, l10n.resurfaceOnThisDay),
        KansoSheetGroup(
          dividerIndent: KansoSettingsRow.iconInset,
          children: <Widget>[
            for (final note in state.day)
              KansoSettingsRow(
                label: note.title.isEmpty ? l10n.noteUntitled : note.title,
                value: note.ago,
                chevron: true,
                onTap: () => _open(context, note.id),
              ),
          ],
        ),
      ],
      if (state.enabled && state.rows.isNotEmpty) ...<Widget>[
        _heading(context, l10n.resurfaceAboutParagraph),
        KansoSheetGroup(
          children: <Widget>[
            for (final row in state.rows)
              ResurfaceItem(
                row: row,
                onOpen: () => _open(context, row.id),
                // A note with no title has nothing to link to.
                onLink: widget.onLink == null || row.title.trim().isEmpty
                    ? null
                    : () {
                        Navigator.of(context).pop();
                        widget.onLink!(row.title);
                      },
                onNotRelated: () => bloc.add(ResurfaceEvent.dismissed(row.id)),
              ),
          ],
        ),
      ],
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: widgets,
    );
  }

  Widget _heading(BuildContext context, String text) => Padding(
    padding: const EdgeInsetsDirectional.fromSTEB(
      KansoSpacing.md,
      KansoSpacing.sm,
      KansoSpacing.md,
      KansoSpacing.xs,
    ),
    child: Text(text, style: context.typography.sectionLabel),
  );

  /// The notes the open note does not link to yet.
  Widget _aboutNote(BuildContext context) {
    final l10n = context.l10n;
    return FutureBuilder<
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
            child: Center(child: KansoSpinner()),
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
              (error) => _message(KansoInlineMessage(message: error.message)),
              (rows) => rows.isEmpty
                  ? _message(Text(l10n.noteRelatedEmpty))
                  : Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: <Widget>[
                        if (widget.resurface != null)
                          _heading(context, l10n.resurfaceAboutNote),
                        KansoSheetGroup(
                          dividerIndent: KansoSettingsRow.iconInset,
                          children: <Widget>[
                            for (final row in rows)
                              KansoSettingsRow(
                                label: row.title,
                                value: row.label,
                                chevron: true,
                                onTap: () => _open(context, row.id),
                              ),
                          ],
                        ),
                      ],
                    ),
            );
          },
        );
      },
    );
  }

  Widget _message(Widget child) => Padding(
    padding: const EdgeInsets.all(KansoSpacing.md),
    child: child,
  );
}

// Kanso — rename_speakers_sheet.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/recording_speakers_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/undo_notice/undo_notice_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../core/views/widgets/sheet_header.dart';
import '../../../injection.dart';
import '../../models/repos/abs/recording_speakers_repository.dart';
import '../../utils/speaker_changes.dart';

/// Opens Rename Speakers for note [noteId]: the names before the paragraphs
/// of its recording's transcript, each in a field. Two voices given the same
/// name become one. Nothing is written until Rename, and the notice that
/// follows has Undo ([undoNotice]).
///
/// [repository] is the app's when null (tests pass one).
Future<void> showRenameSpeakersSheet(
  BuildContext context, {
  required String noteId,
  UndoNoticeBloc? undoNotice,
  RecordingSpeakersRepository? repository,
}) {
  final repo = repository ?? getIt<RecordingSpeakersRepository>();
  return showKansoBottomSheet<void>(
    context,
    builder: (_) => RenameSpeakersSheet(
      noteId: noteId,
      repository: repo,
      undoNotice: undoNotice,
    ),
  );
}

/// Where the sheet is.
enum SpeakersPhase { loading, nobody, editing, failed }

/// The sheet's content: see [showRenameSpeakersSheet].
class RenameSpeakersSheet extends StatefulWidget {
  const RenameSpeakersSheet({
    required this.noteId,
    required this.repository,
    this.undoNotice,
    this.initialNames,
    super.key,
  });

  final String noteId;
  final RecordingSpeakersRepository repository;
  final UndoNoticeBloc? undoNotice;

  /// For goldens: start with these names, without asking.
  final List<SpeakerNameView>? initialNames;

  @override
  State<RenameSpeakersSheet> createState() => _RenameSpeakersSheetState();
}

class _RenameSpeakersSheetState extends State<RenameSpeakersSheet> {
  SpeakersPhase _phase = SpeakersPhase.loading;
  List<SpeakerNameView> _names = const <SpeakerNameView>[];
  List<TextEditingController> _controllers = <TextEditingController>[];
  String? _failure;

  /// A rename is on its way: a second tap would ask again for names that
  /// are already changed.
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    final fixed = widget.initialNames;
    if (fixed != null) {
      _show(fixed);
      return;
    }
    unawaited(_load());
  }

  @override
  void dispose() {
    for (final controller in _controllers) {
      controller.dispose();
    }
    super.dispose();
  }

  void _show(List<SpeakerNameView> names) {
    _names = names;
    _controllers = <TextEditingController>[
      for (final name in names) TextEditingController(text: name.name),
    ];
    _phase = names.isEmpty ? SpeakersPhase.nobody : SpeakersPhase.editing;
  }

  Future<void> _load() async {
    final read = await widget.repository.names(widget.noteId);
    if (!mounted) {
      return;
    }
    setState(() {
      read.fold(
        (error) {
          _phase = SpeakersPhase.failed;
          _failure = error.message;
        },
        _show,
      );
    });
  }

  Future<void> _rename() async {
    if (_busy) {
      return;
    }
    final changes = speakerChanges(_names, <String>[
      for (final controller in _controllers) controller.text,
    ]);
    if (changes.isEmpty) {
      unawaited(Navigator.of(context).maybePop());
      return;
    }
    final l10n = context.l10n;
    final undo = widget.undoNotice;
    setState(() => _busy = true);
    final result = await widget.repository.rename(widget.noteId, changes);
    if (!mounted) {
      return;
    }
    setState(() => _busy = false);
    result.fold(
      (error) => setState(() {
        _phase = SpeakersPhase.failed;
        _failure = error.message;
      }),
      (renamed) {
        final ticket = renamed.undo;
        if (ticket != null && undo != null) {
          undo.add(
            UndoNoticeEvent.shown(ticket, l10n.recordingSpeakersRenamed),
          );
        }
        unawaited(Navigator.of(context).maybePop());
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return SafeArea(
      top: false,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          SheetHeader(
            title: l10n.recordingSpeakersTitle,
            actionLabel: l10n.recordingSpeakersClose,
          ),
          Flexible(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(KansoSpacing.md),
              child: _body(context),
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(
              KansoSpacing.md,
              0,
              KansoSpacing.md,
              KansoSpacing.md,
            ),
            child: _buttons(context),
          ),
        ],
      ),
    );
  }

  Widget _body(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    switch (_phase) {
      case SpeakersPhase.loading:
        return const SizedBox(height: KansoSpacing.lg);
      case SpeakersPhase.nobody:
        return Text(l10n.recordingSpeakersNobody, style: typography.body);
      case SpeakersPhase.failed:
        return Text(
          l10n.recordingSpeakersFailed(_failure ?? ''),
          style: typography.body.copyWith(color: colors.danger),
        );
      case SpeakersPhase.editing:
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(
              l10n.recordingSpeakersHint,
              style: typography.meta.copyWith(color: colors.textMuted),
            ),
            const SizedBox(height: KansoSpacing.sm),
            for (var i = 0; i < _names.length; i++)
              Padding(
                padding: const EdgeInsets.only(bottom: KansoSpacing.sm),
                child: KansoTextField(
                  controller: _controllers[i],
                  label: l10n.recordingSpeakersField(
                    _names[i].name,
                    _names[i].paragraphs,
                  ),
                  textInputAction: i == _names.length - 1
                      ? TextInputAction.done
                      : TextInputAction.next,
                  onSubmitted: i == _names.length - 1
                      ? (_) => unawaited(_rename())
                      : null,
                ),
              ),
          ],
        );
    }
  }

  Widget _buttons(BuildContext context) {
    final l10n = context.l10n;
    return switch (_phase) {
      SpeakersPhase.loading => const SizedBox.shrink(),
      SpeakersPhase.nobody || SpeakersPhase.failed => KansoButton(
        label: l10n.recordingSpeakersClose,
        variant: KansoButtonVariant.quiet,
        onPressed: () => Navigator.of(context).maybePop(),
      ),
      SpeakersPhase.editing => Row(
        children: <Widget>[
          Expanded(
            child: KansoButton(
              label: l10n.recordingSpeakersCancel,
              variant: KansoButtonVariant.quiet,
              onPressed: () => Navigator.of(context).maybePop(),
            ),
          ),
          const SizedBox(width: KansoSpacing.sm),
          Expanded(
            child: KansoButton(
              label: l10n.recordingSpeakersRename,
              onPressed: _busy ? null : _rename,
            ),
          ),
        ],
      ),
    };
  }
}

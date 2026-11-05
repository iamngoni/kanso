// Kanso — recording_summary_sheet.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/recording_summary_types.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/state/undo_notice/undo_notice_bloc.dart';
import '../../../core/utils/ai_trouble_text.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/sheet_header.dart';
import '../../../injection.dart';
import '../../models/repos/abs/recording_summary_repository.dart';

/// Opens Summarise Recording for note [noteId]: the sheet that says where the
/// transcript would go, then (only on Send) asks the person's own AI, then
/// shows the summary and each action item as a tick box. Nothing is written
/// until Apply, and the notice that follows has Undo ([undoNotice]).
///
/// [repository] is the app's when null (tests pass one).
Future<void> showRecordingSummarySheet(
  BuildContext context, {
  required String noteId,
  UndoNoticeBloc? undoNotice,
  RecordingSummaryRepository? repository,
}) {
  final repo = repository ?? getIt<RecordingSummaryRepository>();
  return showKansoBottomSheet<void>(
    context,
    builder: (_) => RecordingSummarySheet(
      noteId: noteId,
      repository: repo,
      undoNotice: undoNotice,
    ),
  );
}

/// Where the sheet is.
enum SummaryPhase { loading, unavailable, confirm, working, review, failed }

/// The sheet's content: see [showRecordingSummarySheet].
class RecordingSummarySheet extends StatefulWidget {
  const RecordingSummarySheet({
    required this.noteId,
    required this.repository,
    this.undoNotice,
    this.initialPhase,
    this.initialOffer,
    this.initialDraft,
    super.key,
  });

  final String noteId;
  final RecordingSummaryRepository repository;
  final UndoNoticeBloc? undoNotice;

  /// For goldens: start in a phase over fixed content, without asking.
  final SummaryPhase? initialPhase;
  final SummaryOffer? initialOffer;
  final SummaryDraftView? initialDraft;

  @override
  State<RecordingSummarySheet> createState() => _RecordingSummarySheetState();
}

class _RecordingSummarySheetState extends State<RecordingSummarySheet> {
  SummaryPhase _phase = SummaryPhase.loading;
  SummaryOffer? _offer;
  SummaryDraftView? _draft;
  Set<int> _ticked = <int>{};
  String? _failure;

  /// Why the last Apply wrote nothing, shown under the review.
  String? _applyFailure;
  bool _closed = false;

  @override
  void initState() {
    super.initState();
    final fixed = widget.initialPhase;
    if (fixed != null) {
      _phase = fixed;
      _offer = widget.initialOffer;
      _draft = widget.initialDraft;
      _ticked = <int>{
        for (var i = 0; i < (_draft?.actions.length ?? 0); i++) i,
      };
      return;
    }
    unawaited(_loadOffer());
  }

  @override
  void dispose() {
    _closed = true;
    if (_phase == SummaryPhase.working) {
      widget.repository.cancel();
    }
    super.dispose();
  }

  Future<void> _loadOffer() async {
    final read = await widget.repository.offer(widget.noteId);
    if (!mounted) {
      return;
    }
    setState(() {
      final offer = read.toOption().toNullable();
      _offer = offer;
      _phase = offer != null && offer.available
          ? SummaryPhase.confirm
          : SummaryPhase.unavailable;
    });
  }

  Future<void> _send() async {
    setState(() {
      _phase = SummaryPhase.working;
      _failure = null;
    });
    final result = await widget.repository.summarize(widget.noteId);
    if (!mounted || _closed || _phase != SummaryPhase.working) {
      return;
    }
    result.fold(
      (error) => setState(() {
        _phase = SummaryPhase.failed;
        _failure = error.message;
      }),
      (draft) => setState(() {
        _draft = draft;
        _ticked = <int>{for (var i = 0; i < draft.actions.length; i++) i};
        _phase = SummaryPhase.review;
      }),
    );
  }

  Future<void> _apply() async {
    final draft = _draft;
    if (draft == null) {
      return;
    }
    final l10n = context.l10n;
    final undo = widget.undoNotice;
    setState(() => _applyFailure = null);
    final result = await widget.repository.apply(
      widget.noteId,
      draft,
      (_ticked.toList()..sort()),
    );
    if (!mounted) {
      return;
    }
    result.fold(
      // The review stays, so what was looked over is not lost: the reason
      // shows under it, and Apply can be pressed again.
      (error) => setState(() => _applyFailure = error.message),
      (applied) {
        final ticket = applied.undo;
        if (ticket != null && undo != null) {
          undo.add(
            UndoNoticeEvent.shown(
              ticket,
              applied.replaced
                  ? l10n.recordingSummaryReplaced
                  : l10n.recordingSummaryAdded,
            ),
          );
        }
        Navigator.of(context).maybePop();
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
            title: l10n.recordingSummaryTitle,
            actionLabel: l10n.recordingSummaryClose,
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
    Widget quiet(String text) => Padding(
      padding: const EdgeInsets.only(top: KansoSpacing.xs),
      child: Text(
        text,
        style: typography.meta.copyWith(color: colors.textMuted),
      ),
    );
    switch (_phase) {
      case SummaryPhase.loading:
        return const SizedBox(height: KansoSpacing.lg);
      case SummaryPhase.unavailable:
        return Text(l10n.recordingSummaryUnavailable, style: typography.body);
      case SummaryPhase.confirm:
        final offer = _offer;
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(offer?.sendLine ?? '', style: typography.body),
            quiet(l10n.recordingSummaryNothingYet),
            if (offer?.replaces ?? false)
              quiet(l10n.recordingSummaryReplacesNote),
          ],
        );
      case SummaryPhase.working:
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(l10n.recordingSummaryWorking, style: typography.body),
            quiet(l10n.recordingSummaryCancelHint),
          ],
        );
      case SummaryPhase.failed:
        final reason = aiTroubleText(_failure ?? '', l10n);
        return Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(
              l10n.recordingSummaryFailed(reason),
              style: typography.body.copyWith(color: colors.danger),
            ),
            quiet(l10n.recordingSummaryUnchanged),
          ],
        );
      case SummaryPhase.review:
        return _review(context);
    }
  }

  Widget _heading(BuildContext context, String text) => Padding(
    padding: const EdgeInsets.only(
      top: KansoSpacing.sm,
      bottom: KansoSpacing.xs,
    ),
    child: Text(
      text,
      style: context.typography.sectionLabel.copyWith(
        color: context.colors.textSecondary,
      ),
    ),
  );

  Widget _review(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final typography = context.typography;
    final draft = _draft;
    if (draft == null) {
      return const SizedBox.shrink();
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        if (draft.overview.isNotEmpty) ...<Widget>[
          _heading(context, l10n.recordingSummarySectionSummary),
          for (final point in draft.overview)
            Padding(
              padding: const EdgeInsets.only(bottom: KansoSpacing.xs),
              child: Text('•  $point', style: typography.body),
            ),
        ],
        if (draft.decisions.isNotEmpty) ...<Widget>[
          _heading(context, l10n.recordingSummarySectionDecisions),
          for (final item in draft.decisions)
            Padding(
              padding: const EdgeInsets.only(bottom: KansoSpacing.xs),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Expanded(child: Text(item.text, style: typography.body)),
                  const SizedBox(width: KansoSpacing.sm),
                  Text(
                    _stamp(item.moment),
                    style: typography.meta.copyWith(color: colors.textMuted),
                  ),
                ],
              ),
            ),
        ],
        if (draft.actions.isNotEmpty) ...<Widget>[
          _heading(context, l10n.recordingSummarySectionActions),
          Text(
            l10n.recordingSummaryActionsHint,
            style: typography.meta.copyWith(color: colors.textMuted),
          ),
          const SizedBox(height: KansoSpacing.xs),
          for (var i = 0; i < draft.actions.length; i++)
            _actionRow(context, i, draft.actions[i]),
        ],
        if (draft.dropped > 0)
          Padding(
            padding: const EdgeInsets.only(top: KansoSpacing.sm),
            child: Text(
              l10n.recordingSummaryDropped(draft.dropped),
              style: typography.meta.copyWith(color: colors.textMuted),
            ),
          ),
        if (draft.cut)
          Padding(
            padding: const EdgeInsets.only(top: KansoSpacing.xs),
            child: Text(
              l10n.recordingSummaryCut,
              style: typography.meta.copyWith(color: colors.textMuted),
            ),
          ),
        if (_offer?.replaces ?? false)
          Padding(
            padding: const EdgeInsets.only(top: KansoSpacing.xs),
            child: Text(
              l10n.recordingSummaryReplacesNote,
              style: typography.meta.copyWith(color: colors.textMuted),
            ),
          ),
        if (_applyFailure case final reason?)
          Padding(
            padding: const EdgeInsets.only(top: KansoSpacing.sm),
            child: Text(
              l10n.recordingSummaryApplyFailed(reason),
              style: typography.body.copyWith(color: colors.danger),
            ),
          ),
      ],
    );
  }

  Widget _actionRow(BuildContext context, int index, SummaryItemView item) {
    final colors = context.colors;
    final typography = context.typography;
    final on = _ticked.contains(index);
    final meta = <String>[
      if (item.who case final who?) who,
      if (item.when case final due?) due,
      _stamp(item.moment),
    ].join(' · ');
    void toggle() => setState(() {
      if (on) {
        _ticked.remove(index);
      } else {
        _ticked.add(index);
      }
    });
    return KansoPressable(
      onTap: toggle,
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: KansoSpacing.xs),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            KansoCheckbox(value: on, onChanged: (_) => toggle()),
            const SizedBox(width: KansoSpacing.sm),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: <Widget>[
                  Text(item.text, style: typography.body),
                  Text(
                    meta,
                    style: typography.meta.copyWith(color: colors.textMuted),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buttons(BuildContext context) {
    final l10n = context.l10n;
    final draft = _draft;
    final nothingToWrite =
        _ticked.isEmpty &&
        draft != null &&
        draft.overview.isEmpty &&
        draft.decisions.isEmpty;
    return switch (_phase) {
      SummaryPhase.loading ||
      SummaryPhase.unavailable => const SizedBox.shrink(),
      SummaryPhase.confirm => Row(
        children: <Widget>[
          Expanded(
            child: KansoButton(
              label: l10n.recordingSummaryCancel,
              variant: KansoButtonVariant.quiet,
              onPressed: () => Navigator.of(context).maybePop(),
            ),
          ),
          const SizedBox(width: KansoSpacing.sm),
          Expanded(
            child: KansoButton(
              label: l10n.recordingSummarySend,
              onPressed: _send,
            ),
          ),
        ],
      ),
      SummaryPhase.working => KansoButton(
        label: l10n.recordingSummaryCancel,
        variant: KansoButtonVariant.quiet,
        onPressed: () {
          widget.repository.cancel();
          Navigator.of(context).maybePop();
        },
      ),
      SummaryPhase.review => Row(
        children: <Widget>[
          Expanded(
            child: KansoButton(
              label: l10n.recordingSummaryDiscard,
              variant: KansoButtonVariant.quiet,
              onPressed: () => Navigator.of(context).maybePop(),
            ),
          ),
          const SizedBox(width: KansoSpacing.sm),
          Expanded(
            child: KansoButton(
              label: l10n.recordingSummaryApply,
              onPressed: nothingToWrite ? null : _apply,
            ),
          ),
        ],
      ),
      SummaryPhase.failed => Row(
        children: <Widget>[
          Expanded(
            child: KansoButton(
              label: l10n.recordingSummaryClose,
              variant: KansoButtonVariant.quiet,
              onPressed: () => Navigator.of(context).maybePop(),
            ),
          ),
          const SizedBox(width: KansoSpacing.sm),
          Expanded(
            child: KansoButton(
              label: l10n.recordingSummaryTryAgain,
              onPressed: _send,
            ),
          ),
        ],
      ),
    };
  }
}

/// `00:22`, or `1:02:03` past an hour: the desktop's stamp.
String _stamp(int seconds) {
  final h = seconds ~/ 3600;
  final m = seconds ~/ 60 % 60;
  final s = seconds % 60;
  String two(int n) => n.toString().padLeft(2, '0');
  return h > 0 ? '$h:${two(m)}:${two(s)}' : '${two(m)}:${two(s)}';
}

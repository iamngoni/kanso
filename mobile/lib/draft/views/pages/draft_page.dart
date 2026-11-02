// Kanso — draft_page.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/ai.dart';
import '../../../core/bridge/api/draft.dart';
import '../../../core/bridge/api/draft_types.dart';
import '../../../core/bridge/api/library.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/utils/kanso_error_mapping.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_confirm_sheet.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../flashcards/utils/flashcards_text.dart';
import '../../../injection.dart';
import '../../../shell/views/widgets/shell_back_button.dart';

/// Draft from notes, at `/draft`: the notes it will read, what to write, the
/// streamed draft with its citations checked, then a review of the note it
/// would become. Nothing is saved until **Save as Note**; closing a draft
/// asks first.
class DraftPage extends StatefulWidget {
  const DraftPage({required this.noteIds, super.key});

  final List<String> noteIds;

  @override
  State<DraftPage> createState() => _DraftPageState();
}

enum _Stage { compose, draft, review }

class _DraftPageState extends State<DraftPage> {
  late final KansoDrafting _drafting = getIt<KansoLibrary>().drafting();
  final TextEditingController _request = TextEditingController();
  DraftSources? _sources;
  int _shape = 2;
  int _length = 1;
  _Stage _stage = _Stage.compose;
  KansoAiRun? _run;
  StreamSubscription<DraftEvent>? _events;
  DraftView? _view;
  List<DraftSourceRef> _refs = <DraftSourceRef>[];
  String _status = '';
  String? _error;
  bool _working = false;
  bool _finished = false;
  DraftPreview? _preview;
  final TextEditingController _title = TextEditingController();

  @override
  void initState() {
    super.initState();
    _drafting
        .sources(noteIds: widget.noteIds)
        .then((s) {
          if (mounted) {
            setState(() => _sources = s);
          }
        })
        .catchError((Object e) {
          if (mounted) {
            setState(() => _error = applicationErrorOf(e).message);
          }
        });
  }

  void _write() {
    _run?.stop();
    final run = getIt<KansoAi>().newRun();
    _run = run;
    setState(() {
      _stage = _Stage.draft;
      _working = true;
      _finished = false;
      _error = null;
      _view = null;
      _status = 'Writing…';
    });
    _events = _drafting
        .write(
          noteIds: widget.noteIds,
          shape: FlashcardsText.shapeAt(_shape),
          request: _request.text,
          length: FlashcardsText.lengthAt(_length),
          run: run,
        )
        .listen((e) {
          if (!mounted) {
            return;
          }
          switch (e) {
            case DraftEvent_Status(:final line):
              setState(() => _status = line);
            case DraftEvent_Text(:final draft):
              setState(() => _view = draft);
            case DraftEvent_Done(:final draft, :final sources):
              setState(() {
                _view = draft;
                _refs = sources;
                _working = false;
                _finished = true;
              });
            case DraftEvent_CutOff(:final message):
              setState(() {
                _error = message;
                _working = false;
              });
            case DraftEvent_Failed(:final message):
              setState(() {
                _error = message;
                _working = false;
              });
            case DraftEvent_Stopped():
              setState(() => _working = false);
          }
        });
  }

  Future<void> _toReview() async {
    final view = _view;
    if (view == null) {
      return;
    }
    try {
      final preview = await _drafting.preview(
        text: view.text,
        sources: _refs,
      );
      if (mounted) {
        setState(() {
          _preview = preview;
          _title.text = preview.title;
          _stage = _Stage.review;
          _error = null;
        });
      }
    } on Object catch (e) {
      if (mounted) {
        setState(() => _error = applicationErrorOf(e).message);
      }
    }
  }

  Future<void> _save() async {
    final preview = _preview;
    if (preview == null) {
      return;
    }
    try {
      final id = await _drafting.save(
        title: _title.text,
        markdown: preview.markdown,
        bookId: preview.defaultBookId,
        sourceCount: _refs.length,
      );
      if (!mounted) {
        return;
      }
      _finished = false;
      GoRouter.of(context)
        ..pop()
        ..unawaitedPush('${KansoRoutes.noteById}/${Uri.encodeComponent(id)}');
    } on Object catch (e) {
      if (mounted) {
        setState(() => _error = applicationErrorOf(e).message);
      }
    }
  }

  Future<void> _close() async {
    if (_view == null || !_finished) {
      GoRouter.of(context).pop();
      return;
    }
    final yes = await KansoConfirmSheet.show(
      context,
      title: FlashcardsText.draftDiscardTitle,
      message: FlashcardsText.draftDiscardBody,
      confirmLabel: FlashcardsText.draftDiscard,
    );
    if (yes && mounted) {
      GoRouter.of(context).pop();
    }
  }

  @override
  void dispose() {
    _run?.stop();
    unawaited(_events?.cancel());
    _request.dispose();
    _title.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colors = context.colors;
    return PopScope(
      canPop: !(_finished && _view != null),
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop) {
          _close();
        }
      },
      child: KansoPage(
        backgroundColor: colors.app,
        body: ColoredBox(
          color: colors.app,
          child: Column(
            children: <Widget>[
              KansoTopBar(
                backgroundColor: colors.app,
                bordered: false,
                leading: const ShellBackButton(),
                title: _stage == _Stage.review
                    ? FlashcardsText.draftReviewTitle
                    : FlashcardsText.draftTitle,
              ),
              Expanded(child: _content(context)),
            ],
          ),
        ),
      ),
    );
  }

  Widget _content(BuildContext context) {
    final colors = context.colors;
    final type = context.typography;
    final sources = _sources;
    if (_stage == _Stage.review) {
      final preview = _preview!;
      return ListView(
        padding: const EdgeInsets.all(KansoSpacing.md),
        children: <Widget>[
          TextField(
            controller: _title,
            style: type.heading1,
            decoration: const InputDecoration(border: InputBorder.none),
          ),
          const SizedBox(height: KansoSpacing.sm),
          Text(preview.markdown, style: type.body),
          const SizedBox(height: KansoSpacing.lg),
          if (_error case final message?)
            Text(message, style: type.meta.copyWith(color: colors.danger)),
          KansoButton(label: FlashcardsText.draftSave, onPressed: _save),
          const SizedBox(height: KansoSpacing.xs),
          KansoButton(
            label: 'Back to the draft',
            variant: KansoButtonVariant.quiet,
            onPressed: () => setState(() => _stage = _Stage.draft),
          ),
        ],
      );
    }
    return ListView(
      padding: const EdgeInsets.all(KansoSpacing.md),
      children: <Widget>[
        Text(FlashcardsText.draftSources, style: type.sectionLabel),
        const SizedBox(height: KansoSpacing.xxs),
        if (sources == null)
          const LinearProgressIndicator()
        else ...<Widget>[
          for (final note in sources.notes.take(8))
            Text(
              note.title.isEmpty ? 'Untitled' : note.title,
              style: type.body,
            ),
          if (sources.notes.length > 8)
            Text(
              '+ ${sources.notes.length - 8} more',
              style: type.meta.copyWith(color: colors.textSecondary),
            ),
          const SizedBox(height: KansoSpacing.xxs),
          Text(
            sources.summary,
            style: type.meta.copyWith(color: colors.textSecondary),
          ),
          if (sources.tooMany)
            Text(
              "That's more than a draft can read. Pick up to 200 notes.",
              style: type.meta.copyWith(color: colors.danger),
            )
          else if (sources.notes.isEmpty)
            Text(
              FlashcardsText.draftNone,
              style: type.meta.copyWith(color: colors.danger),
            )
          else
            Text(
              FlashcardsText.draftSent(sources.notes.length),
              style: type.meta.copyWith(color: colors.textMuted),
            ),
        ],
        const SizedBox(height: KansoSpacing.md),
        Wrap(
          spacing: KansoSpacing.xs,
          children: <Widget>[
            for (var i = 0; i < FlashcardsText.shapes.length; i++)
              ChoiceChip(
                label: Text(FlashcardsText.shapes[i]),
                selected: _shape == i,
                onSelected: (_) => setState(() => _shape = i),
              ),
          ],
        ),
        const SizedBox(height: KansoSpacing.xs),
        Wrap(
          spacing: KansoSpacing.xs,
          children: <Widget>[
            for (var i = 0; i < FlashcardsText.lengths.length; i++)
              ChoiceChip(
                label: Text(FlashcardsText.lengths[i]),
                selected: _length == i,
                onSelected: (_) => setState(() => _length = i),
              ),
          ],
        ),
        const SizedBox(height: KansoSpacing.sm),
        TextField(
          controller: _request,
          minLines: 2,
          maxLines: 6,
          onChanged: (_) => setState(() {}),
          decoration: const InputDecoration(hintText: FlashcardsText.draftWhat),
        ),
        const SizedBox(height: KansoSpacing.sm),
        if (_working)
          KansoButton(
            label: FlashcardsText.draftStop,
            variant: KansoButtonVariant.secondary,
            onPressed: () => _run?.stop(),
          )
        else
          KansoButton(
            label: _view == null
                ? FlashcardsText.draftWrite
                : FlashcardsText.draftTryAgain,
            onPressed:
                sources == null || sources.notes.isEmpty || sources.tooMany
                ? null
                : _write,
          ),
        if (_stage == _Stage.draft) ...<Widget>[
          const SizedBox(height: KansoSpacing.md),
          if (_working)
            Text(
              _status,
              style: type.meta.copyWith(color: colors.textSecondary),
            ),
          if (_view case final view?) ...<Widget>[
            Text.rich(_spans(context, view), style: type.body),
            if (_finished && !view.notCovered) ...<Widget>[
              const SizedBox(height: KansoSpacing.md),
              KansoButton(
                label: FlashcardsText.draftSave,
                onPressed: _toReview,
              ),
              const SizedBox(height: KansoSpacing.xs),
              KansoButton(
                label: FlashcardsText.draftCopy,
                variant: KansoButtonVariant.secondary,
                onPressed: () =>
                    Clipboard.setData(ClipboardData(text: view.text)),
              ),
            ],
          ],
          if (_error case final message?)
            Padding(
              padding: const EdgeInsets.only(top: KansoSpacing.sm),
              child: Text(
                message,
                style: type.meta.copyWith(color: colors.danger),
              ),
            ),
        ],
      ],
    );
  }

  /// The draft with a muted underline under what has no source, and the
  /// cited stretches tappable to name their sources.
  InlineSpan _spans(BuildContext context, DraftView view) {
    final colors = context.colors;
    final text = view.text;
    final units = text.codeUnits;
    final children = <InlineSpan>[];
    var at = 0;
    for (final span in view.spans) {
      final start = span.start.clamp(at, units.length);
      final end = span.end.clamp(start, units.length);
      if (start > at) {
        children.add(TextSpan(text: String.fromCharCodes(units, at, start)));
      }
      final piece = String.fromCharCodes(units, start, end);
      if (span.unsourced) {
        children.add(
          TextSpan(
            text: piece,
            style: TextStyle(
              decoration: TextDecoration.underline,
              decorationStyle: TextDecorationStyle.dotted,
              decorationColor: colors.warning,
            ),
          ),
        );
      } else {
        children.add(
          TextSpan(text: piece),
        );
      }
      at = end;
    }
    if (at < units.length) {
      children.add(TextSpan(text: String.fromCharCodes(units, at)));
    }
    if (view.removed > 0 || view.spans.any((s) => s.unsourced)) {
      children.add(
        TextSpan(
          text: '\n\n${FlashcardsText.draftNoSource} (underlined)',
          style: TextStyle(color: colors.textMuted, fontSize: 12),
        ),
      );
    }
    return TextSpan(children: children);
  }
}

/// `push` whose result nobody waits for.
extension on GoRouter {
  void unawaitedPush(String location) {
    unawaited(push<void>(location));
  }
}

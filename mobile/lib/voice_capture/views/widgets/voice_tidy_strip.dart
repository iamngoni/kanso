// Kanso — voice_tidy_strip.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';

import '../../../core/bridge/api/voice_capture_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../injection.dart';
import '../../models/repos/abs/voice_capture_repository.dart';

/// A quiet strip at the top of a voice note that can be tidied: **Tidy**
/// (a title, short paragraphs and tasks, from the person's own AI; the
/// transcript stays), **Tidying…** while the model works, and why a try
/// failed with **Try again**. Nothing is shown for any other note, so the
/// note's layout does not move.
class VoiceTidyStrip extends StatefulWidget {
  const VoiceTidyStrip({
    required this.noteId,
    this.revision,
    this.repository,
    super.key,
  });

  /// The open note.
  final String noteId;

  /// The note's revision: the strip reads the note again when it changes.
  final String? revision;

  /// The voice capture repository; the app's when null (tests pass one).
  final VoiceCaptureRepository? repository;

  @override
  State<VoiceTidyStrip> createState() => _VoiceTidyStripState();
}

class _VoiceTidyStripState extends State<VoiceTidyStrip> {
  /// Null where the app has no voice capture (a page built without the
  /// library, as some tests do): the strip shows nothing.
  late final VoiceCaptureRepository? _repository =
      widget.repository ??
      (getIt.isRegistered<VoiceCaptureRepository>()
          ? getIt<VoiceCaptureRepository>()
          : null);
  VoiceNoteInfo? _info;
  bool _working = false;
  String? _failure;
  int _generation = 0;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  @override
  void didUpdateWidget(VoiceTidyStrip old) {
    super.didUpdateWidget(old);
    if (old.noteId != widget.noteId) {
      _info = null;
      _working = false;
      _failure = null;
      unawaited(_load());
    } else if (old.revision != widget.revision && !_working) {
      unawaited(_load());
    }
  }

  Future<void> _load() async {
    final repository = _repository;
    if (repository == null) {
      return;
    }
    final generation = ++_generation;
    final read = await repository.noteInfo(widget.noteId);
    if (!mounted || generation != _generation) {
      return;
    }
    setState(() => _info = read.toOption().toNullable());
  }

  Future<void> _tidy() async {
    final repository = _repository;
    if (repository == null) {
      return;
    }
    final noteId = widget.noteId;
    setState(() {
      _working = true;
      _failure = null;
    });
    final done = await repository.tidy(noteId);
    if (!mounted || noteId != widget.noteId) {
      return;
    }
    done.fold(
      (error) => setState(() {
        _working = false;
        _failure = error.message;
      }),
      (outcome) {
        setState(() => _working = false);
        unawaited(_load());
      },
    );
  }

  @override
  Widget build(BuildContext context) {
    final info = _info;
    final offered = info != null && info.canTidy;
    if (!offered && !_working && _failure == null) {
      return const SizedBox.shrink();
    }
    final colors = context.colors;
    final l10n = context.l10n;
    final failure = _failure;
    final words = _working
        ? l10n.voiceTidyWorking
        : failure != null
        ? l10n.voiceTidyFailed(failure)
        : l10n.voiceTidyOffer;
    return Container(
      margin: const EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.xs,
        KansoSpacing.md,
        0,
      ),
      padding: const EdgeInsets.symmetric(
        horizontal: KansoSpacing.sm,
        vertical: KansoSpacing.xs,
      ),
      decoration: BoxDecoration(
        color: colors.fill,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: <Widget>[
          KansoIcon(
            KansoIcons.sparkles,
            color: failure != null ? colors.danger : colors.textSecondary,
          ),
          const SizedBox(width: KansoSpacing.xs),
          Expanded(
            child: Text(
              words,
              style: context.typography.meta.copyWith(
                color: failure != null ? colors.danger : colors.textSecondary,
              ),
            ),
          ),
          if (!_working)
            KansoButton(
              label: failure != null ? l10n.voiceTidyTryAgain : l10n.voiceTidy,
              variant: KansoButtonVariant.quiet,
              compact: true,
              expanded: false,
              onPressed: _tidy,
            ),
        ],
      ),
    );
  }
}

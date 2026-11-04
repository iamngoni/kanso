// Kanso — recording_page.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../injection.dart';
import '../../state/recording/recording_bloc.dart';
import '../../utils/recording_clock.dart';

/// Record a note, at `/record`: the microphone into a new note, or into the
/// note in `?note=<id>`. The strip over the app has Pause, Stop and Discard
/// while it runs; this page starts it and says how it went.
class RecordingPage extends StatelessWidget {
  const RecordingPage({this.noteId, this.bloc, super.key});

  /// The note to record into; a new note when null.
  final String? noteId;

  /// The recording bloc; the app's when null (tests pass one).
  final RecordingBloc? bloc;

  @override
  Widget build(BuildContext context) {
    final source = bloc ?? getIt<RecordingBloc>();
    return KansoPage(
      backgroundColor: context.colors.editor,
      body: BlocConsumer<RecordingBloc, RecordingState>(
        bloc: source,
        listenWhen: (a, b) => a.saved != b.saved && b.saved != null,
        listener: (context, state) {
          final saved = state.saved!;
          source.add(const RecordingEvent.noticeCleared());
          // The note the recording is in opens, in All Notes.
          context.go('/notes/note/${Uri.encodeComponent(saved.noteId)}');
        },
        builder: (context, state) =>
            _Body(bloc: source, state: state, noteId: noteId),
      ),
    );
  }
}

class _Body extends StatelessWidget {
  const _Body({required this.bloc, required this.state, required this.noteId});

  final RecordingBloc bloc;
  final RecordingState state;
  final String? noteId;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    final recording = state.phase == RecordingPhase.recording;
    final busy = state.phase != RecordingPhase.idle;
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.all(KansoSpacing.md),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Row(
              children: <Widget>[
                KansoIconButton(
                  icon: KansoIcons.chevronLeft,
                  semanticLabel: l10n.recordingBack,
                  onPressed: () => context.pop(),
                ),
                const SizedBox(width: KansoSpacing.xs),
                Text(l10n.recordingTitle, style: context.typography.barTitle),
              ],
            ),
            const Spacer(),
            Center(
              child: Text(
                recordingClock(state.seconds),
                style: context.typography.largeTitle,
              ),
            ),
            const SizedBox(height: KansoSpacing.xs),
            Center(
              child: Text(
                noteId == null
                    ? l10n.recordingIntoNewNote
                    : l10n.recordingIntoThisNote,
                style: context.typography.secondary,
              ),
            ),
            if (state.problem case final problem?) ...<Widget>[
              const SizedBox(height: KansoSpacing.md),
              Center(
                child: Text(
                  _problemText(context, problem),
                  textAlign: TextAlign.center,
                  style: context.typography.secondary.copyWith(
                    color: colors.danger,
                  ),
                ),
              ),
            ],
            const Spacer(),
            Text(
              l10n.recordingReminder,
              textAlign: TextAlign.center,
              style: context.typography.meta,
            ),
            const SizedBox(height: KansoSpacing.md),
            KansoButton(
              label: recording ? l10n.recordingStop : l10n.recordingStart,
              icon: recording ? KansoIcons.square : KansoIcons.mic,
              onPressed: busy && !recording
                  ? null
                  : () => bloc.add(
                      recording
                          ? const RecordingEvent.stopped()
                          : RecordingEvent.started(noteId: noteId),
                    ),
            ),
          ],
        ),
      ),
    );
  }

  String _problemText(BuildContext context, RecordingProblem problem) {
    final l10n = context.l10n;
    return switch (problem) {
      RecordingProblem.busy => l10n.recordingProblemBusy,
      RecordingProblem.cannotStart => l10n.recordingProblemCannotStart,
      RecordingProblem.locked => l10n.recordingProblemLocked,
      RecordingProblem.noteGone => l10n.recordingProblemNoteGone,
      RecordingProblem.saveFailed => l10n.recordingProblemSaveFailed,
    };
  }
}

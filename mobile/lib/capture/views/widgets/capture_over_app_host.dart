// Kanso — capture_over_app_host.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/router/deep_links.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_feedback.dart';
import '../../models/data/capture_first_row.dart';
import '../../models/data/capture_over_app_status.dart';
import '../../models/repos/abs/capture_window_repository.dart';
import '../../state/capture_over_app/capture_over_app_cubit.dart';
import '../../state/capture_task/capture_task_cubit.dart';
import 'capture_sheet.dart';

/// The capture sheet over another app, and nothing under it: the window is
/// see-through, so the app the person was in shows around the sheet.
///
/// Once the window is on screen it asks which mode its link wants and opens
/// the sheet the app has, in that mode: Task mode writes a task through the
/// same library call and parser, Note mode offers a new note (opened in
/// Kanso), a photo, photos or a PDF (saved to the Inbox here). A task
/// written shows where it went for a moment, then the window goes; so does
/// a dismissed sheet. A locked destination is refused as in the app, with
/// the same words, and the sheet stays.
class CaptureOverAppHost extends StatefulWidget {
  const CaptureOverAppHost({
    required this.window,
    required this.tasks,
    required this.media,
    super.key,
  });

  /// How long "Added to …" shows before the window goes.
  static const Duration savedPause = Duration(milliseconds: 900);

  /// How long the line about what was picked shows before the window goes.
  static const Duration noticePause = Duration(milliseconds: 1800);

  /// How long a failure shows before the window goes.
  static const Duration failurePause = Duration(milliseconds: 3600);

  final CaptureWindowRepository window;

  /// Makes the Task mode's cubit, which the sheet closes.
  final CaptureTaskCubit Function() tasks;

  /// Picks and saves for Note mode; the caller closes it.
  final CaptureOverAppCubit media;

  @override
  State<CaptureOverAppHost> createState() => _CaptureOverAppHostState();
}

class _CaptureOverAppHostState extends State<CaptureOverAppHost> {
  AppLifecycleListener? _lifecycle;
  bool _opened = false;
  bool _leaving = false;

  @override
  void initState() {
    super.initState();
    // An engine started ahead runs before its window is shown: the sheet
    // waits for the window, so that it is seen opening.
    if (WidgetsBinding.instance.lifecycleState == AppLifecycleState.resumed) {
      _openSoon();
    } else {
      _lifecycle = AppLifecycleListener(onResume: _openSoon);
    }
  }

  @override
  void dispose() {
    _lifecycle?.dispose();
    super.dispose();
  }

  void _openSoon() {
    if (_opened) {
      return;
    }
    _opened = true;
    WidgetsBinding.instance
      ..addPostFrameCallback((_) => _open())
      ..scheduleFrame();
  }

  Future<void> _open() async {
    final mode = await widget.window.mode();
    if (!mounted) {
      return;
    }
    await CaptureSheet.show(
      context,
      first: CaptureFirstRow(
        icon: KansoIcons.plus,
        label: context.l10n.noteListNewNote,
        onTap: _newNote,
      ),
      onSource: widget.media.sourceChosen,
      tasks: widget.tasks(),
      mode: mode,
      onTaskSaved: _taskSaved,
    );
    // Dismissed, or closed by a choice: with nothing being picked, go.
    if (widget.media.state == CaptureOverAppStatus.idle) {
      _finish();
    }
  }

  /// A new note is written in Kanso itself.
  void _newNote() {
    if (_leaving) {
      return;
    }
    _leaving = true;
    widget.window.openInKanso(
      '$kansoScheme://$kansoOpenHost${KansoRoutes.newNote}',
    );
  }

  void _taskSaved() {
    Future<void>.delayed(CaptureOverAppHost.savedPause, () {
      if (mounted) {
        Navigator.of(context, rootNavigator: true).maybePop();
      }
    });
  }

  void _picked(BuildContext context, CaptureOverAppStatus status) {
    final l10n = context.l10n;
    final (message, tone, pause) = switch (status) {
      CaptureOverAppStatus.saved => (
        l10n.captureOverAppSaved,
        KansoFeedbackTone.success,
        CaptureOverAppHost.noticePause,
      ),
      CaptureOverAppStatus.locked => (
        l10n.captureOverAppLocked,
        KansoFeedbackTone.error,
        CaptureOverAppHost.failurePause,
      ),
      CaptureOverAppStatus.failed => (
        l10n.captureOverAppFailed,
        KansoFeedbackTone.error,
        CaptureOverAppHost.failurePause,
      ),
      _ => (null, KansoFeedbackTone.info, Duration.zero),
    };
    if (message != null) {
      KansoFeedback.show(context, message, tone: tone);
    }
    Future<void>.delayed(pause, _finish);
  }

  void _finish() {
    if (_leaving) {
      return;
    }
    _leaving = true;
    widget.window.finish();
  }

  @override
  Widget build(BuildContext context) {
    return BlocListener<CaptureOverAppCubit, CaptureOverAppStatus>(
      bloc: widget.media,
      listenWhen: (before, after) => after.finished,
      listener: _picked,
      child: const SizedBox.expand(),
    );
  }
}

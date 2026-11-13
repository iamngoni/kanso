// Kanso — capture_sheet.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_segment.dart';
import '../../../core/views/widgets/kanso_segmented.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_sheet_group.dart';
import '../../models/data/capture_first_row.dart';
import '../../models/data/capture_mode.dart';
import '../../models/data/media_source.dart';
import '../../state/capture_task/capture_task_cubit.dart';
import 'capture_task_form.dart';

/// What can be added from the phone: [first] (a new note, from the list's
/// button; an image link, from the editor's), then a photo taken now,
/// photos from the library, or a PDF. A choice closes the sheet, then says
/// which it was.
///
/// Given a [CaptureTaskCubit] the sheet opens with a Note | Task switch, as
/// the desktop's Quick Capture has, remembering the last choice. In Task
/// mode it is a field for one task, with the date and repeat it reads under
/// it, and the task is written to today's daily note or the Inbox.
class CaptureSheet extends StatelessWidget {
  const CaptureSheet({
    required this.first,
    required this.onSource,
    this.tasks,
    super.key,
  });

  /// Shows the sheet over [context]. With [tasks] it offers Task mode too;
  /// the sheet closes the cubit when it is dismissed, which was made for it.
  /// With [mode] it opens in that mode rather than the last one chosen.
  static Future<void> show(
    BuildContext context, {
    required CaptureFirstRow first,
    required ValueChanged<MediaSource> onSource,
    CaptureTaskCubit? tasks,
    CaptureMode? mode,
  }) {
    if (tasks != null && !tasks.state.loaded) {
      tasks.started(mode: mode);
    }
    return showKansoBottomSheet<void>(
      context,
      builder: (_) =>
          CaptureSheet(first: first, onSource: onSource, tasks: tasks),
    ).whenComplete(() => tasks?.close());
  }

  /// The row above the camera's, with what it does.
  final CaptureFirstRow first;

  final ValueChanged<MediaSource> onSource;

  /// The Task mode's state, or none for a sheet that only picks media.
  final CaptureTaskCubit? tasks;

  @override
  Widget build(BuildContext context) {
    final cubit = tasks;
    if (cubit == null) {
      return _Sources(first: first, onSource: onSource);
    }
    return BlocProvider<CaptureTaskCubit>.value(
      value: cubit,
      child: BlocBuilder<CaptureTaskCubit, CaptureTaskState>(
        buildWhen: (before, after) => before.mode != after.mode,
        builder: (context, state) {
          final l10n = context.l10n;
          return SafeArea(
            top: false,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                Padding(
                  padding: const EdgeInsets.fromLTRB(
                    KansoSpacing.md,
                    KansoSpacing.xs,
                    KansoSpacing.md,
                    KansoSpacing.sm,
                  ),
                  child: KansoSegmented<CaptureMode>(
                    expanded: true,
                    semanticLabel: l10n.captureModeSemantics,
                    selected: state.mode,
                    onChanged: cubit.modeChosen,
                    segments: <KansoSegment<CaptureMode>>[
                      KansoSegment<CaptureMode>(
                        value: CaptureMode.note,
                        label: l10n.captureModeNote,
                        icon: KansoIcons.fileText,
                      ),
                      KansoSegment<CaptureMode>(
                        value: CaptureMode.task,
                        label: l10n.captureModeTask,
                        icon: KansoIcons.listChecks,
                      ),
                    ],
                  ),
                ),
                if (state.mode == CaptureMode.task)
                  const CaptureTaskForm()
                else
                  _Sources(first: first, onSource: onSource),
              ],
            ),
          );
        },
      ),
    );
  }
}

/// The rows of the sheet that makes a note: [CaptureSheet.first], the camera,
/// photos and a PDF.
class _Sources extends StatelessWidget {
  const _Sources({required this.first, required this.onSource});

  final CaptureFirstRow first;
  final ValueChanged<MediaSource> onSource;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    void choose(VoidCallback then) {
      Navigator.of(context).pop();
      then();
    }

    return SafeArea(
      top: false,
      child: Padding(
        padding: const EdgeInsets.only(bottom: KansoSpacing.md),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            KansoSheetGroup(
              children: <Widget>[
                KansoSettingsRow(
                  icon: first.icon,
                  label: first.label,
                  onTap: () => choose(first.onTap),
                ),
              ],
            ),
            KansoSheetGroup(
              children: <Widget>[
                KansoSettingsRow(
                  icon: KansoIcons.camera,
                  label: l10n.captureTakePhoto,
                  onTap: () => choose(() => onSource(MediaSource.camera)),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.image,
                  label: l10n.captureChoosePhotos,
                  onTap: () => choose(() => onSource(MediaSource.photos)),
                ),
                KansoSettingsRow(
                  icon: KansoIcons.fileText,
                  label: l10n.captureAttachPdf,
                  onTap: () => choose(() => onSource(MediaSource.pdf)),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

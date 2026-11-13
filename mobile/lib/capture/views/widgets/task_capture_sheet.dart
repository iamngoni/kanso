// Kanso — task_capture_sheet.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../../shell/state/note_creation/note_creation_bloc.dart';
import '../../models/data/capture_first_row.dart';
import '../../models/data/capture_mode.dart';
import '../../state/capture_task/capture_task_cubit.dart';
import 'capture_sheet.dart';

/// The capture sheet in Task mode, as `kanso://open/new?task=1` opens it:
/// the Add task launcher shortcut and Quick Settings tile on Android. It is
/// the sheet the list's + button opens when held, so switching to Note
/// still offers a new note, the camera, photos and a PDF, in All Notes.
Future<void> showTaskCaptureSheet(BuildContext context) {
  final creation = context.read<NoteCreationBloc>();
  return CaptureSheet.show(
    context,
    first: CaptureFirstRow(
      icon: KansoIcons.plus,
      label: context.l10n.noteListNewNote,
      onTap: () =>
          creation.add(const NoteCreationEvent.requested(ListPlace.all())),
    ),
    onSource: (source) => creation.add(NoteCreationEvent.captured(source)),
    tasks: getIt<CaptureTaskCubit>(),
    mode: CaptureMode.task,
  );
}

// Kanso — journal_flows.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../templates/views/widgets/template_picker_sheet.dart';
import '../models/data/new_choice.dart';
import '../state/journal/journal_bloc.dart';
import '../views/widgets/calendar_sheet.dart';
import '../views/widgets/new_menu_sheet.dart';

/// Opens the calendar and, if a day is chosen, opens that day's note. It
/// needs the [JournalBloc] of a `JournalOpener` above [context]. [openNote]
/// is the daily note open, whose month the calendar starts on.
Future<void> showCalendar(BuildContext context, {String? openNote}) async {
  final bloc = context.read<JournalBloc>();
  final day = await CalendarSheet.show(context, openNote: openNote);
  if (day != null) {
    bloc.add(JournalEvent.dayRequested(day));
  }
}

/// Shows the templates and, if one is chosen, makes a note from it in
/// [notebookId] (the notebook the list shows; the Inbox when null).
Future<void> showTemplatePicker(
  BuildContext context, {
  String? notebookId,
}) async {
  final bloc = context.read<JournalBloc>();
  final id = await TemplatePickerSheet.show(context);
  if (id != null) {
    bloc.add(
      JournalEvent.fromTemplateRequested(
        templateId: id,
        notebookId: notebookId,
      ),
    );
  }
}

/// Shows the new menu and does what is chosen in it. [notebookId] is the
/// notebook the list shows.
Future<void> showNewMenu(BuildContext context, {String? notebookId}) async {
  final bloc = context.read<JournalBloc>();
  final choice = await NewMenuSheet.show(context);
  if (choice == null || !context.mounted) {
    return;
  }
  switch (choice) {
    case NewChoice.today:
      bloc.add(const JournalEvent.todayRequested());
    case NewChoice.calendar:
      await showCalendar(context);
    case NewChoice.fromTemplate:
      await showTemplatePicker(context, notebookId: notebookId);
    case NewChoice.newTemplate:
      bloc.add(const JournalEvent.newTemplateRequested());
  }
}

// Kanso — kanso_intent.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

/// What a keyboard shortcut of the browsing screens asks for.
sealed class KansoIntent extends Intent {
  const KansoIntent();
}

/// Back to the note opened before this one.
class BackIntent extends KansoIntent {
  const BackIntent();
}

/// Forward to the note opened after this one.
class ForwardIntent extends KansoIntent {
  const ForwardIntent();
}

/// Search all notes.
class SearchAllIntent extends KansoIntent {
  const SearchAllIntent();
}

/// Open settings.
class SettingsIntent extends KansoIntent {
  const SettingsIntent();
}

/// Show or hide the library: the pane beside the list, or the drawer.
class ToggleLibraryIntent extends KansoIntent {
  const ToggleLibraryIntent();
}

/// Open the next note of the list.
class NextNoteIntent extends KansoIntent {
  const NextNoteIntent();
}

/// Open the previous note of the list.
class PreviousNoteIntent extends KansoIntent {
  const PreviousNoteIntent();
}

/// Pin the open note, or unpin it.
class PinNoteIntent extends KansoIntent {
  const PinNoteIntent();
}

/// Make a new note, in the place being listed.
class NewNoteIntent extends KansoIntent {
  const NewNoteIntent();
}

/// Make a new notebook: asks for its name.
class NewNotebookIntent extends KansoIntent {
  const NewNotebookIntent();
}

/// Show the open note as text to edit.
class EditNoteIntent extends KansoIntent {
  const EditNoteIntent();
}

/// Show the open note to read.
class PreviewNoteIntent extends KansoIntent {
  const PreviewNoteIntent();
}

/// Open or make today's daily note.
class OpenTodayIntent extends KansoIntent {
  const OpenTodayIntent();
}

/// Open the library-wide replace screen.
class ReplaceInNotesIntent extends KansoIntent {
  const ReplaceInNotesIntent();
}

/// Show the open note's information sheet.
class ToggleInspectorIntent extends KansoIntent {
  const ToggleInspectorIntent();
}

/// Leave the settings page.
class CloseSettingsIntent extends KansoIntent {
  const CloseSettingsIntent();
}

/// Move to the next or previous settings page.
class SettingsPageStepIntent extends KansoIntent {
  const SettingsPageStepIntent({required this.forward});

  final bool forward;
}

/// Open the command palette.
class OpenPaletteIntent extends KansoIntent {
  const OpenPaletteIntent();
}

/// Show the open note by itself, or bring the rest of the app back.
class ToggleFocusModeIntent extends KansoIntent {
  const ToggleFocusModeIntent();
}

/// Bring the rest of the app back from focus mode.
class ExitFocusModeIntent extends KansoIntent {
  const ExitFocusModeIntent();
}

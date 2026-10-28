// Kanso — shortcut_intents.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/widgets.dart';

import '../../core/bridge/api/types.dart' show FormatKind, ShortcutEntry;
import '../../editor/models/data/editor_intent.dart';
import '../../tabs/models/data/tab_intent.dart';
import '../../window/models/data/window_intent.dart';
import '../models/data/kanso_intent.dart';

/// Mobile catalog entries deliberately left without an action, with the
/// reason each desktop key has no corresponding mobile surface or handler.
const Set<String> unboundOnMobile = <String>{
  'quit', // The operating system owns quitting the app.
  'save_capture', // Quick Capture is a separate desktop window.
  'cancel_capture', // Quick Capture is a separate desktop window.
  'dictate', // Voice input is not reachable from the shell.
  'show_split', // A phone has no split view.
  'toggle_focus_mode', // Focus mode is not on the mobile app.
  'exit_focus_mode', // Focus mode is not on the mobile app.
  'zoom_in', // The mobile app has no page zoom.
  'zoom_out', // The mobile app has no page zoom.
  'actual_size', // The mobile app has no page zoom.
  'ask_your_notes', // The Ask screen is not on master.
  'open_note_picker', // There is no mobile note picker or quick-open screen.
  'focus_search', // The shell cannot focus the search field from this context.
  'focus_note_list', // The note list has no keyboard focus model.
  'paste_plain_text', // The editor has no paste-as-plain-text action.
  'select_next', // The note list has no keyboard row-selection model.
  'select_previous', // The note list has no keyboard row-selection model.
  'select_all_notes', // The note list has no select-all event.
  'select_one_note', // The note list has no keyboard row-selection model.
  'open_focused_in_new_tab', // There is no keyboard-focused note to open.
  'focus_editor', // The note list has no keyboard-focused row to open.
  'select_next_task', // Tasks view has no keyboard selection model.
  'select_previous_task', // Tasks view has no keyboard selection model.
  'toggle_task', // Tasks view has no keyboard selection model.
  'open_task', // Tasks view has no keyboard selection model.
  'undo_check_off', // Tasks view has no keyboard selection model.
  'previous_day_cell', // Calendar view has no keyboard selection model.
  'next_day_cell', // Calendar view has no keyboard selection model.
  'previous_week', // Calendar view has no keyboard selection model.
  'next_week', // Calendar view has no keyboard selection model.
  'previous_month', // Calendar view has no keyboard selection model.
  'next_month', // Calendar view has no keyboard selection model.
  'go_to_today', // Calendar view has no keyboard selection model.
  'open_day', // Calendar view has no keyboard selection model.
  'close_graph', // Graph view has no keyboard selection model.
  'graph_zoom_in', // Graph view has no keyboard selection model.
  'graph_zoom_out', // Graph view has no keyboard selection model.
  'graph_show_all', // Graph view has no keyboard selection model.
  'select_card_up', // Board view has no keyboard selection model.
  'select_card_down', // Board view has no keyboard selection model.
  'select_card_left', // Board view has no keyboard selection model.
  'select_card_right', // Board view has no keyboard selection model.
  'move_card_left', // Board view has no keyboard selection model.
  'move_card_right', // Board view has no keyboard selection model.
  'open_card', // Board view has no keyboard selection model.
  'close_board', // Board view has no keyboard selection model.
  'confirm', // Palette navigation is not on mobile.
  'dismiss', // Palette navigation is not on mobile.
  'back', // Palette navigation is not on mobile.
};

/// The action behind an entry in the desktop-owned shortcut catalog.
///
/// An id with two key entries resolves to the same intent twice, preserving
/// both activators in the binding map.
Intent? intentForShortcut(ShortcutEntry entry) => switch (entry.id) {
  'new_note' => const NewNoteIntent(),
  'new_notebook' => const NewNotebookIntent(),
  'save_note' => const SaveIntent(),
  'open_today' => const OpenTodayIntent(),
  'paste_plain_text' => null, // The editor has no plain-text paste action.
  'format_bold' => const FormatIntent(FormatKind.bold),
  'format_italic' => const FormatIntent(FormatKind.italic),
  'format_strikethrough' => const FormatIntent(FormatKind.strikethrough),
  'format_code' => const FormatIntent(FormatKind.code),
  'format_link' => const FormatIntent(FormatKind.link),
  'format_heading_1' => const FormatIntent(FormatKind.heading1),
  'format_heading_2' => const FormatIntent(FormatKind.heading2),
  'format_heading_3' => const FormatIntent(FormatKind.heading3),
  'format_heading_4' => const FormatIntent(FormatKind.heading4),
  'format_heading_5' => const FormatIntent(FormatKind.heading5),
  'format_heading_6' => const FormatIntent(FormatKind.heading6),
  'format_task' => const FormatIntent(FormatKind.task),
  'format_bulleted' => const FormatIntent(FormatKind.bulleted),
  'format_numbered' => const FormatIntent(FormatKind.numbered),
  'format_quote' => const FormatIntent(FormatKind.quote),
  'format_code_block' => const FormatIntent(FormatKind.codeBlock),
  'toggle_pinned' => const PinNoteIntent(),
  'dictate' => null, // No voice-input action is reachable from the shell.
  'open_in_new_window' => const OpenInNewWindowIntent(),
  'close_note_window' => const CloseNoteWindowIntent(),
  'show_editor' => const EditNoteIntent(),
  'show_split' => null, // A phone has no split view.
  'show_preview' => const PreviewNoteIntent(),
  'toggle_sidebar' => const ToggleLibraryIntent(),
  'toggle_inspector' => const ToggleInspectorIntent(),
  'toggle_focus_mode' ||
  'exit_focus_mode' => null, // Focus mode is not on mobile.
  'ask_your_notes' => null, // The Ask screen is not on master.
  'search_all_notes' => const SearchAllIntent(),
  'replace_in_notes' => const ReplaceInNotesIntent(),
  'focus_search' ||
  'focus_note_list' => null, // Neither field has shell focus routing.
  'zoom_in' ||
  'zoom_out' ||
  'actual_size' => null, // The mobile app has no page zoom.
  'go_back' => const BackIntent(),
  'go_forward' => const ForwardIntent(),
  'next_note' => const NextNoteIntent(),
  'previous_note' => const PreviousNoteIntent(),
  'open_command_palette' => const OpenPaletteIntent(),
  'open_note_picker' => null, // There is no quick-open surface yet.
  'close_tab' => const CloseTabIntent(),
  'next_tab' => const NextTabIntent(),
  'previous_tab' => const PreviousTabIntent(),
  'reopen_closed_tab' => const ReopenClosedTabIntent(),
  'go_to_tab_1' => const GoToTabIntent(1),
  'go_to_tab_2' => const GoToTabIntent(2),
  'go_to_tab_3' => const GoToTabIntent(3),
  'go_to_tab_4' => const GoToTabIntent(4),
  'go_to_tab_5' => const GoToTabIntent(5),
  'go_to_tab_6' => const GoToTabIntent(6),
  'go_to_tab_7' => const GoToTabIntent(7),
  'go_to_tab_8' => const GoToTabIntent(8),
  'go_to_tab_9' => const GoToTabIntent(9),
  'open_settings' => const SettingsIntent(),
  'quit' => null, // The operating system owns quitting the app.
  'focus_editor' ||
  'select_next' ||
  'select_previous' ||
  'select_all_notes' ||
  'open_focused_in_new_tab' ||
  'select_one_note' => null, // The list has no keyboard-focused row model.
  'confirm' ||
  'dismiss' ||
  'back' => null, // Palette controls are not on mobile.
  'previous_day_cell' ||
  'next_day_cell' ||
  'previous_week' ||
  'next_week' ||
  'previous_month' ||
  'next_month' ||
  'go_to_today' ||
  'open_day' => null, // Calendar has no keyboard selection model.
  'select_next_task' ||
  'select_previous_task' ||
  'toggle_task' ||
  'open_task' ||
  'undo_check_off' => null, // Tasks have no keyboard selection model.
  'close_settings' => const CloseSettingsIntent(),
  'next_settings_page' => const SettingsPageStepIntent(forward: true),
  'previous_settings_page' => const SettingsPageStepIntent(forward: false),
  'close_graph' ||
  'graph_zoom_in' ||
  'graph_zoom_out' ||
  'graph_show_all' => null, // Graph has no keyboard selection model.
  'select_card_up' ||
  'select_card_down' ||
  'select_card_left' ||
  'select_card_right' ||
  'move_card_left' ||
  'move_card_right' ||
  'open_card' ||
  'close_board' => null, // Board has no keyboard selection model.
  'save_capture' ||
  'cancel_capture' => null, // Capture is a separate desktop window.
  _ => null,
};

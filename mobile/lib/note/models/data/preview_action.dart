// Kanso — preview_action.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:freezed_annotation/freezed_annotation.dart';

import '../../../core/bridge/api/types.dart';

part 'preview_action.freezed.dart';

/// What the screen does for a tap in the preview page that the page can't
/// do by itself: the core has said what the tap was, and the preview's BLoC
/// has done what needs the library.
@freezed
sealed class PreviewAction with _$PreviewAction {
  /// Opens the note [noteId].
  const factory PreviewAction.openNote(String noteId) = PreviewOpenNote;

  /// Opens the list of the notes tagged [name].
  const factory PreviewAction.openTag(String name) = PreviewOpenTag;

  /// Scrolls the page to the heading whose `id` is [anchor].
  const factory PreviewAction.scrollTo(String anchor) = PreviewScrollTo;

  /// Shows the attachment [file]: an image full screen, anything else in a
  /// sheet to share it from.
  const factory PreviewAction.showAttachment(AttachmentFile file) =
      PreviewShowAttachment;

  /// Puts [code], the [index]-th code block, on the clipboard.
  const factory PreviewAction.copyCode(int index, String code) =
      PreviewCopyCode;
}

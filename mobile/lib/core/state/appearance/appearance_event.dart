// Kanso — appearance_event.dart
// Created 2026-10-25.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'appearance_bloc.dart';

@freezed
sealed class AppearanceEvent with _$AppearanceEvent {
  /// The app started: read the saved appearance.
  const factory AppearanceEvent.started() = AppearanceStartedEvent;

  /// The person picked the theme [id]; [dark] is whether it is a dark one.
  const factory AppearanceEvent.themePicked(String id, {required bool dark}) =
      AppearanceThemePickedEvent;

  /// The person picked [id] to colour the Markdown source and the code in it;
  /// null for the interface's own theme.
  const factory AppearanceEvent.syntaxPicked(String? id) =
      AppearanceSyntaxPickedEvent;

  /// The person picked [id] to colour the rendered note; null for the
  /// interface's own theme.
  const factory AppearanceEvent.previewPicked(String? id) =
      AppearancePreviewPickedEvent;

  /// The person turned following the system's light and dark mode on or off.
  const factory AppearanceEvent.followSystemChanged({required bool follow}) =
      AppearanceFollowSystemChangedEvent;

  /// The person picked the icon theme [id] (`phosphor`); "lucide" is Kanso's
  /// own icons.
  const factory AppearanceEvent.iconThemePicked(String id) =
      AppearanceIconThemePickedEvent;

  /// The person picked [font] for writing in the editor.
  const factory AppearanceEvent.editorFontChanged(KansoFontFamily font) =
      AppearanceEditorFontChangedEvent;

  /// The person picked [font] for reading a note.
  const factory AppearanceEvent.previewFontChanged(KansoFontFamily font) =
      AppearancePreviewFontChangedEvent;

  /// The person picked [font] for code; a family that is not monospaced
  /// is ignored.
  const factory AppearanceEvent.codeFontChanged(KansoFontFamily font) =
      AppearanceCodeFontChangedEvent;

  /// The person picked [size] for the text of notes.
  const factory AppearanceEvent.textSizeChanged(KansoTextSize size) =
      AppearanceTextSizeChangedEvent;

  /// The person picked [size] for rendered previews.
  const factory AppearanceEvent.previewTextSizeChanged(KansoTextSize size) =
      AppearancePreviewTextSizeChangedEvent;

  /// The person picked [renderer] for Mermaid diagrams.
  const factory AppearanceEvent.diagramRendererChanged(
    DiagramRenderer renderer,
  ) = AppearanceDiagramRendererChangedEvent;

  /// The theme [id] installed from the gallery was installed again (updated)
  /// or removed: a theme in use is drawn again from its new tokens, and one
  /// that is gone is replaced, in any slot that used it, by Kanso's own.
  const factory AppearanceEvent.themeChanged(String id) =
      AppearanceThemeChangedEvent;
}

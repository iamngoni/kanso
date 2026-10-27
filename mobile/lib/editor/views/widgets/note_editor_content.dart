// Kanso — note_editor_content.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/types.dart' hide EditorEvent;
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../note/models/data/note_mode.dart';
import '../../../note/state/note/note_bloc.dart';
import '../../../shell/state/note_creation/note_creation_bloc.dart';
import '../../models/data/editor_destination.dart';
import '../../models/data/flush_reason.dart';
import '../../models/data/outside_side.dart';
import '../../models/repos/abs/editor_repository.dart';
import '../../state/editor/editor_bloc.dart';
import '../controllers/kanso_markdown_controller.dart';
import 'completion_host.dart';
import 'editor_conflict_strip.dart';
import 'editor_failure_strip.dart';
import 'editor_shortcuts.dart';
import 'editor_title_field.dart';
import 'keyboard_accessory_bar.dart';
import 'link_context_menu.dart';
import 'markdown_editor.dart';
import 'outside_text.dart';
import 'properties_section.dart';

/// The open note's editor: the strips for a change made elsewhere or a
/// failed save, the title, the properties, and the text, in one page that
/// scrolls.
///
/// It owns the field's controller and focus, the title's, and the page's
/// scroll position; it hands them what the `EditorBloc` says, and the
/// `EditorBloc` what they do. When it goes it leaves the caret and scroll
/// position behind for the next time, and the `EditorBloc` above it saves
/// what was typed.
class NoteEditorContent extends StatefulWidget {
  const NoteEditorContent({
    required this.document,
    required this.snapshot,
    required this.gutter,
    this.caret,
    this.caretInBody = false,
    this.scroll = 0,
    this.focusTitle = false,
    super.key,
  });

  /// The note open for editing.
  final EditorDocument document;

  /// The note as the editor opens it.
  final EditorSnapshot snapshot;

  /// The space either side of the page.
  final double gutter;

  /// Where the caret starts, in units of the text; the start when null.
  final int? caret;

  /// Whether [caret] counts the front matter that the text does not have.
  final bool caretInBody;

  /// How far the page was scrolled when the editor was left before.
  final double scroll;

  /// Whether the title, rather than the text, gets the focus (a new note).
  final bool focusTitle;

  @override
  State<NoteEditorContent> createState() => _NoteEditorContentState();
}

class _NoteEditorContentState extends State<NoteEditorContent> {
  late final EditorBloc editor;
  late final NoteBloc note;
  late final KansoMarkdownController controller;
  late final TextEditingController title;
  late final FocusNode textFocus;
  late final FocusNode titleFocus;
  late final ScrollController scroll;
  final GlobalKey<EditableTextState> editableKey =
      GlobalKey<EditableTextState>();
  late final AppLifecycleListener lifecycle;
  EditorContext? lastContext;

  // Keeping the place on the screen when the window moves: where in the
  // note, and where on the screen, the page was anchored before it did.
  final Key headerKey = const ValueKey<String>('editor header');
  final Key editorSliverKey = const ValueKey<String>('editor text');
  int seenWindow = 0;
  int? anchorOffset;
  double? anchorY;
  double? tappedY;

  @override
  void initState() {
    super.initState();
    editor = context.read<EditorBloc>();
    note = context.read<NoteBloc>();
    final caret = widget.caret;
    int? start;
    if (caret != null) {
      // The text is the end of the body: what is before it is the front
      // matter.
      final body = note.state.note?.body ?? '';
      final text = widget.snapshot.text;
      final hidden = widget.caretInBody && body.endsWith(text)
          ? body.length - text.length
          : 0;
      start = (caret - hidden).clamp(0, text.length);
    }
    controller = KansoMarkdownController(
      document: widget.document,
      snapshot: widget.snapshot,
      themeId: note.state.themeId,
      caret: start,
      onContext: told,
      onEdited: () => editor.add(const EditorEvent.edited()),
      onLocked: () => editor.add(
        // Rust says the lock is closed: the same as being told.
        const EditorEvent.signalReceived(EditorEvent_Closed()),
      ),
    )..onWindowWillMove = anchorAt;
    controller.addListener(keepPlace);
    title = TextEditingController(text: widget.snapshot.title);
    textFocus = FocusNode(debugLabel: 'Note text');
    titleFocus = FocusNode(debugLabel: 'Note title');
    // The page's place is kept from its top; with text before the window it
    // is from the editor, and the caret is shown as the field takes focus.
    scroll = ScrollController(
      initialScrollOffset: controller.hasBefore.value ? 0 : widget.scroll,
    );
    lifecycle = AppLifecycleListener(
      onStateChange: (state) => editor.add(EditorEvent.lifecycleChanged(state)),
    );
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        (widget.focusTitle ? titleFocus : textFocus).requestFocus();
      }
    });
  }

  /// What is under the caret: told on, unless it is what was told last and
  /// nothing completes there.
  void told(EditorContext context) {
    if (context == lastContext && !context.completionHere) {
      return;
    }
    lastContext = context;
    editor.add(EditorEvent.caretMoved(context));
  }

  @override
  void dispose() {
    lifecycle.dispose();
    controller.removeListener(keepPlace);
    if (!note.isClosed) {
      final selection = controller.noteSelection;
      note.add(
        NoteEvent.placeKept(
          selection.isValid ? selection.baseOffset : 0,
          scroll.hasClients && !controller.hasBefore.value ? scroll.offset : 0,
        ),
      );
    }
    scroll.dispose();
    titleFocus.dispose();
    textFocus.dispose();
    title.dispose();
    controller.dispose();
    super.dispose();
  }

  void onTitleChanged(String text) {
    widget.document.setTitle(text);
    editor.add(const EditorEvent.edited());
  }

  void focusText() {
    textFocus.requestFocus();
  }

  /// The blank space below the text puts the caret at its end.
  void focusEnd() {
    controller.placeCaret(controller.noteLength);
    textFocus.requestFocus();
  }

  /// Where on the screen the caret at [offset] of the note is drawn now;
  /// null when it is not in the field.
  double? yOf(int offset) {
    final at = controller.fieldOffsetOf(offset);
    final render = editableKey.currentState?.renderEditable;
    if (at == null || render == null || !render.attached) {
      return null;
    }
    final rect = render.getLocalRectForCaret(TextPosition(offset: at));
    return render.localToGlobal(rect.topLeft).dy;
  }

  /// The window is about to move: remember where [offset] is on the screen.
  void anchorAt(int offset) {
    anchorOffset = offset;
    anchorY = tappedY ?? yOf(offset);
    tappedY = null;
  }

  /// After the window has moved, scrolls the page so that what was anchored
  /// is where it was on the screen.
  void keepPlace() {
    if (controller.windowRevision == seenWindow) {
      return;
    }
    seenWindow = controller.windowRevision;
    final offset = anchorOffset;
    final before = anchorY;
    anchorOffset = null;
    anchorY = null;
    if (offset == null || before == null) {
      return;
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || !scroll.hasClients) {
        return;
      }
      final after = yOf(offset);
      if (after != null && (after - before).abs() > 0.5) {
        scroll.jumpTo(scroll.offset + (after - before));
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final gutter = widget.gutter;
    return MultiBlocListener(
      listeners: <BlocListener<dynamic, dynamic>>[
        BlocListener<EditorBloc, EditorState>(
          listenWhen: (before, now) => before.reloadSeq != now.reloadSeq,
          listener: (context, state) {
            final snapshot = state.snapshot;
            if (snapshot != null) {
              controller.reload(snapshot);
              if (title.text != snapshot.title) {
                title.text = snapshot.title;
              }
            }
          },
        ),
        BlocListener<EditorBloc, EditorState>(
          listenWhen: (before, now) => before.completion != now.completion,
          listener: (context, state) => controller.offer(state.completion),
        ),
        BlocListener<EditorBloc, EditorState>(
          listenWhen: (before, now) =>
              now.destination != null && before.destination != now.destination,
          listener: (context, state) {
            final destination = state.destination;
            editor.add(const EditorEvent.destinationHandled());
            switch (destination) {
              case NoteDestination(:final noteId):
                context.go(
                  BrowseLocation.place(
                    const ListPlace.all(),
                    noteId: noteId,
                  ).location,
                );
              case NewNoteDestination(:final title):
                context.read<NoteCreationBloc>().add(
                  NoteCreationEvent.titled(title),
                );
              case null:
                break;
            }
          },
        ),
        BlocListener<NoteBloc, NoteState>(
          listenWhen: (before, now) => before.themeId != now.themeId,
          listener: (context, state) {
            controller
              ..themeId = state.themeId
              ..requestRuns();
          },
        ),
      ],
      child: Padding(
        padding: EdgeInsets.only(
          bottom: MediaQuery.viewInsetsOf(context).bottom,
        ),
        child: Column(
          children: <Widget>[
            BlocBuilder<EditorBloc, EditorState>(
              buildWhen: (before, now) => before.save != now.save,
              builder: (context, state) {
                return switch (state.save) {
                  SaveState_Conflict(:final deleted) => EditorConflictStrip(
                    deleted: deleted,
                    gutter: gutter,
                    onKeepMine: () =>
                        editor.add(const EditorEvent.keepMineRequested()),
                    onUseTheirs: () =>
                        editor.add(const EditorEvent.useTheirsRequested()),
                  ),
                  SaveState_Failed() => EditorFailureStrip(
                    gutter: gutter,
                    onRetry: () => editor.add(
                      const EditorEvent.flushRequested(FlushReason.retry),
                    ),
                  ),
                  _ => const SizedBox.shrink(),
                };
              },
            ),
            Expanded(
              child: ValueListenableBuilder<bool>(
                valueListenable: controller.hasBefore,
                // A page that starts at the top (the whole note, or its
                // start, is in the field) grows down from its title. With
                // text before the window it grows from the editor, up and
                // down, so that text far from where the page is scrolled
                // to costs nothing to lay out.
                builder: (context, hasBefore, _) => CustomScrollView(
                  controller: scroll,
                  center: hasBefore ? editorSliverKey : headerKey,
                  slivers: <Widget>[
                    SliverPadding(
                      key: headerKey,
                      padding: EdgeInsets.fromLTRB(
                        gutter,
                        KansoSpacing.xxs,
                        gutter,
                        0,
                      ),
                      sliver: SliverToBoxAdapter(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: <Widget>[
                            EditorTitleField(
                              controller: title,
                              focusNode: titleFocus,
                              onChanged: onTitleChanged,
                              onSubmitted: focusText,
                            ),
                            BlocBuilder<EditorBloc, EditorState>(
                              buildWhen: (before, now) =>
                                  before.properties != now.properties ||
                                  before.propertiesOpen != now.propertiesOpen ||
                                  before.propertyError != now.propertyError,
                              builder: (context, state) => PropertiesSection(
                                properties: state.properties,
                                open: state.propertiesOpen,
                                failed: state.propertyError != null,
                                onToggle: () => editor.add(
                                  const EditorEvent.propertiesToggled(),
                                ),
                                onSet: (key, value, {index}) => editor.add(
                                  EditorEvent.propertySet(
                                    key: key,
                                    value: value,
                                    index: index,
                                  ),
                                ),
                                onRemove: (index) => editor.add(
                                  EditorEvent.propertyRemoved(index),
                                ),
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),
                    SliverPadding(
                      padding: EdgeInsets.symmetric(horizontal: gutter),
                      sliver: OutsideText(
                        controller: controller,
                        side: OutsideSide.before,
                        reverse: true,
                        onTapAt: (y) => tappedY = y,
                      ),
                    ),
                    SliverPadding(
                      key: editorSliverKey,
                      padding: EdgeInsets.symmetric(horizontal: gutter),
                      sliver: SliverToBoxAdapter(
                        child: EditorShortcuts(
                          controller: controller,
                          completionOpen: () => controller.completion != null,
                          onSave: () => editor.add(
                            const EditorEvent.flushRequested(
                              FlushReason.shortcut,
                            ),
                          ),
                          onDismissCompletion: () => editor.add(
                            const EditorEvent.completionDismissed(),
                          ),
                          onEscape: () {
                            textFocus.unfocus();
                            note.add(
                              const NoteEvent.modeChanged(NoteMode.preview),
                            );
                          },
                          child: CompletionHost(
                            controller: controller,
                            editableKey: editableKey,
                            reservedBottom: KansoSizes.touchTarget + 1,
                            child: MarkdownEditor(
                              controller: controller,
                              focusNode: textFocus,
                              editableKey: editableKey,
                              // The platform checks what the field holds: a
                              // window of a long note, never the whole.
                              spellCheckService: DefaultSpellCheckService(),
                              menuBuilder: (context, state) => LinkContextMenu(
                                state: state,
                                document: widget.document,
                                windowStart: controller.windowStart,
                                onOpen: (target) => editor.add(
                                  EditorEvent.linkOpenRequested(target),
                                ),
                              ),
                            ),
                          ),
                        ),
                      ),
                    ),
                    SliverPadding(
                      padding: EdgeInsets.symmetric(horizontal: gutter),
                      sliver: OutsideText(
                        controller: controller,
                        side: OutsideSide.after,
                        onTapAt: (y) => tappedY = y,
                      ),
                    ),
                    SliverFillRemaining(
                      hasScrollBody: false,
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTap: focusEnd,
                        child: SizedBox(
                          height:
                              MediaQuery.paddingOf(context).bottom +
                              KansoSpacing.xl,
                          width: double.infinity,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
            ListenableBuilder(
              listenable: textFocus,
              builder: (context, _) {
                if (!textFocus.hasFocus) {
                  return const SizedBox.shrink();
                }
                final insets = MediaQuery.viewInsetsOf(context).bottom;
                return Padding(
                  // With no keyboard (a hardware one), the home indicator.
                  padding: EdgeInsets.only(
                    bottom: insets > 0
                        ? 0
                        : MediaQuery.paddingOf(context).bottom,
                  ),
                  child: BlocBuilder<EditorBloc, EditorState>(
                    buildWhen: (before, now) => before.context != now.context,
                    builder: (context, state) => KeyboardAccessoryBar(
                      editorContext: state.context,
                      onCommand: controller.run,
                      onUndo: controller.undo,
                      onRedo: controller.redo,
                    ),
                  ),
                );
              },
            ),
          ],
        ),
      ),
    );
  }
}

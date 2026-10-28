// Kanso — note_editor_content.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../capture/models/data/capture_first_row.dart';
import '../../../capture/views/widgets/capture_sheet.dart';
import '../../../core/bridge/api/types.dart' hide EditorEvent;
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/note_drag_data.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../editor_extras/models/data/editor_extras.dart';
import '../../../editor_extras/models/data/markdown_editor_text_target.dart';
import '../../../editor_extras/views/widgets/editor_extras_scope.dart';
import '../../../note/models/data/note_mode.dart';
import '../../../note/state/note/note_bloc.dart';
import '../../../note_list/views/widgets/note_list_notice.dart';
import '../../../sharing/state/comments/comments_bloc.dart';
import '../../../sharing/state/shared_note_status/shared_note_status_bloc.dart';
import '../../../sharing/views/widgets/comment_compose_sheet.dart';
import '../../../shell/state/note_creation/note_creation_bloc.dart';
import '../../models/data/editor_destination.dart';
import '../../models/data/flush_reason.dart';
import '../../models/data/outside_side.dart';
import '../../models/repos/abs/editor_repository.dart';
import '../../state/editor/editor_bloc.dart';
import '../controllers/kanso_markdown_controller.dart';
import 'completion_host.dart';
import 'editor_conflict_strip.dart';
import 'editor_destination_scope.dart';
import 'editor_failure_strip.dart';
import 'editor_shortcuts.dart';
import 'editor_title_field.dart';
import 'keyboard_accessory_bar.dart';
import 'link_context_menu.dart';
import 'markdown_editor.dart';
import 'outside_text.dart';
import 'properties_section.dart';
import 'remote_carets.dart';

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
  late final MarkdownEditorTextTarget textTarget;
  late final TextEditingController title;
  late final FocusNode textFocus;
  late final FocusNode titleFocus;
  late final ScrollController scroll;
  final GlobalKey<EditableTextState> editableKey =
      GlobalKey<EditableTextState>();
  late final AppLifecycleListener lifecycle;
  EditorContext? lastContext;

  /// The note's comments, where the page has them.
  CommentsBloc? comments;

  /// The features currently adding keys and actions to this editor.
  EditorExtras? editorExtras;

  /// What the page says of the note's sharing, where it has that.
  SharedNoteStatusBloc? status;

  /// Whether the note is shared with this account to view or comment on
  /// only: its title, properties and text are for reading.
  bool readOnly = false;

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
    try {
      comments = context.read<CommentsBloc>();
    } on ProviderNotFoundException {
      comments = null;
    }
    try {
      status = context.read<SharedNoteStatusBloc>();
    } on ProviderNotFoundException {
      status = null;
    }
    readOnly = status?.state.readOnly ?? false;
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
    textTarget = MarkdownEditorTextTarget(
      controller: controller,
      noteId: note.state.noteId,
    );
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
    if (status != null) {
      unawaited(refreshCursors());
    }
    if (comments != null) {
      unawaited(refreshMarks());
      final selecting = comments?.state.selecting;
      if (selecting != null) {
        unawaited(selectComment(selecting));
      }
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final next = EditorExtrasScope.of(context);
    if (!identical(editorExtras, next)) {
      editorExtras?.release(textTarget);
      editorExtras = next;
    }
  }

  /// Asks the Rust core where the open comments' words are now, to tint.
  Future<void> refreshMarks() async {
    final found = await widget.document.commentMarks();
    if (!mounted) {
      return;
    }
    found.fold((_) {}, (marks) {
      controller.commentMarks = <TextRange>[
        for (final mark in marks) TextRange(start: mark.start, end: mark.end),
      ];
    });
  }

  /// Asks the Rust core where other people's cursors are now.
  Future<void> refreshCursors() async {
    final found = await widget.document.remoteCursors();
    if (!mounted) {
      return;
    }
    found.fold((_) {}, (cursors) => controller.remoteCursors = cursors);
  }

  /// Selects the words comment [commentId] is on, for the person to see.
  Future<void> selectComment(String commentId) async {
    comments?.add(const CommentsEvent.selectionHandled());
    final found = await widget.document.selectComment(commentId);
    if (!mounted) {
      return;
    }
    found.fold((_) {}, (range) {
      if (range != null && controller.selectRange(range.start, range.end)) {
        textFocus.requestFocus();
      }
    });
  }

  /// Comment, from the text menu: asks for the words and puts them on the
  /// selection.
  void commentOnSelection() {
    final bloc = comments;
    if (bloc == null) {
      return;
    }
    final l10n = context.l10n;
    unawaited(
      CommentComposeSheet.show(
        context,
        title: l10n.commentNewTitle,
        action: l10n.commentNewAction,
        onSend: (words) async {
          final made = await widget.document.commentOnSelection(words);
          return made.fold((error) => error.message, (_) {
            bloc.add(const CommentsEvent.refreshed());
            unawaited(refreshMarks());
            return null;
          });
        },
      ),
    );
  }

  /// Offers the camera, photos and PDFs to attach where the caret is, or
  /// an image's Markdown to write by hand.
  void attach() {
    final l10n = context.l10n;
    unawaited(
      CaptureSheet.show(
        context,
        first: CaptureFirstRow(
          icon: KansoIcons.link,
          label: l10n.captureImageLink,
          onTap: () => controller.run(
            const EditCommand.format(format: FormatKind.image),
          ),
        ),
        onSource: (source) => editor.add(EditorEvent.attachRequested(source)),
      ),
    );
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
    editorExtras?.release(textTarget);
    textTarget.dispose();
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
              textTarget.invalidate();
              controller.reload(snapshot);
              if (title.text != snapshot.title) {
                title.text = snapshot.title;
              }
              if (comments != null) {
                unawaited(refreshMarks());
              }
            }
          },
        ),
        BlocListener<EditorBloc, EditorState>(
          listenWhen: (before, now) => before.attachSeq != now.attachSeq,
          listener: (context, state) {
            final attached = state.attached;
            if (attached != null) {
              textTarget.invalidate();
              controller.adopt(attached);
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
            final scope = EditorDestinationScope.maybeOf(context);
            if (scope != null && destination != null) {
              scope.onDestination(context, destination);
              return;
            }
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
        if (comments case final comments?) ...<BlocListener<dynamic, dynamic>>[
          BlocListener<CommentsBloc, CommentsState>(
            bloc: comments,
            listenWhen: (before, now) =>
                before.comments != now.comments || before.role != now.role,
            listener: (context, state) => unawaited(refreshMarks()),
          ),
          BlocListener<CommentsBloc, CommentsState>(
            bloc: comments,
            listenWhen: (before, now) =>
                now.selecting != null && before.selecting != now.selecting,
            listener: (context, state) =>
                unawaited(selectComment(state.selecting!)),
          ),
          BlocListener<EditorBloc, EditorState>(
            // Saved: where the words are now is the Rust core's to say.
            listenWhen: (before, now) =>
                before.save != now.save && now.save is SaveState_Saved,
            listener: (context, state) => unawaited(refreshMarks()),
          ),
        ],
        if (status case final status?) ...<BlocListener<dynamic, dynamic>>[
          BlocListener<SharedNoteStatusBloc, SharedNoteStatusState>(
            bloc: status,
            listenWhen: (before, now) => before.readOnly != now.readOnly,
            listener: (context, state) =>
                setState(() => readOnly = state.readOnly),
          ),
          BlocListener<SharedNoteStatusBloc, SharedNoteStatusState>(
            bloc: status,
            listenWhen: (before, now) => before.beat != now.beat,
            listener: (context, state) => unawaited(refreshCursors()),
          ),
        ],
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
            BlocBuilder<EditorBloc, EditorState>(
              buildWhen: (before, now) =>
                  before.attachProblems != now.attachProblems,
              builder: (context, state) {
                final problems = state.attachProblems;
                if (problems.isEmpty) {
                  return const SizedBox.shrink();
                }
                final l10n = context.l10n;
                return Padding(
                  padding: EdgeInsets.fromLTRB(
                    gutter,
                    KansoSpacing.sm,
                    gutter,
                    0,
                  ),
                  child: NoteListNotice(
                    message: <String>[
                      for (final problem in problems)
                        switch (problem) {
                          InvalidError(:final message) => message,
                          _ => l10n.captureAttachFailed,
                        },
                    ].join('\n'),
                    onDismiss: () => editor.add(
                      const EditorEvent.attachNoticeDismissed(),
                    ),
                  ),
                );
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
                              readOnly: readOnly,
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
                                readOnly: readOnly,
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
                            child: Stack(
                              children: <Widget>[
                                DragTarget<NoteDragData>(
                                  onWillAcceptWithDetails: (_) => !readOnly,
                                  onAcceptWithDetails: (details) {
                                    final links = <String>[
                                      for (final title in details.data.titles)
                                        if (title.isNotEmpty) '[[$title]]',
                                    ];
                                    if (links.isEmpty) {
                                      return;
                                    }
                                    final selection = controller.noteSelection;
                                    final start = selection.isValid
                                        ? selection.start
                                        : controller.noteLength;
                                    final end = selection.isValid
                                        ? selection.end
                                        : controller.noteLength;
                                    controller.run(
                                      EditCommand.insert(
                                        range: Range16(start: start, end: end),
                                        text: links.join(' '),
                                      ),
                                    );
                                    textFocus.requestFocus();
                                  },
                                  builder: (context, hovering, rejected) {
                                    return DecoratedBox(
                                      decoration: BoxDecoration(
                                        border: hovering.isEmpty
                                            ? null
                                            : Border.all(
                                                color: context.colors.accent,
                                                width: 2,
                                              ),
                                      ),
                                      child: MarkdownEditor(
                                        controller: controller,
                                        focusNode: textFocus,
                                        editableKey: editableKey,
                                        readOnly: readOnly,
                                        // The platform checks what the field
                                        // holds: a window of a long note,
                                        // never the whole.
                                        spellCheckService:
                                            DefaultSpellCheckService(),
                                        menuBuilder: (context, state) {
                                          final selectionActions = readOnly
                                              ? const <EditorSelectionAction>[]
                                              : editorExtras?.selectionActions(
                                                      textTarget,
                                                    ) ??
                                                    const <
                                                      EditorSelectionAction
                                                    >[];
                                          return LinkContextMenu(
                                            state: state,
                                            document: widget.document,
                                            windowStart: controller.windowStart,
                                            extraActions: [
                                              for (final action
                                                  in selectionActions)
                                                (
                                                  action.label,
                                                  () {
                                                    state.hideToolbar();
                                                    action.onPressed(
                                                      context,
                                                      textTarget,
                                                    );
                                                  },
                                                ),
                                            ],
                                            onOpen: (target) => editor.add(
                                              EditorEvent.linkOpenRequested(
                                                target,
                                              ),
                                            ),
                                            onComment:
                                                comments?.state.mayComment ??
                                                    false
                                                ? commentOnSelection
                                                : null,
                                          );
                                        },
                                      ),
                                    );
                                  },
                                ),
                                // Where the others in a shared note are.
                                if (status != null)
                                  Positioned.fill(
                                    child: RemoteCarets(
                                      controller: controller,
                                      editableKey: editableKey,
                                    ),
                                  ),
                              ],
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
                // Nothing to type in a note for reading.
                if (!textFocus.hasFocus || readOnly) {
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
                      onImage: attach,
                      extras:
                          editorExtras?.accessoryKeys(textTarget) ??
                          const <EditorAccessoryExtra>[],
                      target: textTarget,
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

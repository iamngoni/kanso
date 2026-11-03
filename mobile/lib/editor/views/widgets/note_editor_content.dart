// Kanso — note_editor_content.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../assist/utils/open_assist_action.dart';
import '../../../capture/models/data/capture_first_row.dart';
import '../../../capture/views/widgets/capture_sheet.dart';
import '../../../core/bridge/api/types.dart' hide EditorEvent;
import '../../../core/bridge/api/word_goal.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_palette.dart';
import '../../../core/configs/kanso_sizes.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/models/data/note_drag_data.dart';
import '../../../core/models/repos/abs/renderer_repository.dart';
import '../../../core/models/repos/abs/themes_repository.dart';
import '../../../core/state/appearance/appearance_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_feedback.dart';
import '../../../editor_extras/models/data/editor_extras.dart';
import '../../../editor_extras/models/data/markdown_editor_text_target.dart';
import '../../../editor_extras/views/widgets/editor_extras_scope.dart';
import '../../../injection.dart';
import '../../../metrics/models/repos/abs/usage_metrics_repository.dart';
import '../../../metrics/utils/count_usage.dart';
import '../../../note/models/data/note_mode.dart';
import '../../../note/state/note/note_bloc.dart';
import '../../../note/state/note_assist_intent/note_assist_intent_bloc.dart';
import '../../../note/state/preview/preview_bloc.dart'
    show PreviewThemes, previewBaseTextSize;
import '../../../note/state/resurface/resurface_bloc.dart';
import '../../../note/state/word_goal_intent/word_goal_intent_bloc.dart';
import '../../../note/views/widgets/resurface_pill.dart';
import '../../../note_list/views/widgets/note_list_notice.dart';
import '../../../people/models/repos/abs/people_repository.dart';
import '../../../people/views/widgets/page_panel.dart';
import '../../../sharing/state/comments/comments_bloc.dart';
import '../../../sharing/state/shared_note_status/shared_note_status_bloc.dart';
import '../../../sharing/views/widgets/comment_compose_sheet.dart';
import '../../../shell/state/note_creation/note_creation_bloc.dart';
import '../../models/data/editor_destination.dart';
import '../../models/data/flush_reason.dart';
import '../../models/data/outside_side.dart';
import '../../models/data/word_goal_choice.dart';
import '../../models/repos/abs/editor_repository.dart';
import '../../state/editor/editor_bloc.dart';
import '../../utils/live_document/live_preview_drawer.dart';
import '../controllers/kanso_markdown_controller.dart';
import '../controllers/live_layout_controller.dart';
import 'completion_host.dart';
import 'editor_conflict_strip.dart';
import 'editor_destination_scope.dart';
import 'editor_failure_strip.dart';
import 'editor_shortcuts.dart';
import 'editor_title_field.dart';
import 'editor_word_goal_footer.dart';
import 'keyboard_accessory_bar.dart';
import 'link_context_menu.dart';
import 'live_document_editor.dart';
import 'markdown_editor.dart';
import 'outside_text.dart';
import 'properties_section.dart';
import 'remote_carets.dart';
import 'word_goal_picker_sheet.dart';

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
    this.caretEnd,
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

  /// Where a range to mark ends, in the units of [caret]. The range is
  /// selected for three seconds, then let go.
  final int? caretEnd;

  /// How far the page was scrolled when the editor was left before.
  final double scroll;

  /// Whether the title, rather than the text, gets the focus (a new note).
  final bool focusTitle;

  @override
  State<NoteEditorContent> createState() => _NoteEditorContentState();
}

class _NoteEditorContentState extends State<NoteEditorContent> {
  /// A range of the text to mark for [markFor], where a receipt points.
  int? markStart;
  int? markEnd;
  Timer? markTimer;

  /// How long a range an Ask receipt points at stays marked.
  static const Duration markFor = Duration(seconds: 3);

  /// Selects the range the note opened at, then lets go of it.
  void markReceiptRange() {
    final from = markStart;
    final to = markEnd;
    if (from == null || to == null || to <= from) {
      return;
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) {
        return;
      }
      final length = controller.text.length;
      final a = from.clamp(0, length);
      final b = to.clamp(a, length);
      controller.selection = TextSelection(baseOffset: a, extentOffset: b);
      markTimer = Timer(markFor, () {
        if (!mounted) {
          return;
        }
        final now = controller.selection;
        if (now.baseOffset == a && now.extentOffset == b) {
          controller.selection = TextSelection.collapsed(offset: a);
        }
      });
    });
  }

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
  final GlobalKey<LiveDocumentEditorState> liveKey =
      GlobalKey<LiveDocumentEditorState>();

  /// Whether the note is in Live mode: drawn by the Live editor, as the
  /// preview draws it, rather than as Markdown in a field.
  bool live = false;

  /// What the Live editor draws, while the note is in Live mode.
  LiveLayoutController? liveLayout;

  /// What draws the preview's blocks of the text for the Live editor.
  LivePreviewDrawer? previewDrawer;
  late final AppLifecycleListener lifecycle;
  late final ValueNotifier<int> wordCount;
  Timer? wordCountTimer;
  EditorContext? lastContext;

  /// The note's comments, where the page has them.
  CommentsBloc? comments;

  /// The features currently adding keys and actions to this editor.
  EditorExtras? editorExtras;

  /// What the page says of the note's sharing, where it has that.
  SharedNoteStatusBloc? status;

  /// A palette or other app-level assistant request for this note.
  NoteAssistIntentBloc? assistIntents;

  /// A palette request to open this note's word-goal picker.
  WordGoalIntentBloc? wordGoalIntents;

  /// Past notes about the paragraph being written, where the page has them.
  ResurfaceBloc? resurface;

  /// Coalesces the changes told to [resurface].
  Timer? resurfaceTimer;

  /// The server request last taken, so a rebuild cannot run it again.
  int? handledAssistRequest;

  /// The word-goal request last taken, so a rebuild cannot run it again.
  int? handledWordGoalRequest;

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
    try {
      assistIntents = context.read<NoteAssistIntentBloc>();
    } on ProviderNotFoundException {
      assistIntents = null;
    }
    try {
      wordGoalIntents = context.read<WordGoalIntentBloc>();
    } on ProviderNotFoundException {
      wordGoalIntents = null;
    }
    try {
      resurface = context.read<ResurfaceBloc>();
    } on ProviderNotFoundException {
      resurface = null;
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
      final end = widget.caretEnd;
      if (end != null) {
        markStart = start;
        markEnd = (end - hidden).clamp(start, text.length);
      }
    }
    live = note.state.mode == NoteMode.live;
    controller = KansoMarkdownController(
      document: widget.document,
      snapshot: widget.snapshot,
      themeId: note.state.themeId,
      caret: start,
      unwindowed: live,
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
    markReceiptRange();
    if (live) {
      startLive();
    }
    wordCount = ValueNotifier<int>(
      wordGoalWordCount(text: widget.snapshot.text),
    );
    controller.addListener(scheduleWordCount);
    title = TextEditingController(text: widget.snapshot.title);
    if (resurface != null) {
      controller.addListener(tellResurface);
      title.addListener(tellResurface);
      tellResurface();
    }
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
      if (!mounted) {
        return;
      }
      // The command can arrive while the note is in Preview, before this
      // editor exists. Take that initial request once the field is mounted.
      _takeAssistIntent();
      _takeWordGoalIntent();
      (widget.focusTitle ? titleFocus : textFocus).requestFocus();
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

  /// Starts drawing the note in Live mode: the core's layout of the whole
  /// text, and the preview's blocks of it for its widgets.
  void startLive() {
    if (liveLayout != null) {
      return;
    }
    final drawer =
        getIt.isRegistered<RendererRepository>() &&
            getIt.isRegistered<ThemesRepository>()
        ? previewDrawer ??= LivePreviewDrawer(
            renderer: getIt<RendererRepository>(),
            themes: getIt<ThemesRepository>(),
            noteId: note.state.noteId,
            inForce: (
              ui: 'kanso-light',
              syntax: 'kanso-light',
              preview: 'kanso-light',
              dark: false,
            ),
            size: previewBaseTextSize,
          )
        : null;
    liveLayout = LiveLayoutController(
      controller: controller,
      previewSource: drawer?.draw,
    );
  }

  void stopLive() {
    liveLayout?.dispose();
    liveLayout = null;
  }

  /// The themes and the preview's text size in force, from the appearance
  /// above the editor where there is one; Kanso's own otherwise.
  (PreviewThemes, double, PaletteDto) liveLook(BuildContext context) {
    final dark = context.brightness == Brightness.dark;
    AppearanceState? appearance;
    try {
      appearance = context.watch<AppearanceBloc>().state;
    } on ProviderNotFoundException {
      appearance = null;
    }
    if (appearance == null) {
      final own = dark ? 'kanso-dark' : 'kanso-light';
      return (
        (ui: own, syntax: own, preview: own, dark: dark),
        previewBaseTextSize,
        (dark ? KansoPalette.kansoDark : KansoPalette.kansoLight).preview,
      );
    }
    final selection = appearance.selection;
    return (
      (
        ui: selection.uiFor(systemDark: dark),
        syntax: selection.syntaxFor(systemDark: dark),
        preview: selection.previewFor(systemDark: dark),
        dark: dark,
      ),
      previewBaseTextSize * appearance.previewTextSize,
      (dark ? appearance.dark : appearance.light).preview,
    );
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

  /// Runs the pending assistant request for this note, if there is one.
  ///
  /// Whole-note summaries belong to the page and have no editor action.
  void _takeAssistIntent() {
    final bloc = assistIntents;
    if (bloc == null) {
      return;
    }
    final intent = bloc.state;
    final action = intent.action;
    if (!intent.pending ||
        intent.noteId != note.state.noteId ||
        action == null ||
        intent.request == handledAssistRequest) {
      return;
    }
    handledAssistRequest = intent.request;
    bloc.add(NoteAssistIntentEvent.consumed(request: intent.request));
    if (readOnly) {
      KansoFeedback.show(context, context.l10n.assistReadOnly);
      return;
    }
    unawaited(openAssistAction(context, textTarget, action: action));
  }

  /// Runs the pending word-goal request for this note, if there is one.
  void _takeWordGoalIntent() {
    final bloc = wordGoalIntents;
    if (bloc == null) {
      return;
    }
    final intent = bloc.state;
    if (!intent.pending ||
        intent.noteId != note.state.noteId ||
        intent.request == handledWordGoalRequest) {
      return;
    }
    if (readOnly) {
      handledWordGoalRequest = intent.request;
      bloc.add(WordGoalIntentEvent.consumed(request: intent.request));
      KansoFeedback.show(context, context.l10n.assistReadOnly);
      return;
    }
    final state = editor.state;
    // The editor is still opening; keep the request for its ready state.
    if (state.wordGoalSetting == null) {
      return;
    }
    handledWordGoalRequest = intent.request;
    bloc.add(WordGoalIntentEvent.consumed(request: intent.request));
    unawaited(chooseWordGoal(state));
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
    markTimer?.cancel();
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
    wordCountTimer?.cancel();
    resurfaceTimer?.cancel();
    controller
      ..removeListener(tellResurface)
      ..removeListener(scheduleWordCount);
    wordCount.dispose();
    titleFocus.dispose();
    textFocus.dispose();
    title.dispose();
    stopLive();
    controller.dispose();
    super.dispose();
  }

  void onTitleChanged(String text) {
    widget.document.setTitle(text);
    editor.add(const EditorEvent.edited());
  }

  /// Tells the pill of related notes what is being written, once the
  /// changes slow down; it asks only after the text has paused.
  void tellResurface() {
    final bloc = resurface;
    if (bloc == null || readOnly) {
      return;
    }
    resurfaceTimer?.cancel();
    resurfaceTimer = Timer(const Duration(milliseconds: 250), () {
      if (!mounted || bloc.isClosed) {
        return;
      }
      final selection = controller.noteSelection;
      bloc.add(
        ResurfaceEvent.edited(
          title: title.text,
          body: controller.noteText,
          caret: selection.isValid
              ? selection.baseOffset
              : controller.noteLength,
        ),
      );
    });
  }

  /// A link to the note titled [noteTitle], where the caret is.
  void linkTo(String noteTitle) {
    if (noteTitle.isEmpty || readOnly) {
      return;
    }
    final selection = controller.noteSelection;
    final at = selection.isValid ? selection.end : controller.noteLength;
    controller.run(
      EditCommand.insert(
        range: Range16(start: at, end: at),
        text: '[[$noteTitle]]',
      ),
    );
    textFocus.requestFocus();
  }

  /// Debounces Markdown counting while a long note is being typed in.
  void scheduleWordCount() {
    wordCountTimer?.cancel();
    wordCountTimer = Timer(const Duration(milliseconds: 250), () {
      if (mounted) {
        wordCount.value = wordGoalWordCount(text: controller.text);
      }
    });
  }

  Future<void> chooseWordGoal(EditorState state) async {
    final setting = state.wordGoalSetting;
    if (setting == null) {
      return;
    }
    final current = switch (setting) {
      GoalSetting_Default() => const WordGoalDefault(),
      GoalSetting_Off() => const WordGoalOff(),
      GoalSetting_Words(:final field0) => WordGoalWords(field0),
    };
    final picked = await WordGoalPickerSheet.show(
      context,
      setting: current,
      defaultGoal: state.defaultWordGoal,
    );
    if (!mounted || picked == null) {
      return;
    }
    final selected = switch (picked) {
      WordGoalDefault() => const GoalSetting.default_(),
      WordGoalOff() => const GoalSetting.off(),
      WordGoalWords(:final words) => GoalSetting.words(words),
    };
    editor.add(EditorEvent.wordGoalChanged(selected));
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

  /// The title and the properties, above the text.
  Widget header(double gutter) {
    return SliverPadding(
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
    );
  }

  /// The room below the text, where a tap puts the caret at its end.
  Widget room() {
    return SliverFillRemaining(
      hasScrollBody: false,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: focusEnd,
        child: SizedBox(
          height: MediaQuery.paddingOf(context).bottom + KansoSpacing.xl,
          width: double.infinity,
        ),
      ),
    );
  }

  /// The page in Edit mode: the title and properties, then the field, in
  /// one scroll view; with a long note the field holds a window of it and
  /// the rest is drawn around it.
  Widget editScroll(BuildContext context, double gutter) {
    return ValueListenableBuilder<bool>(
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
          header(gutter),
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
                    const NoteEvent.modeChanged(
                      NoteMode.preview,
                    ),
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
                        onAcceptWithDetails: dropLinks,
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
                              spellCheckService: DefaultSpellCheckService(),
                              menuBuilder: (context, state) {
                                final selectionActions = readOnly
                                    ? const <EditorSelectionAction>[]
                                    : editorExtras?.selectionActions(
                                            textTarget,
                                          ) ??
                                          const <EditorSelectionAction>[];
                                return LinkContextMenu(
                                  state: state,
                                  document: widget.document,
                                  windowStart: controller.windowStart,
                                  extraActions: [
                                    for (final action in selectionActions)
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
                                  onComment: comments?.state.mayComment ?? false
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
          room(),
        ],
      ),
    );
  }

  /// The page in Live mode: the title and properties, then the note drawn
  /// by the Live editor, in one scroll view that lays out only the blocks on
  /// screen.
  Widget liveScroll(BuildContext context, double gutter) {
    final layout = liveLayout;
    if (layout == null) {
      return const SizedBox.shrink();
    }
    final (themes, size, palette) = liveLook(context);
    final drawer = previewDrawer;
    if (drawer != null && (drawer.inForce != themes || drawer.size != size)) {
      drawer
        ..inForce = themes
        ..size = size;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        liveLayout?.redrawPreview();
      });
    }
    return DragTarget<NoteDragData>(
      onWillAcceptWithDetails: (_) => !readOnly,
      onAcceptWithDetails: dropLinks,
      builder: (context, hovering, rejected) => CompletionHost(
        controller: controller,
        caretOnScreen: () => liveKey.currentState?.caretOnScreen(),
        reservedBottom: KansoSizes.touchTarget + 1,
        child: CustomScrollView(
          controller: scroll,
          slivers: <Widget>[
            header(gutter),
            SliverPadding(
              key: editorSliverKey,
              padding: EdgeInsets.symmetric(horizontal: gutter),
              sliver: EditorShortcuts(
                controller: controller,
                includeSemantics: false,
                completionOpen: () => controller.completion != null,
                onSave: () => editor.add(
                  const EditorEvent.flushRequested(FlushReason.shortcut),
                ),
                onDismissCompletion: () =>
                    editor.add(const EditorEvent.completionDismissed()),
                onEscape: () {
                  textFocus.unfocus();
                  note.add(const NoteEvent.modeChanged(NoteMode.preview));
                },
                child: LiveDocumentEditor(
                  key: liveKey,
                  controller: controller,
                  layout: layout,
                  focusNode: textFocus,
                  palette: palette,
                  readOnly: readOnly,
                  menuExtras: () {
                    final actions = readOnly
                        ? const <EditorSelectionAction>[]
                        : editorExtras?.selectionActions(textTarget) ??
                              const <EditorSelectionAction>[];
                    return <(String, VoidCallback)>[
                      for (final action in actions)
                        (
                          action.label,
                          () => action.onPressed(context, textTarget),
                        ),
                    ];
                  },
                  linkAt: (offset) => widget.document
                      .linkAt(offset)
                      .then(
                        (found) =>
                            found.fold<LinkTarget?>((_) => null, (hit) => hit),
                      ),
                  onOpenLink: (target) {
                    countUsage(UsageFeature.wikilinkFollowed);
                    editor.add(EditorEvent.linkOpenRequested(target));
                  },
                  onComment: comments?.state.mayComment ?? false
                      ? commentOnSelection
                      : null,
                ),
              ),
            ),
            // A person's or project's live sections, under the text.
            if (getIt.isRegistered<PeopleRepository>())
              SliverToBoxAdapter(
                child: PeoplePagePanel(noteId: note.state.noteId),
              ),
            room(),
          ],
        ),
      ),
    );
  }

  /// Notes dropped on the text: links to them where the selection is.
  void dropLinks(DragTargetDetails<NoteDragData> details) {
    final links = <String>[
      for (final title in details.data.titles)
        if (title.isNotEmpty) '[[$title]]',
    ];
    if (links.isEmpty) {
      return;
    }
    final selection = controller.noteSelection;
    final start = selection.isValid ? selection.start : controller.noteLength;
    final end = selection.isValid ? selection.end : controller.noteLength;
    controller.run(
      EditCommand.insert(
        range: Range16(start: start, end: end),
        text: links.join(' '),
      ),
    );
    textFocus.requestFocus();
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
        if (assistIntents
            case final intents?) ...<BlocListener<dynamic, dynamic>>[
          BlocListener<NoteAssistIntentBloc, NoteAssistIntentState>(
            bloc: intents,
            listenWhen: (before, now) =>
                now.pending &&
                now.noteId == note.state.noteId &&
                now.action != null &&
                now.request != before.request,
            listener: (context, state) => _takeAssistIntent(),
          ),
        ],
        if (wordGoalIntents
            case final intents?) ...<BlocListener<dynamic, dynamic>>[
          BlocListener<EditorBloc, EditorState>(
            listenWhen: (before, now) =>
                before.wordGoalSetting != now.wordGoalSetting &&
                now.wordGoalSetting != null,
            listener: (context, state) => _takeWordGoalIntent(),
          ),
          BlocListener<WordGoalIntentBloc, WordGoalIntentState>(
            bloc: intents,
            listenWhen: (before, now) =>
                now.pending &&
                now.noteId == note.state.noteId &&
                now.request != before.request,
            listener: (context, state) => _takeWordGoalIntent(),
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
        BlocListener<NoteBloc, NoteState>(
          listenWhen: (before, now) => before.mode != now.mode,
          listener: (context, state) {
            if (state.mode.isEditing) {
              final next = state.mode == NoteMode.live;
              if (next == live) {
                return;
              }
              setState(() {
                live = next;
                controller.unwindowed = next;
                if (next) {
                  startLive();
                } else {
                  stopLive();
                }
              });
            }
          },
        ),
        BlocListener<EditorBloc, EditorState>(
          listenWhen: (before, now) =>
              before.wordGoalError != now.wordGoalError &&
              now.wordGoalError != null,
          listener: (context, state) {
            KansoFeedback.show(
              context,
              state.wordGoalError!.message,
              tone: KansoFeedbackTone.error,
            );
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
              child: live
                  ? liveScroll(context, gutter)
                  : editScroll(context, gutter),
            ),
            BlocBuilder<EditorBloc, EditorState>(
              buildWhen: (before, now) =>
                  before.wordGoalSetting != now.wordGoalSetting ||
                  before.defaultWordGoal != now.defaultWordGoal ||
                  before.effectiveWordGoal != now.effectiveWordGoal,
              builder: (context, state) => ValueListenableBuilder<int>(
                valueListenable: wordCount,
                builder: (context, count, _) => EditorWordGoalFooter(
                  words: count,
                  goal: state.effectiveWordGoal,
                  onTap: readOnly ? null : () => chooseWordGoal(state),
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
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: <Widget>[
                      // Above the keys: a pill when past notes are about
                      // the paragraph being written.
                      ResurfacePillRow(
                        noteId: note.state.noteId,
                        onLink: linkTo,
                      ),
                      BlocBuilder<EditorBloc, EditorState>(
                        buildWhen: (before, now) =>
                            before.context != now.context,
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
                    ],
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

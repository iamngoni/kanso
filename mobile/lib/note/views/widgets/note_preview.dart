// Kanso — note_preview.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';
import 'dart:convert';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';
import 'package:webview_flutter/webview_flutter.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/browse_location.dart';
import '../../../core/state/library_index/library_index_bloc.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../injection.dart';
import '../../models/data/preview_action.dart';
import '../../models/data/source_line.dart';
import '../../state/note/note_bloc.dart';
import '../../state/preview/preview_bloc.dart';
import 'attachment_sheet.dart';
import 'note_source_text.dart';

/// The script the page is given once it has loaded: it says how tall the
/// page is whenever that changes (the page is as tall as its content and the
/// note's list scrolls it), and asks to edit on a double tap, as the
/// source text does. The page itself has no script.
const String _pageScript = '''
(function () {
  if (window.kansoReady) { return; }
  window.kansoReady = true;
  function post(message) { KansoPage.postMessage(JSON.stringify(message)); }
  function height() {
    post({ height: Math.ceil(document.documentElement.scrollHeight) });
  }
  new ResizeObserver(height).observe(document.documentElement);
  window.addEventListener('load', height);
  height();
  document.addEventListener('dblclick', function () { post({ edit: true }); });
})();
''';

/// A note's text in preview: the core's page, in a web view as tall as the
/// page, in the note's scroll. Every tap on an address in it goes to
/// [PreviewBloc]; the web view itself goes nowhere.
///
/// Where no web view can be made (the widget tests, which can't draw one),
/// it is the note's source text, as the editor shows it.
class NotePreview extends StatelessWidget {
  const NotePreview({
    required this.note,
    required this.lines,
    this.onEditAt,
    super.key,
  });

  final NoteView note;

  /// The text cut into lines, for the source text.
  final List<SourceLine> lines;

  /// Asks to edit at an offset in the text; a double tap on the page asks
  /// to edit at its start.
  final ValueChanged<int>? onEditAt;

  @override
  Widget build(BuildContext context) {
    if (WebViewPlatform.instance == null) {
      return NoteSourceText(
        text: note.body ?? '',
        lines: lines,
        onEditAt: onEditAt,
      );
    }
    return SliverToBoxAdapter(
      child: BlocProvider<PreviewBloc>(
        key: ValueKey<String>(note.id),
        create: (context) => getIt<PreviewBloc>()
          ..add(
            PreviewEvent.started(
              note.id,
              context.read<NoteBloc>().state.themeId,
            ),
          ),
        child: BlocListener<NoteBloc, NoteState>(
          listenWhen: (before, now) => before.themeId != now.themeId,
          listener: (context, state) => context.read<PreviewBloc>().add(
            PreviewEvent.themeChanged(state.themeId),
          ),
          child: PreviewPage(onEdit: () => onEditAt?.call(0)),
        ),
      ),
    );
  }
}

/// The page of [PreviewBloc] in a web view, and what the screen does for a
/// tap in it.
class PreviewPage extends StatefulWidget {
  const PreviewPage({this.onEdit, super.key});

  final VoidCallback? onEdit;

  @override
  State<PreviewPage> createState() => _PreviewPageState();
}

class _PreviewPageState extends State<PreviewPage> {
  late final WebViewController controller;

  /// The page shown, so that the same page isn't loaded twice.
  String? shown;

  /// How tall the page says it is; a screen's worth until it has said.
  double? height;

  @override
  void initState() {
    super.initState();
    controller = WebViewController()
      ..setJavaScriptMode(JavaScriptMode.unrestricted)
      ..setBackgroundColor(Colors.transparent)
      ..addJavaScriptChannel('KansoPage', onMessageReceived: onMessage)
      ..setNavigationDelegate(
        NavigationDelegate(
          onNavigationRequest: onNavigation,
          onPageFinished: (_) =>
              unawaited(controller.runJavaScript(_pageScript)),
        ),
      );
    if (defaultTargetPlatform == TargetPlatform.iOS ||
        defaultTargetPlatform == TargetPlatform.android) {
      unawaited(controller.setVerticalScrollBarEnabled(false));
    }
    // A page drawn before this was built.
    if (context.read<PreviewBloc>().state.html case final html?) {
      load(html);
    }
  }

  /// Only the page itself is loaded: anything tapped is handed to the core.
  NavigationDecision onNavigation(NavigationRequest request) {
    final url = request.url;
    if (url == 'about:blank' || url.startsWith('data:')) {
      return NavigationDecision.navigate;
    }
    // A link to a heading in the page is `#id` on the page's address.
    final tapped = url.startsWith('about:blank#') ? url.substring(11) : url;
    context.read<PreviewBloc>().add(PreviewEvent.linkTapped(tapped));
    return NavigationDecision.prevent;
  }

  void onMessage(JavaScriptMessage message) {
    final data = jsonDecode(message.message);
    if (data is! Map<String, dynamic> || !mounted) {
      return;
    }
    if (data['height'] case final num reported) {
      final next = reported.toDouble();
      if (next > 0 && next != height) {
        setState(() => height = next);
      }
    }
    if (data['edit'] == true) {
      widget.onEdit?.call();
    }
  }

  void load(String html) {
    if (html == shown) {
      return;
    }
    shown = html;
    unawaited(controller.loadHtmlString(html));
  }

  /// Scrolls the note so that the heading [anchor] is at the top.
  Future<void> scrollTo(String anchor) async {
    final result = await controller.runJavaScriptReturningResult(
      '(function () { var e = document.getElementById(${jsonEncode(anchor)});'
      ' return e ? e.getBoundingClientRect().top + window.scrollY : -1; })()',
    );
    final top = double.tryParse('$result') ?? -1;
    if (!mounted || top < 0) {
      return;
    }
    final scrollable = Scrollable.maybeOf(context);
    final box = context.findRenderObject() as RenderBox?;
    final viewport = scrollable?.context.findRenderObject() as RenderBox?;
    if (scrollable == null || box == null || viewport == null) {
      return;
    }
    final position = scrollable.position;
    final here = box.localToGlobal(Offset.zero, ancestor: viewport).dy;
    final target = (position.pixels + here + top).clamp(
      position.minScrollExtent,
      position.maxScrollExtent,
    );
    await position.animateTo(
      target,
      duration: const Duration(milliseconds: 300),
      curve: Curves.easeOutCubic,
    );
  }

  void act(BuildContext context, PreviewAction action) {
    switch (action) {
      case PreviewOpenNote(:final noteId):
        context.go(
          BrowseLocation.place(const ListPlace.all(), noteId: noteId).location,
        );
      case PreviewOpenTag(:final name):
        final tags = context.read<LibraryIndexBloc>().state.tags;
        final tag = tags.where((tag) => tag.name == name).firstOrNull;
        if (tag != null) {
          context.go(BrowseLocation.place(ListPlace.tag(id: tag.id)).location);
        }
      case PreviewScrollTo(:final anchor):
        unawaited(scrollTo(anchor));
      case PreviewShowAttachment(:final file):
        unawaited(showAttachment(context, file));
      case PreviewCopyCode(:final code):
        unawaited(Clipboard.setData(ClipboardData(text: code)));
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return MultiBlocListener(
      listeners: <BlocListener<PreviewBloc, PreviewState>>[
        BlocListener<PreviewBloc, PreviewState>(
          listenWhen: (before, now) =>
              now.html != null && before.html != now.html,
          listener: (context, state) => load(state.html!),
        ),
        BlocListener<PreviewBloc, PreviewState>(
          listenWhen: (before, now) =>
              now.action != null && before.action != now.action,
          listener: (context, state) {
            final action = state.action!;
            context.read<PreviewBloc>().add(const PreviewEvent.actionHandled());
            act(context, action);
          },
        ),
        BlocListener<PreviewBloc, PreviewState>(
          listenWhen: (before, now) =>
              now.notice != null && before.notice != now.notice,
          listener: (context, state) {
            final notice = state.notice!;
            context.read<PreviewBloc>().add(
              const PreviewEvent.noticeDismissed(),
            );
            ScaffoldMessenger.maybeOf(context)?.showSnackBar(
              SnackBar(
                content: Text(
                  notice is LockedError
                      ? l10n.previewLinkLocked
                      : l10n.previewLinkFailed,
                ),
              ),
            );
          },
        ),
      ],
      child: BlocBuilder<PreviewBloc, PreviewState>(
        buildWhen: (before, now) =>
            before.error != now.error ||
            (before.html == null) != (now.html == null),
        builder: (context, state) {
          if (state.html == null && state.error != null) {
            return Padding(
              padding: const EdgeInsets.symmetric(vertical: 24),
              child: Text(
                l10n.previewFailed,
                style: context.typography.secondary,
              ),
            );
          }
          return SizedBox(
            height: height ?? MediaQuery.sizeOf(context).height / 2,
            child: Semantics(
              label: l10n.previewLabel,
              child: WebViewWidget(controller: controller),
            ),
          );
        },
      ),
    );
  }
}

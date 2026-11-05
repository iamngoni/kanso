// Kanso — pdf_reader_page.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';

import '../../../core/bridge/api/pdf_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_checkbox.dart';
import '../../../core/views/widgets/kanso_feedback.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_search_input.dart';
import '../../../core/views/widgets/kanso_text_field.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../../note/views/widgets/pdf_highlights_sheet.dart'
    show highlightColor;
import '../../models/data/pdf_zoom.dart';
import '../../models/repos/abs/pdf_reader_repository.dart';
import '../../state/pdf_reader/pdf_reader_bloc.dart';
import '../widgets/pdf_page_overlay.dart';
import '../widgets/pdf_page_view.dart';

/// Reads a PDF attachment of a note (issue #43): the pages as pictures, the
/// text selected by holding it, a search through the text, the note's
/// highlights, and highlighting what is selected.
class PdfReaderPage extends StatelessWidget {
  const PdfReaderPage({
    required this.noteId,
    required this.file,
    required this.name,
    this.page,
    this.highlight,
    this.overlays = const <PdfPageOverlay>[],
    super.key,
  });

  /// The note the attachment belongs to.
  final String noteId;

  /// The attachment, as a link names it (`file:<id>`).
  final String file;

  /// The file's name, shown in the bar.
  final String name;

  /// The page to open at, 1-based.
  final int? page;

  /// The highlight to keep in view, when the link named one.
  final String? highlight;

  /// Layers to draw over each page, for a later tool such as the pen.
  final List<PdfPageOverlay> overlays;

  @override
  Widget build(BuildContext context) {
    final dark = context.brightness == Brightness.dark;
    return BlocProvider<PdfReaderBloc>(
      create: (_) => getIt<PdfReaderBloc>()
        ..add(
          PdfReaderEvent.started(
            noteId: noteId,
            file: file,
            name: name,
            page: page,
            highlight: highlight,
            dark: dark,
          ),
        ),
      child: _PdfReaderView(overlays: overlays, dark: dark),
    );
  }
}

class _PdfReaderView extends StatefulWidget {
  const _PdfReaderView({required this.overlays, required this.dark});

  final List<PdfPageOverlay> overlays;
  final bool dark;

  @override
  State<_PdfReaderView> createState() => _PdfReaderViewState();
}

class _PdfReaderViewState extends State<_PdfReaderView> {
  final ScrollController _scroll = ScrollController();

  /// The top of every page in the list, and its height, for the last layout.
  List<double> _tops = const <double>[];
  double _pageWidth = 0;
  double _viewportHeight = 0;

  /// The page last read out of the scroll, so a scroll that stays on a page
  /// does not say it again.
  int _reported = 0;

  /// Whether the page a link named has been scrolled to.
  bool _placed = false;

  /// The zoom the list was laid out at, to keep the same page in view when
  /// it changes.
  PdfZoom? _laidOutZoom;

  bool _searching = false;

  /// The colour the next highlight is made in, and whether its quote goes
  /// into the note.
  String _color = 'yellow';
  bool _quote = false;

  @override
  void didUpdateWidget(covariant _PdfReaderView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.dark != widget.dark) {
      context.read<PdfReaderBloc>().add(
        PdfReaderEvent.darkChanged(dark: widget.dark),
      );
    }
  }

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  void _jumpTo(int page) {
    if (!_scroll.hasClients || _tops.isEmpty) {
      return;
    }
    final index = (page - 1).clamp(0, _tops.length - 1);
    final offset = _tops[index].clamp(0.0, _scroll.position.maxScrollExtent);
    _scroll.animateTo(
      offset,
      duration: const Duration(milliseconds: 240),
      curve: Curves.easeOutCubic,
    );
  }

  /// Lays the page tops out from what the document says its pages measure.
  void _layout(PdfInfo info, double width) {
    _pageWidth = width;
    final tops = <double>[];
    var top = KansoSpacing.xs;
    _pageHeight = <double>[];
    for (var index = 0; index < info.pages; index++) {
      final size = index < info.sizes.length
          ? info.sizes[index]
          : const PdfPageSize(width: 1, height: 1.414);
      final height = PdfPageView.heightFor(size, width);
      tops.add(top);
      _pageHeight.add(height);
      top += height + KansoSpacing.xs;
    }
    _tops = tops;
  }

  List<double> _pageHeight = const <double>[];

  bool _onScroll(ScrollNotification notification) {
    if (notification.depth != 0 || _tops.isEmpty) {
      return false;
    }
    final at = _scroll.offset + _viewportHeight * 0.3;
    var page = 1;
    for (var index = 0; index < _tops.length; index++) {
      if (_tops[index] <= at) {
        page = index + 1;
      } else {
        break;
      }
    }
    // The last page can be shorter than the one before it, and then no
    // scroll brings its 30% marker into view. At the end of the document,
    // that page is what is read.
    final atEnd =
        _scroll.hasClients &&
        _scroll.position.maxScrollExtent > 0 &&
        _scroll.offset >= _scroll.position.maxScrollExtent - 0.5;
    if (atEnd) {
      page = _tops.length;
    }
    if (page != _reported) {
      _reported = page;
      context.read<PdfReaderBloc>().add(PdfReaderEvent.pageChanged(page));
    }
    return false;
  }

  @override
  Widget build(BuildContext context) {
    return MultiBlocListener(
      listeners: <BlocListener<PdfReaderBloc, PdfReaderState>>[
        BlocListener<PdfReaderBloc, PdfReaderState>(
          listenWhen: (before, now) => before.jump != now.jump,
          listener: (context, state) => _jumpTo(state.page),
        ),
        BlocListener<PdfReaderBloc, PdfReaderState>(
          listenWhen: (before, now) =>
              now.notice != null && before.notice != now.notice,
          listener: (context, state) {
            final notice = state.notice!;
            context.read<PdfReaderBloc>().add(
              const PdfReaderEvent.noticeDismissed(),
            );
            KansoFeedback.show(
              context,
              notice.message,
              tone: KansoFeedbackTone.error,
            );
          },
        ),
      ],
      child: BlocBuilder<PdfReaderBloc, PdfReaderState>(
        builder: (context, state) {
          final info = state.info;
          final opened = state.status == PdfReaderStatus.opened;
          return KansoPage(
            resizeToAvoidBottomInset: false,
            body: Column(
              children: <Widget>[
                if (_searching && opened)
                  _SearchBar(
                    key: const ValueKey<String>('pdf-search-bar'),
                    onClose: () {
                      setState(() => _searching = false);
                      context.read<PdfReaderBloc>().add(
                        const PdfReaderEvent.searchClosed(),
                      );
                    },
                  )
                else
                  _bar(context, state, info),
                if (_searching && opened && state.query.trim().isNotEmpty)
                  _SearchStatus(state: state),
                Expanded(child: _body(context, state, info)),
                if (opened && state.inkDrawing)
                  _InkToolbar(
                    color: state.inkColor,
                    width: state.inkWidth,
                    saving: state.addingInk,
                    onColor: (color) => context.read<PdfReaderBloc>().add(
                      PdfReaderEvent.inkColorChanged(color),
                    ),
                    onWidth: (width) => context.read<PdfReaderBloc>().add(
                      PdfReaderEvent.inkWidthChanged(width),
                    ),
                  ),
                if (opened && state.selection != null)
                  _SelectionBar(
                    selection: state.selection!,
                    editable: state.marks?.editable ?? false,
                    adding: state.adding,
                    color: _color,
                    quote: _quote,
                    onColor: (color) => setState(() => _color = color),
                    onQuote: (quote) => setState(() => _quote = quote),
                    onHighlight: () => context.read<PdfReaderBloc>().add(
                      PdfReaderEvent.highlightRequested(
                        color: _color,
                        quote: _quote,
                      ),
                    ),
                  ),
                if (opened && state.selection == null && info != null)
                  _PageChip(
                    page: state.page,
                    pages: info.pages,
                    onTap: () => _jumpSheet(context, info.pages),
                  ),
              ],
            ),
          );
        },
      ),
    );
  }

  Widget _bar(BuildContext context, PdfReaderState state, PdfInfo? info) {
    final l10n = context.l10n;
    final marks = state.marks;
    return KansoTopBar(
      leading: KansoIconButton(
        icon: KansoIcons.arrowLeft,
        semanticLabel: l10n.pdfReaderClose,
        onPressed: () => Navigator.of(context).maybePop(),
      ),
      title: state.name,
      subtitle: info == null
          ? null
          : l10n.pdfReaderPageOf(state.page, info.pages),
      trailing: <Widget>[
        if (state.status == PdfReaderStatus.opened) ...<Widget>[
          KansoIconButton(
            icon: KansoIcons.search,
            semanticLabel: l10n.pdfReaderSearch,
            onPressed: () => setState(() => _searching = true),
          ),
          KansoIconButton(
            icon: KansoIcons.textSize,
            semanticLabel: l10n.pdfReaderZoom,
            onPressed: () => _zoomSheet(context, state.zoom),
          ),
          if (widget.dark)
            KansoIconButton(
              icon: state.invert ? KansoIcons.sun : KansoIcons.moon,
              semanticLabel: state.invert
                  ? l10n.pdfReaderOriginalPage
                  : l10n.pdfReaderInvertPage,
              onPressed: () => context.read<PdfReaderBloc>().add(
                PdfReaderEvent.invertChanged(invert: !state.invert),
              ),
            ),
          if (marks != null && marks.marks.isNotEmpty)
            KansoIconButton(
              icon: KansoIcons.highlighter,
              semanticLabel: l10n.pdfReaderHighlights,
              onPressed: () => _marksSheet(context),
            ),
          KansoIconButton(
            icon: KansoIcons.penLine,
            semanticLabel: 'Draw on PDF',
            selected: state.inkDrawing,
            onPressed: marks?.editable ?? false
                ? () => context.read<PdfReaderBloc>().add(
                    PdfReaderEvent.inkDrawingChanged(!state.inkDrawing),
                  )
                : null,
          ),
        ],
      ],
    );
  }

  Widget _body(BuildContext context, PdfReaderState state, PdfInfo? info) {
    switch (state.status) {
      case PdfReaderStatus.opening:
        return Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              const KansoSpinner(),
              const SizedBox(height: KansoSpacing.sm),
              Text(
                context.l10n.pdfReaderLoading,
                style: context.typography.secondary,
              ),
            ],
          ),
        );
      case PdfReaderStatus.failed:
        final error = state.error;
        return Center(
          child: Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: KansoMessageCard(
              title: context.l10n.pdfReaderUnavailableTitle,
              message: error?.message ?? context.l10n.pdfReaderOpenFailed,
              icon: KansoIcons.circleX,
              actions: <Widget>[
                KansoButton(
                  label: context.l10n.pdfReaderTryAgain,
                  onPressed: () => context.read<PdfReaderBloc>().add(
                    const PdfReaderEvent.retried(),
                  ),
                ),
                const SizedBox(height: KansoSpacing.xs),
                KansoButton(
                  label: context.l10n.pdfReaderClose,
                  variant: KansoButtonVariant.secondary,
                  onPressed: () => Navigator.of(context).maybePop(),
                ),
              ],
            ),
          ),
        );
      case PdfReaderStatus.declined:
        final declined = state.declined;
        return switch (declined) {
          PdfDeclined_NeedsPassword() ||
          PdfDeclined_WrongPassword() => _PasswordPanel(
            wrong: state.passwordWrong,
            onPassword: (password) => context.read<PdfReaderBloc>().add(
              PdfReaderEvent.passwordSubmitted(password),
            ),
            onClose: () => Navigator.of(context).maybePop(),
          ),
          PdfDeclined_LockedNote(:final message) => _message(
            context,
            title: context.l10n.pdfReaderLockedTitle,
            message: message,
            icon: KansoIcons.lock,
          ),
          PdfDeclined_Unavailable(:final message) => _message(
            context,
            title: context.l10n.pdfReaderUnavailableTitle,
            message: message,
            icon: KansoIcons.circleX,
          ),
          PdfDeclined_Refused(:final message) => _message(
            context,
            title: context.l10n.pdfReaderRefusedTitle,
            message: message,
            icon: KansoIcons.shieldAlert,
          ),
          PdfDeclined_NotAPdf() => _message(
            context,
            title: context.l10n.pdfReaderNotPdfTitle,
            message: context.l10n.pdfReaderNotPdfMessage,
            icon: KansoIcons.fileText,
          ),
          null => _message(
            context,
            title: context.l10n.pdfReaderUnavailableTitle,
            message: context.l10n.pdfReaderOpenFailed,
            icon: KansoIcons.circleX,
          ),
        };
      case PdfReaderStatus.opened:
        final document = info;
        final session = context.read<PdfReaderBloc>().session;
        if (document == null || session == null) {
          return const SizedBox.shrink();
        }
        return _pages(context, state, document, session);
    }
  }

  Widget _message(
    BuildContext context, {
    required String title,
    required String message,
    required KansoIcons icon,
  }) => Center(
    child: Padding(
      padding: const EdgeInsets.all(KansoSpacing.md),
      child: KansoMessageCard(
        title: title,
        message: message,
        icon: icon,
        actions: <Widget>[
          KansoButton(
            label: context.l10n.pdfReaderClose,
            variant: KansoButtonVariant.secondary,
            onPressed: () => Navigator.of(context).maybePop(),
          ),
        ],
      ),
    ),
  );

  Widget _pages(
    BuildContext context,
    PdfReaderState state,
    PdfInfo info,
    PdfDocumentSession session,
  ) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final viewportWidth = constraints.maxWidth;
        final viewportHeight = constraints.maxHeight;
        if (_pageWidth == 0 || _laidOutZoom != state.zoom) {
          _layout(info, viewportWidth * state.zoom.factor);
          _laidOutZoom = state.zoom;
          _placeAfterLayout(state.page);
        } else {
          _layout(info, viewportWidth * state.zoom.factor);
        }
        _viewportHeight = viewportHeight;
        if (!_placed) {
          _placed = true;
          _placeAfterLayout(state.page);
        }
        final contentWidth = math.max(viewportWidth, _pageWidth);
        final firstHit = state.found?.hits;
        final hit = firstHit != null && firstHit.isNotEmpty
            ? firstHit[math.min(state.hit, firstHit.length - 1)]
            : null;
        return SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          physics: _pageWidth > viewportWidth
              ? const ClampingScrollPhysics()
              : const NeverScrollableScrollPhysics(),
          child: SizedBox(
            width: contentWidth,
            height: viewportHeight,
            child: NotificationListener<ScrollNotification>(
              onNotification: _onScroll,
              child: ListView.builder(
                controller: _scroll,
                physics: state.inkDrawing
                    ? const NeverScrollableScrollPhysics()
                    : null,
                padding: const EdgeInsets.only(top: KansoSpacing.xs),
                itemCount: info.pages,
                itemBuilder: (context, index) {
                  final page = index + 1;
                  return Padding(
                    padding: const EdgeInsets.only(bottom: KansoSpacing.xs),
                    child: Align(
                      alignment: Alignment.topCenter,
                      child: PdfPageView(
                        key: ValueKey<String>('pdf-page-$page'),
                        session: session,
                        page: page,
                        size: index < info.sizes.length
                            ? info.sizes[index]
                            : const PdfPageSize(width: 1, height: 1.414),
                        width: _pageWidth,
                        invert: state.invert,
                        marks: <PdfMark>[
                          for (final mark
                              in state.marks?.marks ?? const <PdfMark>[])
                            if (mark.page == page) mark,
                        ],
                        ink: <PdfInkStroke>[
                          for (final inkPage
                              in state.marks?.ink ?? const <PdfInkPage>[])
                            if (inkPage.page == page) ...inkPage.strokes,
                        ],
                        selection: state.selection,
                        search: hit != null && hit.page == page
                            ? hit.marks
                                  .where((marks) => marks.page == page)
                                  .firstOrNull
                            : null,
                        focus: state.focusHighlight,
                        onWordSelected: (page, x, y) => context
                            .read<PdfReaderBloc>()
                            .add(PdfReaderEvent.wordSelected(page, x, y)),
                        onLineSelected: (page, x, y) => context
                            .read<PdfReaderBloc>()
                            .add(PdfReaderEvent.lineSelected(page, x, y)),
                        onSelectionExtended:
                            (page, x, y, {required fromStart}) =>
                                context.read<PdfReaderBloc>().add(
                                  PdfReaderEvent.selectionExtended(
                                    page,
                                    x,
                                    y,
                                    fromStart: fromStart,
                                  ),
                                ),
                        onSelectionCleared: () => context
                            .read<PdfReaderBloc>()
                            .add(const PdfReaderEvent.selectionCleared()),
                        inkDrawing: state.inkDrawing,
                        inkColor: state.inkColor,
                        inkWidth: state.inkWidth,
                        onInkStroke: (page, stroke) =>
                            context.read<PdfReaderBloc>().add(
                              PdfReaderEvent.inkStrokeRequested(
                                page: page,
                                stroke: stroke,
                              ),
                            ),
                        overlays: widget.overlays,
                      ),
                    ),
                  );
                },
              ),
            ),
          ),
        );
      },
    );
  }

  void _placeAfterLayout(int page) {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) {
        _jumpTo(page);
      }
    });
  }

  Future<void> _zoomSheet(BuildContext context, PdfZoom current) async {
    final l10n = context.l10n;
    final bloc = context.read<PdfReaderBloc>();
    await showKansoBottomSheet<void>(
      context,
      builder: (sheet) => BlocProvider<PdfReaderBloc>.value(
        value: bloc,
        child: SafeArea(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              Padding(
                padding: const EdgeInsets.all(KansoSpacing.md),
                child: Text(
                  l10n.pdfReaderZoom,
                  style: context.typography.heading2,
                ),
              ),
              for (final step in PdfZoom.steps)
                KansoPressable(
                  onTap: () {
                    context.read<PdfReaderBloc>().add(
                      PdfReaderEvent.zoomChanged(PdfZoom.of(step)),
                    );
                    Navigator.of(sheet).pop();
                  },
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: KansoSpacing.md,
                      vertical: KansoSpacing.sm,
                    ),
                    child: Row(
                      children: <Widget>[
                        Expanded(
                          child: Text(
                            step == 1
                                ? l10n.pdfReaderZoomFitWidth
                                : l10n.pdfReaderZoomPercent(
                                    (step * 100).round(),
                                  ),
                            style: context.typography.body,
                          ),
                        ),
                        if (current.factor == step)
                          KansoIcon(
                            KansoIcons.check,
                            size: 18,
                            color: context.colors.accent,
                          ),
                      ],
                    ),
                  ),
                ),
              const SizedBox(height: KansoSpacing.sm),
            ],
          ),
        ),
      ),
    );
  }

  Future<void> _marksSheet(BuildContext context) async {
    final l10n = context.l10n;
    final bloc = context.read<PdfReaderBloc>();
    final marks = bloc.state.marks?.marks ?? const <PdfMark>[];
    await showKansoBottomSheet<void>(
      context,
      builder: (sheet) => BlocProvider<PdfReaderBloc>.value(
        value: bloc,
        child: SafeArea(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 480),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                Padding(
                  padding: const EdgeInsets.all(KansoSpacing.md),
                  child: Text(
                    l10n.pdfReaderHighlights,
                    style: context.typography.heading2,
                  ),
                ),
                Flexible(
                  child: ListView(
                    shrinkWrap: true,
                    children: <Widget>[
                      for (final mark in marks)
                        KansoPressable(
                          onTap: () {
                            context.read<PdfReaderBloc>().add(
                              PdfReaderEvent.highlightFocused(mark.id),
                            );
                            Navigator.of(sheet).pop();
                          },
                          child: Padding(
                            padding: const EdgeInsets.symmetric(
                              horizontal: KansoSpacing.md,
                              vertical: KansoSpacing.sm,
                            ),
                            child: Row(
                              children: <Widget>[
                                Container(
                                  width: 18,
                                  height: 18,
                                  decoration: BoxDecoration(
                                    color: highlightColor(mark.color),
                                    borderRadius: BorderRadius.circular(4),
                                  ),
                                ),
                                const SizedBox(width: KansoSpacing.sm),
                                Text(
                                  l10n.pdfReaderHighlightPage(mark.page),
                                  style: context.typography.body,
                                ),
                              ],
                            ),
                          ),
                        ),
                    ],
                  ),
                ),
                const SizedBox(height: KansoSpacing.sm),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Future<void> _jumpSheet(BuildContext context, int pages) async {
    final bloc = context.read<PdfReaderBloc>();
    final page = await showKansoBottomSheet<int>(
      context,
      builder: (_) => _JumpSheet(pages: pages),
    );
    if (page != null) {
      bloc.add(PdfReaderEvent.jumpTo(page));
    }
  }
}

/// The bar while searching: the field, and the way out.
class _SearchBar extends StatelessWidget {
  const _SearchBar({required this.onClose, super.key});

  final VoidCallback onClose;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return ColoredBox(
      color: context.colors.editor,
      child: SafeArea(
        bottom: false,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.xs,
            KansoSpacing.xxs,
            KansoSpacing.xxs,
            KansoSpacing.xxs,
          ),
          child: Row(
            children: <Widget>[
              Expanded(
                child: KansoSearchInput(
                  hint: l10n.pdfReaderSearchHint,
                  autofocus: true,
                  clearLabel: l10n.pdfReaderSearchClear,
                  onChanged: (query) => context.read<PdfReaderBloc>().add(
                    PdfReaderEvent.searchChanged(query),
                  ),
                ),
              ),
              KansoIconButton(
                icon: KansoIcons.close,
                semanticLabel: l10n.pdfReaderClose,
                onPressed: onClose,
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// Compact pencil controls shown while drawing on the document.
class _InkToolbar extends StatelessWidget {
  const _InkToolbar({
    required this.color,
    required this.width,
    required this.saving,
    required this.onColor,
    required this.onWidth,
  });

  final String color;
  final double width;
  final bool saving;
  final ValueChanged<String> onColor;
  final ValueChanged<double> onWidth;

  static const List<String> colors = <String>[
    '#1e1e1e',
    '#e03131',
    '#1971c2',
    '#2f9e44',
  ];
  static const List<double> widths = <double>[2, 4, 7, 12];

  @override
  Widget build(BuildContext context) => ColoredBox(
    color: context.colors.editor,
    child: SafeArea(
      top: false,
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: KansoSpacing.sm,
          vertical: KansoSpacing.xs,
        ),
        child: SingleChildScrollView(
          scrollDirection: Axis.horizontal,
          child: Row(
            children: <Widget>[
              for (final value in colors)
                KansoPressable(
                  semanticLabel: 'Pencil color $value',
                  onTap: () => onColor(value),
                  child: Padding(
                    padding: const EdgeInsets.all(KansoSpacing.xxs),
                    child: Container(
                      width: 24,
                      height: 24,
                      decoration: BoxDecoration(
                        color: Color(
                          int.parse(value.substring(1), radix: 16) | 0xff000000,
                        ),
                        shape: BoxShape.circle,
                        border: Border.all(
                          color: color == value
                              ? context.colors.accent
                              : context.colors.divider,
                          width: color == value ? 2 : 1,
                        ),
                      ),
                    ),
                  ),
                ),
              const SizedBox(width: KansoSpacing.xs),
              for (final value in widths)
                KansoPressable(
                  semanticLabel: 'Pencil width ${value.round()}',
                  onTap: () => onWidth(value),
                  child: Container(
                    constraints: const BoxConstraints(
                      minWidth: 32,
                      minHeight: 36,
                    ),
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      color: width == value
                          ? context.colors.accent.withValues(alpha: 0.14)
                          : null,
                      borderRadius: BorderRadius.circular(8),
                    ),
                    child: Text(
                      '${value.round()}',
                      style: context.typography.secondary.copyWith(
                        color: width == value ? context.colors.accent : null,
                      ),
                    ),
                  ),
                ),
              if (saving) ...<Widget>[
                const SizedBox(width: KansoSpacing.xs),
                const SizedBox.square(
                  dimension: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
              ],
            ],
          ),
        ),
      ),
    ),
  );
}

/// How the search stands: the hit shown of the hits found, and the buttons
/// that move between them.
class _SearchStatus extends StatelessWidget {
  const _SearchStatus({required this.state});

  final PdfReaderState state;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final found = state.found;
    final total = found?.hits.length ?? 0;
    return ColoredBox(
      color: context.colors.editor,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(
          KansoSpacing.md,
          0,
          KansoSpacing.xs,
          KansoSpacing.xxs,
        ),
        child: Row(
          children: <Widget>[
            Expanded(
              child: Text(
                found == null
                    ? (state.query.trim().isEmpty
                          ? ''
                          : l10n.pdfReaderSearchSearching)
                    : total == 0
                    ? l10n.pdfReaderSearchNone
                    : l10n.pdfReaderSearchCount(state.hit + 1, total),
                style: context.typography.meta,
              ),
            ),
            if (found != null && found.pagesWithoutText > 0)
              Padding(
                padding: const EdgeInsetsDirectional.only(end: KansoSpacing.xs),
                child: Text(
                  l10n.pdfReaderSearchNoText(found.pagesWithoutText),
                  style: context.typography.meta,
                ),
              ),
            KansoIconButton(
              icon: KansoIcons.arrowUp,
              semanticLabel: l10n.pdfReaderPreviousMatch,
              onPressed: total == 0
                  ? null
                  : () => context.read<PdfReaderBloc>().add(
                      const PdfReaderEvent.searchPrevious(),
                    ),
            ),
            KansoIconButton(
              icon: KansoIcons.chevronDown,
              semanticLabel: l10n.pdfReaderNextMatch,
              onPressed: total == 0
                  ? null
                  : () => context.read<PdfReaderBloc>().add(
                      const PdfReaderEvent.searchNext(),
                    ),
            ),
          ],
        ),
      ),
    );
  }
}

/// What can be done with the selection: copy it, and highlight it in one of
/// the desktop's four colours, optionally putting its quote in the note.
class _SelectionBar extends StatelessWidget {
  const _SelectionBar({
    required this.selection,
    required this.editable,
    required this.adding,
    required this.color,
    required this.quote,
    required this.onColor,
    required this.onQuote,
    required this.onHighlight,
  });

  final PdfSelection selection;
  final bool editable;
  final bool adding;
  final String color;
  final bool quote;
  final ValueChanged<String> onColor;
  final ValueChanged<bool> onQuote;
  final VoidCallback onHighlight;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return Material(
      color: colors.elevated,
      elevation: 8,
      child: SafeArea(
        top: false,
        child: Padding(
          padding: const EdgeInsets.fromLTRB(
            KansoSpacing.md,
            KansoSpacing.xs,
            KansoSpacing.md,
            KansoSpacing.xs,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: <Widget>[
              if (selection.text.isNotEmpty)
                Padding(
                  padding: const EdgeInsets.only(bottom: KansoSpacing.xs),
                  child: Text(
                    selection.text,
                    maxLines: 2,
                    overflow: TextOverflow.ellipsis,
                    style: context.typography.secondary,
                  ),
                ),
              if (!editable)
                Text(
                  l10n.pdfReaderNotEditable,
                  style: context.typography.meta,
                )
              else
                Wrap(
                  crossAxisAlignment: WrapCrossAlignment.center,
                  spacing: KansoSpacing.xs,
                  runSpacing: KansoSpacing.xs,
                  children: <Widget>[
                    for (final name in const <String>[
                      'yellow',
                      'green',
                      'blue',
                      'pink',
                    ])
                      KansoPressable(
                        onTap: adding ? null : () => onColor(name),
                        semanticLabel: l10n.pdfNotesColor(name),
                        child: Container(
                          width: 26,
                          height: 26,
                          decoration: BoxDecoration(
                            color: highlightColor(name),
                            shape: BoxShape.circle,
                            border: Border.all(
                              width: 2,
                              color: color == name
                                  ? colors.text
                                  : colors.divider,
                            ),
                          ),
                        ),
                      ),
                    KansoPressable(
                      onTap: adding ? null : () => onQuote(!quote),
                      child: Row(
                        mainAxisSize: MainAxisSize.min,
                        children: <Widget>[
                          KansoCheckbox(value: quote, onChanged: onQuote),
                          const SizedBox(width: KansoSpacing.xxs),
                          Text(
                            l10n.pdfReaderAddQuote,
                            style: context.typography.label,
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              const SizedBox(height: KansoSpacing.xs),
              Row(
                children: <Widget>[
                  KansoIconButton(
                    icon: KansoIcons.copy,
                    semanticLabel: l10n.pdfReaderSelectionCopy,
                    onPressed: () async {
                      await Clipboard.setData(
                        ClipboardData(text: selection.text),
                      );
                      if (context.mounted) {
                        KansoFeedback.show(context, l10n.pdfReaderCopied);
                      }
                    },
                  ),
                  const Spacer(),
                  if (editable)
                    KansoButton(
                      label: adding
                          ? l10n.pdfReaderHighlighting
                          : l10n.pdfReaderHighlightAction,
                      compact: true,
                      expanded: false,
                      onPressed: adding ? null : onHighlight,
                    ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// The page in view, at the foot of the page: a tap asks for a page to go to.
class _PageChip extends StatelessWidget {
  const _PageChip({
    required this.page,
    required this.pages,
    required this.onTap,
  });

  final int page;
  final int pages;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return SafeArea(
      top: false,
      child: Padding(
        padding: const EdgeInsets.only(bottom: KansoSpacing.xxs),
        child: KansoPressable(
          onTap: onTap,
          semanticLabel: context.l10n.pdfReaderPageChip(page, pages),
          child: Container(
            padding: const EdgeInsets.symmetric(
              horizontal: KansoSpacing.sm,
              vertical: KansoSpacing.xxs,
            ),
            decoration: BoxDecoration(
              color: context.colors.elevated,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(color: context.colors.divider),
            ),
            child: Text(
              '$page / $pages',
              style: context.typography.meta,
            ),
          ),
        ),
      ),
    );
  }
}

/// A PDF that is password-protected: its password, and the way in.
class _PasswordPanel extends StatefulWidget {
  const _PasswordPanel({
    required this.wrong,
    required this.onPassword,
    required this.onClose,
  });

  final bool wrong;
  final ValueChanged<String> onPassword;
  final VoidCallback onClose;

  @override
  State<_PasswordPanel> createState() => _PasswordPanelState();
}

class _PasswordPanelState extends State<_PasswordPanel> {
  final TextEditingController _password = TextEditingController();

  @override
  void dispose() {
    _password.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Center(
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(KansoSpacing.lg),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: <Widget>[
            Text(
              l10n.pdfReaderPasswordTitle,
              textAlign: TextAlign.center,
              style: context.typography.heading2,
            ),
            const SizedBox(height: KansoSpacing.xs),
            Text(
              l10n.pdfReaderPasswordMessage,
              textAlign: TextAlign.center,
              style: context.typography.secondary,
            ),
            const SizedBox(height: KansoSpacing.md),
            KansoTextField(
              controller: _password,
              label: l10n.pdfReaderPasswordHint,
              obscure: true,
              autofillHints: const <String>[AutofillHints.password],
              textInputAction: TextInputAction.done,
              error: widget.wrong ? l10n.pdfReaderWrongPassword : null,
              onSubmitted: widget.onPassword,
            ),
            const SizedBox(height: KansoSpacing.md),
            KansoButton(
              label: l10n.pdfReaderUnlock,
              onPressed: () => widget.onPassword(_password.text),
            ),
            const SizedBox(height: KansoSpacing.xs),
            KansoButton(
              label: l10n.pdfReaderClose,
              variant: KansoButtonVariant.secondary,
              onPressed: widget.onClose,
            ),
          ],
        ),
      ),
    );
  }
}

/// Asks for a page number and answers with it, or null.
class _JumpSheet extends StatefulWidget {
  const _JumpSheet({required this.pages});

  final int pages;

  @override
  State<_JumpSheet> createState() => _JumpSheetState();
}

class _JumpSheetState extends State<_JumpSheet> {
  final TextEditingController _page = TextEditingController();
  String? _error;

  @override
  void dispose() {
    _page.dispose();
    super.dispose();
  }

  void _go() {
    final page = int.tryParse(_page.text.trim());
    if (page == null || page < 1 || page > widget.pages) {
      setState(() {
        _error = context.l10n.pdfReaderInvalidPage(widget.pages);
      });
      return;
    }
    Navigator.of(context).pop(page);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return Padding(
      padding: EdgeInsets.fromLTRB(
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.md,
        KansoSpacing.md + MediaQuery.paddingOf(context).bottom,
      ),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: <Widget>[
          Text(l10n.pdfReaderJumpToPage, style: context.typography.heading2),
          const SizedBox(height: KansoSpacing.md),
          KansoTextField(
            controller: _page,
            label: l10n.pdfReaderPageNumber,
            keyboardType: TextInputType.number,
            inputFormatters: <TextInputFormatter>[
              FilteringTextInputFormatter.digitsOnly,
            ],
            error: _error,
            textInputAction: TextInputAction.go,
            onSubmitted: (_) => _go(),
          ),
          const SizedBox(height: KansoSpacing.md),
          KansoButton(label: l10n.pdfReaderGo, onPressed: _go),
        ],
      ),
    );
  }
}

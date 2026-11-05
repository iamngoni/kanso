// Kanso — canvas_page.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../../core/bridge/api/canvas_types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/router/kanso_routes.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../../core/views/widgets/kanso_icon_button.dart';
import '../../../core/views/widgets/kanso_message_card.dart';
import '../../../core/views/widgets/kanso_page.dart';
import '../../../core/views/widgets/kanso_progress.dart';
import '../../../core/views/widgets/kanso_settings_row.dart';
import '../../../core/views/widgets/kanso_switch.dart';
import '../../../core/views/widgets/kanso_top_bar.dart';
import '../../../injection.dart';
import '../../../sharing/views/widgets/share_note_sheet.dart';
import '../../models/repos/abs/canvas_repository.dart';
import '../../state/canvas/canvas_bloc.dart';
import '../../utils/canvas_share.dart';
import '../widgets/canvas_banner.dart';
import '../widgets/canvas_card_sheet.dart';
import '../widgets/canvas_painter.dart';
import '../widgets/canvas_share_bar.dart';
import '../widgets/canvas_surface.dart';
import '../widgets/canvas_toolbar.dart';
import '../widgets/canvas_view_only_bar.dart';

/// A canvas note opened for drawing: pen, finger and eraser on an endless
/// page, with text and note cards and arrows. The document is saved when a
/// stroke ends and two seconds after anything else.
class CanvasPage extends StatelessWidget {
  const CanvasPage({required this.noteId, this.inPane = false, super.key});

  final String noteId;

  /// Whether it is the note pane of a wide window, with nowhere to go back to.
  final bool inPane;

  @override
  Widget build(BuildContext context) {
    return BlocProvider<CanvasBloc>(
      key: ValueKey<String>(noteId),
      create: (_) => getIt<CanvasBloc>()..add(CanvasEvent.started(noteId)),
      child: _CanvasScreen(inPane: inPane),
    );
  }
}

class _CanvasScreen extends StatefulWidget {
  const _CanvasScreen({required this.inPane});

  final bool inPane;

  @override
  State<_CanvasScreen> createState() => _CanvasScreenState();
}

class _CanvasScreenState extends State<_CanvasScreen> {
  final CanvasViewController _view = CanvasViewController();
  late final AppLifecycleListener _lifecycle;

  @override
  void initState() {
    super.initState();
    // The library leaves a shared canvas's room when the app goes to the
    // background, and the room's tick stops; coming back to the front joins
    // it again and starts the tick.
    _lifecycle = AppLifecycleListener(
      onPause: () => context.read<CanvasBloc>().add(const CanvasEvent.paused()),
      onResume: () =>
          context.read<CanvasBloc>().add(const CanvasEvent.resumed()),
    );
  }

  @override
  void dispose() {
    _lifecycle.dispose();
    _view.dispose();
    super.dispose();
  }

  CanvasTheme _theme(BuildContext context) {
    final colors = context.colors;
    return CanvasTheme(
      dark: context.isDark,
      text: colors.text,
      muted: colors.textMuted,
      surface: colors.elevated,
      border: colors.divider,
      accent: colors.accent,
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final colors = context.colors;
    return BlocBuilder<CanvasBloc, CanvasState>(
      builder: (context, state) {
        final bloc = context.read<CanvasBloc>();
        final view = state.view;
        return KansoPage(
          backgroundColor: colors.editor,
          resizeToAvoidBottomInset: false,
          body: Column(
            children: <Widget>[
              KansoTopBar(
                title: view == null
                    ? null
                    : (view.title.trim().isEmpty
                          ? l10n.canvasUntitled
                          : view.title),
                leading: widget.inPane
                    ? null
                    : KansoIconButton(
                        icon: KansoIcons.arrowLeft,
                        semanticLabel: l10n.canvasBack,
                        onPressed: () => context.pop(),
                      ),
                trailing: <Widget>[
                  if (!(view?.readOnly ?? false)) ...<Widget>[
                    KansoIconButton(
                      icon: KansoIcons.undo,
                      semanticLabel: l10n.canvasUndo,
                      onPressed: view?.canUndo ?? false
                          ? () => bloc.add(const CanvasEvent.undone())
                          : null,
                    ),
                    KansoIconButton(
                      icon: KansoIcons.redo2,
                      semanticLabel: l10n.canvasRedo,
                      onPressed: view?.canRedo ?? false
                          ? () => bloc.add(const CanvasEvent.redone())
                          : null,
                    ),
                  ],
                  KansoIconButton(
                    icon: KansoIcons.maximize,
                    semanticLabel: l10n.canvasFit,
                    onPressed: view == null
                        ? null
                        : () {
                            final b = view.bounds;
                            _view.fit(
                              b == null
                                  ? null
                                  : Rect.fromLTWH(b.x, b.y, b.width, b.height),
                            );
                          },
                  ),
                  KansoIconButton(
                    icon: KansoIcons.ellipsis,
                    semanticLabel: l10n.canvasMore,
                    onPressed: view == null
                        ? null
                        : () => _more(context, state),
                  ),
                ],
              ),
              Expanded(child: _body(context, state)),
            ],
          ),
        );
      },
    );
  }

  Widget _body(BuildContext context, CanvasState state) {
    final l10n = context.l10n;
    final bloc = context.read<CanvasBloc>();
    switch (state.status) {
      case CanvasStatus.loading:
        return const Center(child: KansoSpinner());
      case CanvasStatus.locked:
        return Center(
          child: Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: KansoMessageCard(
              icon: KansoIcons.lock,
              title: l10n.canvasLockedTitle,
              message: l10n.canvasLockedMessage,
              actions: const <Widget>[],
            ),
          ),
        );
      case CanvasStatus.failed:
        return Center(
          child: Padding(
            padding: const EdgeInsets.all(KansoSpacing.md),
            child: KansoMessageCard(
              icon: KansoIcons.circleX,
              title: l10n.canvasFailed,
              message: '',
              actions: const <Widget>[],
            ),
          ),
        );
      case CanvasStatus.ready:
        break;
    }
    final view = state.view!;
    final theme = _theme(context);
    return Column(
      children: <Widget>[
        CanvasShareBar(view: view, pointers: state.pointers),
        CanvasBanner(view: view, problem: state.problem),
        Expanded(
          child: Semantics(
            label: l10n.canvasSurface,
            child: CanvasSurface(
              view: view,
              tool: state.tool,
              color: state.color,
              width: state.width,
              fingerDraws: state.fingerDraws,
              stylusSeen: state.stylusSeen,
              selection: <String>{for (final k in state.selection) k.strokeId},
              edit: state.edit,
              readOnly: view.readOnly,
              shared: view.shared,
              pointers: state.pointers,
              erasedUnderPen: state.erasedUnderPen,
              theme: theme,
              controller: _view,
              preview: getIt<CanvasRepository>().preview,
              onStylusSeen: () => bloc.add(const CanvasEvent.stylusSeen()),
              onStroke: (samples) =>
                  bloc.add(CanvasEvent.strokeCommitted(samples)),
              onErase: (path, radius, {required partial}) => bloc.add(
                CanvasEvent.erased(
                  path: path,
                  radius: radius,
                  partial: partial,
                ),
              ),
              onLasso: (polygon) => bloc.add(CanvasEvent.lassoClosed(polygon)),
              onSelectionCleared: () =>
                  bloc.add(const CanvasEvent.selectionCleared()),
              onSelectionMoved: (delta) =>
                  bloc.add(CanvasEvent.selectionMoved(delta.dx, delta.dy)),
              onCardMoved: (id, rect) =>
                  bloc.add(CanvasEvent.cardMoved(id, rect)),
              onCardTapped: (card) => unawaited(_card(context, card)),
              onPlace: (at) => unawaited(_newCard(context, at)),
              onArrow: (from, to) => bloc.add(CanvasEvent.arrowAdded(from, to)),
              onGesture: ({required active}) =>
                  bloc.add(CanvasEvent.gestureChanged(active: active)),
              onStream: (samples) =>
                  bloc.add(CanvasEvent.strokeStreamed(samples)),
              onStrokeCancelled: () =>
                  bloc.add(const CanvasEvent.strokeCancelled()),
              onPointer: (at) => bloc.add(
                at == null
                    ? const CanvasEvent.pointerLeft()
                    : CanvasEvent.pointerMoved(at.dx, at.dy),
              ),
            ),
          ),
        ),
        if (view.readOnly)
          const CanvasViewOnlyBar()
        else
          CanvasToolbar(
            tool: state.tool,
            color: state.color,
            width: state.width,
            dark: theme.dark,
            text: theme.text,
            selected: state.selection,
            onTool: (tool) => bloc.add(CanvasEvent.toolChanged(tool)),
            onColor: (color) => bloc.add(CanvasEvent.colorChanged(color)),
            onWidth: (width) => bloc.add(CanvasEvent.widthChanged(width)),
            onDeleteSelection: () =>
                bloc.add(const CanvasEvent.selectionDeleted()),
            onClearSelection: () =>
                bloc.add(const CanvasEvent.selectionCleared()),
            onMoveToNew: () => bloc.add(
              CanvasEvent.selectionMovedToNewCanvas(l10n.canvasUntitled),
            ),
          ),
      ],
    );
  }

  Future<void> _newCard(BuildContext context, Offset at) async {
    final bloc = context.read<CanvasBloc>();
    final choice = await CanvasCardSheet.show(context);
    if (choice is CardText && choice.text.trim().isNotEmpty) {
      bloc.add(CanvasEvent.textCardAdded(at.dx, at.dy, choice.text));
    }
  }

  Future<void> _card(BuildContext context, CanvasCard card) async {
    final bloc = context.read<CanvasBloc>();
    final router = GoRouter.of(context);
    if (bloc.state.view?.readOnly ?? false) {
      // A canvas to look at: a note card opens its note, a text card has
      // nothing to change.
      final id = card.noteId;
      if (id != null) {
        unawaited(router.push('${KansoRoutes.noteById}/$id'));
      }
      return;
    }
    final choice = await CanvasCardSheet.show(context, card: card);
    switch (choice) {
      case CardText(:final text):
        bloc.add(CanvasEvent.cardTextChanged(card.id, text));
      case CardDelete():
        bloc.add(CanvasEvent.cardDeleted(card.id));
      case CardOpenNote():
        final id = card.noteId;
        if (id != null) {
          unawaited(router.push('${KansoRoutes.noteById}/$id'));
        }
      case null:
        break;
    }
  }

  Future<void> _more(BuildContext context, CanvasState state) {
    final bloc = context.read<CanvasBloc>();
    return showKansoBottomSheet<void>(
      context,
      builder: (sheet) => BlocProvider<CanvasBloc>.value(
        value: bloc,
        child: _MoreSheet(
          title: state.view?.title ?? '',
          noteId: state.view?.noteId ?? '',
        ),
      ),
    );
  }
}

class _MoreSheet extends StatelessWidget {
  const _MoreSheet({required this.title, required this.noteId});

  final String title;
  final String noteId;

  Future<void> _export(
    BuildContext context, {
    required bool excalidraw,
  }) async {
    final bloc = context.read<CanvasBloc>();
    final l10n = context.l10n;
    final dark = context.isDark;
    final navigator = Navigator.of(context);
    final document = bloc.document;
    if (document == null) {
      return;
    }
    final result = excalidraw
        ? await document.exportExcalidraw()
        : await document.exportSvg(dark: dark);
    final text = result.toOption().toNullable();
    final messenger = ScaffoldMessenger.maybeOf(context);
    navigator.pop();
    if (text == null) {
      messenger?.showSnackBar(SnackBar(content: Text(l10n.canvasExportFailed)));
      return;
    }
    await shareCanvasFile(
      text,
      name: canvasFileName(title, excalidraw ? 'excalidraw' : 'svg'),
      mimeType: excalidraw ? 'application/json' : 'image/svg+xml',
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return BlocBuilder<CanvasBloc, CanvasState>(
      builder: (context, state) => Column(
        mainAxisSize: MainAxisSize.min,
        children: <Widget>[
          if (!(state.view?.readOnly ?? false)) ...<Widget>[
            KansoSettingsRow(
              label: l10n.canvasFingerDraws,
              icon: KansoIcons.hand,
              trailing: KansoSwitch(
                value: state.fingerDraws,
                onChanged: (on) => context.read<CanvasBloc>().add(
                  CanvasEvent.fingerDrawsChanged(on: on),
                ),
              ),
            ),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: KansoSpacing.md),
              child: Text(
                l10n.canvasFingerDrawsHelp,
                style: context.typography.meta,
              ),
            ),
          ],
          KansoSettingsRow(
            label: l10n.canvasShareAction,
            icon: KansoIcons.users,
            onTap: () {
              final host = Navigator.of(context, rootNavigator: true).context;
              Navigator.of(context).pop();
              unawaited(ShareNoteSheet.show(host, noteId));
            },
          ),
          KansoSettingsRow(
            label: l10n.canvasExportSvg,
            icon: KansoIcons.share,
            onTap: () => unawaited(_export(context, excalidraw: false)),
          ),
          KansoSettingsRow(
            label: l10n.canvasExportExcalidraw,
            icon: KansoIcons.share,
            onTap: () => unawaited(_export(context, excalidraw: true)),
          ),
        ],
      ),
    );
  }
}

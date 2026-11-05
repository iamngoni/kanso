// Kanso — sketch_block.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/configs/kanso_icons.dart';
import '../../../core/configs/kanso_spacing.dart';
import '../../../core/utils/context_extensions.dart';
import '../../../core/views/widgets/kanso_button.dart';
import '../../../core/views/widgets/kanso_feedback.dart';
import '../../../core/views/widgets/kanso_icon.dart';
import '../../../core/views/widgets/kanso_pressable.dart';
import '../../../injection.dart';
import '../../models/data/canvas_tool.dart';
import '../../models/repos/abs/canvas_repository.dart';
import '../../state/canvas/canvas_bloc.dart';
import 'canvas_painter.dart';
import 'canvas_toolbar.dart';
import 'sketch_surface.dart';

/// The tools a sketch offers: the pens, the erasers and the lasso. It has no
/// text, card, arrow or hand tool.
const List<CanvasTool> sketchTools = <CanvasTool>[
  CanvasTool.pen,
  CanvasTool.pencil,
  CanvasTool.marker,
  CanvasTool.highlighter,
  CanvasTool.eraser,
  CanvasTool.partialEraser,
  CanvasTool.lasso,
];

/// An inline sketch in a note's preview (#45): the fixed box a `sketch`
/// fence holds, drawn on in place. Until the library opens it, it shows the
/// picture the core drew. A finger scrolls the note unless the person turns
/// "Draw with a finger" on; the pen and a mouse draw. A stroke is written
/// into the note's text two seconds after it ends, never per point, and what
/// is left is written when the block goes.
///
/// In a shared note it is read only: the picture, and "Copy as canvas".
class SketchBlock extends StatefulWidget {
  const SketchBlock({
    required this.noteId,
    required this.id,
    required this.width,
    required this.height,
    required this.svg,
    super.key,
  });

  final String noteId;

  /// The sketch's ID in the note.
  final String id;

  /// Its box, in points.
  final int width;
  final int height;

  /// The core's drawing of it, shown until it opens.
  final PreviewSvg svg;

  @override
  State<SketchBlock> createState() => _SketchBlockState();
}

class _SketchBlockState extends State<SketchBlock> {
  late final CanvasBloc _bloc = getIt<CanvasBloc>()
    ..add(CanvasEvent.sketchStarted(widget.noteId, widget.id));
  bool _active = false;
  bool _fingerDraws = false;

  @override
  void dispose() {
    // Closing writes what was drawn and not yet saved.
    _bloc.close();
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

  Future<void> _openAsCanvas(BuildContext context, bool readOnly) async {
    final doc = _bloc.document;
    final view = _bloc.state.view;
    if (doc == null) {
      return;
    }
    final l10n = context.l10n;
    final feedback = KansoFeedback.maybeOf(context);
    // What is drawn goes into the note before the canvas is made from it.
    final title = l10n.sketchTitle(
      (view?.title.trim().isEmpty ?? true)
          ? l10n.sketchUntitledNote
          : view!.title.trim(),
    );
    final made = await doc.openAsCanvas(title);
    if (!mounted) {
      return;
    }
    made.fold(
      (_) => feedback?.show(
        KansoFeedbackMessage(
          text: l10n.sketchFailed,
          tone: KansoFeedbackTone.error,
        ),
      ),
      (_) => feedback?.show(
        KansoFeedbackMessage(
          text: readOnly ? l10n.sketchCanvasCopied : l10n.sketchCanvasMade,
          tone: KansoFeedbackTone.success,
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return BlocProvider<CanvasBloc>.value(
      value: _bloc,
      child: BlocBuilder<CanvasBloc, CanvasState>(
        builder: _build,
      ),
    );
  }

  Widget _build(BuildContext context, CanvasState state) {
    final l10n = context.l10n;
    final colors = context.colors;
    final view = state.view;
    final info = _bloc.document?.sketchInfo;
    final readOnly = info?.readOnly ?? false;
    final box = Size(widget.width.toDouble(), widget.height.toDouble());
    final frame = BoxDecoration(
      color: colors.elevated,
      border: Border.all(color: colors.divider),
      borderRadius: BorderRadius.circular(KansoSpacing.sm),
    );
    final Widget surface;
    if (!state.ready || view == null || readOnly) {
      surface = SvgPicture.string(
        widget.svg.markup,
        width: box.width,
        height: box.height,
      );
    } else {
      surface = SketchSurface(
        view: view,
        box: box,
        tool: state.tool,
        color: state.color,
        width: state.width,
        fingerDraws: _fingerDraws,
        selection: state.selection.map((k) => k.strokeId).toSet(),
        edit: state.edit,
        theme: _theme(context),
        preview: (style, samples) =>
            getIt<CanvasRepository>().preview(style, samples),
        onStroke: (samples) => _bloc.add(CanvasEvent.strokeCommitted(samples)),
        onErase: (path, radius, {required partial}) => _bloc.add(
          CanvasEvent.erased(path: path, radius: radius, partial: partial),
        ),
        onLasso: (polygon) => _bloc.add(CanvasEvent.lassoClosed(polygon)),
        onSelectionCleared: () =>
            _bloc.add(const CanvasEvent.selectionCleared()),
        onSelectionMoved: (d) =>
            _bloc.add(CanvasEvent.selectionMoved(d.dx, d.dy)),
        onTouched: () {
          if (!_active) {
            setState(() => _active = true);
          }
        },
      );
    }
    final notice = readOnly
        ? l10n.sketchShared
        : switch (state.problem) {
            CanvasProblem.full => l10n.sketchFull,
            CanvasProblem.failed || CanvasProblem.notSaved => l10n.sketchFailed,
            null => null,
          };
    return Semantics(
      label: l10n.sketchLabel,
      container: true,
      child: TapRegion(
        onTapOutside: (_) {
          if (_active) {
            setState(() => _active = false);
          }
        },
        child: Align(
          alignment: Alignment.centerLeft,
          child: ConstrainedBox(
            constraints: BoxConstraints(maxWidth: box.width),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                DecoratedBox(
                  decoration: frame,
                  child: ClipRRect(
                    borderRadius: BorderRadius.circular(KansoSpacing.sm),
                    child: AspectRatio(
                      aspectRatio: box.width / box.height,
                      child: FittedBox(
                        fit: BoxFit.fill,
                        child: SizedBox(
                          width: box.width,
                          height: box.height,
                          child: surface,
                        ),
                      ),
                    ),
                  ),
                ),
                if (notice != null)
                  Padding(
                    padding: const EdgeInsets.only(top: KansoSpacing.xs),
                    child: Text(
                      notice,
                      style: context.typography.secondary,
                    ),
                  ),
                if (state.ready && (_active || readOnly))
                  _actions(context, state, readOnly),
                if (state.ready && _active && !readOnly) _toolbar(state),
              ],
            ),
          ),
        ),
      ),
    );
  }

  /// Undo, redo, the finger switch and "Open as canvas" under the box.
  Widget _actions(BuildContext context, CanvasState state, bool readOnly) {
    final l10n = context.l10n;
    final colors = context.colors;
    final view = state.view;
    Widget icon(KansoIcons glyph, String label, VoidCallback? onTap) =>
        Semantics(
          button: true,
          enabled: onTap != null,
          label: label,
          child: KansoPressable(
            onTap: onTap,
            child: SizedBox(
              width: 44,
              height: 44,
              child: Center(
                child: KansoIcon(
                  glyph,
                  color: onTap == null
                      ? colors.textMuted.withValues(alpha: 0.5)
                      : colors.textSecondary,
                ),
              ),
            ),
          ),
        );
    return Padding(
      padding: const EdgeInsets.only(top: KansoSpacing.xs),
      child: Row(
        children: <Widget>[
          if (!readOnly) ...<Widget>[
            icon(
              KansoIcons.undo,
              l10n.canvasUndo,
              view?.canUndo ?? false
                  ? () => _bloc.add(const CanvasEvent.undone())
                  : null,
            ),
            icon(
              KansoIcons.redo2,
              l10n.canvasRedo,
              view?.canRedo ?? false
                  ? () => _bloc.add(const CanvasEvent.redone())
                  : null,
            ),
            Semantics(
              toggled: _fingerDraws,
              label: l10n.sketchFingerDraws,
              child: KansoPressable(
                onTap: () => setState(() => _fingerDraws = !_fingerDraws),
                child: Container(
                  height: 44,
                  padding: const EdgeInsets.symmetric(
                    horizontal: KansoSpacing.sm,
                  ),
                  alignment: Alignment.center,
                  decoration: BoxDecoration(
                    color: _fingerDraws ? colors.selected : null,
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: KansoIcon(
                    KansoIcons.hand,
                    color: _fingerDraws ? colors.accent : colors.textSecondary,
                  ),
                ),
              ),
            ),
          ],
          const Spacer(),
          KansoButton(
            label: readOnly ? l10n.sketchCopyAsCanvas : l10n.sketchOpenAsCanvas,
            variant: KansoButtonVariant.quiet,
            compact: true,
            expanded: false,
            onPressed: () => _openAsCanvas(context, readOnly),
          ),
        ],
      ),
    );
  }

  Widget _toolbar(CanvasState state) {
    return MediaQuery.removePadding(
      context: context,
      removeBottom: true,
      child: CanvasToolbar(
        tool: state.tool,
        color: state.color,
        width: state.width,
        dark: context.isDark,
        text: context.colors.text,
        selected: state.selection,
        toolList: sketchTools,
        onTool: (tool) => _bloc.add(CanvasEvent.toolChanged(tool)),
        onColor: (color) => _bloc.add(CanvasEvent.colorChanged(color)),
        onWidth: (width) => _bloc.add(CanvasEvent.widthChanged(width)),
        onDeleteSelection: () =>
            _bloc.add(const CanvasEvent.selectionDeleted()),
        onClearSelection: () => _bloc.add(const CanvasEvent.selectionCleared()),
      ),
    );
  }
}

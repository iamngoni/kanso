// Kanso — completion_host.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';

import '../controllers/kanso_markdown_controller.dart';
import 'completion_popup.dart';

/// Shows the controller's completions over the [child] (the editor), at the
/// caret: below it, or above it where there is no room below, and never
/// behind the keyboard or the bar above it.
///
/// The popup is in the overlay, tied to the editor so that it scrolls with
/// the text, and never takes the focus.
class CompletionHost extends StatefulWidget {
  const CompletionHost({
    required this.controller,
    required this.editableKey,
    required this.child,
    this.reservedBottom = 0,
    super.key,
  });

  final KansoMarkdownController controller;

  /// The editable text the caret is in.
  final GlobalKey<EditableTextState> editableKey;

  final Widget child;

  /// What at the foot of the screen the popup keeps clear of besides the
  /// keyboard: the bar above it.
  final double reservedBottom;

  /// The most the popup is wide.
  static const double maxWidth = 400;

  /// The space between the caret and the popup.
  static const double gap = 4;

  @override
  State<CompletionHost> createState() => _CompletionHostState();
}

/// Where the caret is, in the editor and on the screen.
typedef _Caret = ({Rect local, double screenTop, double screenBottom});

class _CompletionHostState extends State<CompletionHost> {
  final OverlayPortalController portal = OverlayPortalController();
  final LayerLink link = LayerLink();
  final GlobalKey targetKey = GlobalKey();
  final ValueNotifier<_Caret?> caret = ValueNotifier<_Caret?>(null);

  @override
  void initState() {
    super.initState();
    widget.controller.addListener(sync);
    WidgetsBinding.instance.addPostFrameCallback((_) => sync());
  }

  @override
  void didUpdateWidget(CompletionHost oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controller != widget.controller) {
      oldWidget.controller.removeListener(sync);
      widget.controller.addListener(sync);
    }
  }

  @override
  void dispose() {
    widget.controller.removeListener(sync);
    caret.dispose();
    super.dispose();
  }

  bool get open {
    final offered = widget.controller.completion;
    return offered != null && offered.items.isNotEmpty;
  }

  /// Shows or hides the popup, and finds the caret once the text is laid
  /// out.
  void sync() {
    if (!mounted) {
      return;
    }
    if (open) {
      if (!portal.isShowing) {
        portal.show();
      }
      SchedulerBinding.instance.addPostFrameCallback((_) => measure());
    } else if (portal.isShowing) {
      portal.hide();
    }
  }

  void measure() {
    if (!mounted || !open) {
      return;
    }
    final editable = widget.editableKey.currentState;
    final target = targetKey.currentContext?.findRenderObject();
    final selection = widget.controller.selection;
    if (editable == null ||
        target is! RenderBox ||
        !target.attached ||
        !selection.isValid) {
      return;
    }
    final render = editable.renderEditable;
    final rect = render.getLocalRectForCaret(
      TextPosition(offset: selection.extentOffset),
    );
    final local = MatrixUtils.transformRect(
      render.getTransformTo(target),
      rect,
    );
    final top = render.localToGlobal(rect.topLeft).dy;
    final bottom = render.localToGlobal(rect.bottomLeft).dy;
    final next = (local: local, screenTop: top, screenBottom: bottom);
    if (caret.value != next) {
      caret.value = next;
    }
  }

  @override
  Widget build(BuildContext context) {
    return OverlayPortal(
      controller: portal,
      overlayChildBuilder: (overlayContext) {
        return ListenableBuilder(
          listenable: Listenable.merge(<Listenable>[widget.controller, caret]),
          builder: (context, _) {
            final offered = widget.controller.completion;
            final at = caret.value;
            if (offered == null || offered.items.isEmpty || at == null) {
              return const SizedBox.shrink();
            }
            final box = targetKey.currentContext?.findRenderObject();
            final editorWidth = box is RenderBox && box.hasSize
                ? box.size.width
                : CompletionHost.maxWidth;
            final width = math.min(editorWidth, CompletionHost.maxWidth);
            final height = CompletionPopup.heightFor(offered.items.length);
            final screen = MediaQuery.sizeOf(context);
            final limit =
                screen.height -
                MediaQuery.viewInsetsOf(context).bottom -
                widget.reservedBottom;
            final below = at.screenBottom + CompletionHost.gap;
            final fitsBelow = below + height <= limit;
            final dy = fitsBelow
                ? at.local.bottom + CompletionHost.gap
                : at.local.top - CompletionHost.gap - height;
            final dx = at.local.left.clamp(0.0, editorWidth - width);
            return Positioned(
              left: 0,
              top: 0,
              width: width,
              child: CompositedTransformFollower(
                link: link,
                showWhenUnlinked: false,
                offset: Offset(dx, dy),
                child: TextFieldTapRegion(
                  child: CompletionPopup(
                    completion: offered,
                    activeIndex: widget.controller.completionIndex,
                    onChoose: (item) {
                      widget.controller.insert(item);
                    },
                  ),
                ),
              ),
            );
          },
        );
      },
      child: CompositedTransformTarget(
        key: targetKey,
        link: link,
        child: widget.child,
      ),
    );
  }
}

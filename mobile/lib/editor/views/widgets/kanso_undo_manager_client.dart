// Kanso — kanso_undo_manager_client.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../../../core/bridge/api/types.dart' show HistoryState;
import '../controllers/kanso_markdown_controller.dart';

/// Takes iOS's system undo (shake to undo, a three-finger swipe, the iPad
/// keyboard's undo and redo) from Flutter's own history, which the editor
/// does not use, and gives it to the Rust core's.
///
/// Flutter's `EditableText` makes its `UndoHistory` the system's undo client
/// whenever the field gets focus. This takes the place back once that has
/// happened, and tells the system whether there is anything to undo.
class KansoUndoManagerClient with UndoManagerClient {
  KansoUndoManagerClient({required this.controller, required this.focusNode});

  final KansoMarkdownController controller;
  final FocusNode focusNode;

  bool _attached = false;
  HistoryState? _reported;

  /// Starts watching the focus.
  void attach() {
    if (_attached) {
      return;
    }
    _attached = true;
    focusNode.addListener(onFocus);
    controller.addListener(report);
    onFocus();
  }

  /// Lets go, and gives the place back to nobody.
  void detach() {
    if (!_attached) {
      return;
    }
    _attached = false;
    focusNode.removeListener(onFocus);
    controller.removeListener(report);
    if (UndoManager.client == this) {
      UndoManager.client = null;
    }
  }

  /// Takes the place when the field has focus. The field's own history takes
  /// it in a focus listener of its own, so this waits for the frame.
  void onFocus() {
    if (!focusNode.hasFocus) {
      if (UndoManager.client == this) {
        UndoManager.client = null;
      }
      return;
    }
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_attached && focusNode.hasFocus) {
        UndoManager.client = this;
        _reported = null;
        report();
      }
    });
  }

  /// Tells the system what can be undone now.
  void report() {
    final history = controller.history;
    if (UndoManager.client == this && history != _reported) {
      _reported = history;
      UndoManager.setUndoState(canUndo: canUndo, canRedo: canRedo);
    }
  }

  @override
  void handlePlatformUndo(UndoDirection direction) {
    switch (direction) {
      case UndoDirection.undo:
        undo();
      case UndoDirection.redo:
        redo();
    }
  }

  @override
  void undo() {
    controller.undo();
  }

  @override
  void redo() {
    controller.redo();
  }

  @override
  bool get canUndo => controller.history.canUndo;

  @override
  bool get canRedo => controller.history.canRedo;
}

// Kanso — editor_harness.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/editor.dart';
import 'package:kanso/core/bridge/api/library.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/configs/kanso_theme.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/editor/models/repos/impl/rust_editor_repository.dart';
import 'package:kanso/editor/utils/editor_window.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/markdown_editor.dart';
import 'package:kanso/gen/l10n/app_localizations.dart';

import 'core_library.dart';
import 'fake_editor_document.dart';

/// A note open for editing in the real Rust core, with the editor drawn on
/// it: for widget tests that type against the library.
class EditorHarness {
  EditorHarness({
    required this.library,
    required this.noteId,
    required this.editor,
    required this.document,
    required this.controller,
    required this.focusNode,
  });

  final KansoLibrary library;
  final String noteId;
  final KansoEditor editor;
  final RustEditorDocument document;
  final KansoMarkdownController controller;
  final FocusNode focusNode;

  /// The text Rust has, which must always be the field's.
  Future<String> rustText(WidgetTester tester) async {
    final snapshot = (await tester.runAsync(editor.snapshot))!;
    return snapshot.text;
  }

  /// Lets the core's answers (colours, saves) come in and be drawn.
  Future<void> settle(WidgetTester tester) async {
    for (var i = 0; i < 3; i++) {
      await tester.runAsync(
        () => Future<void>.delayed(const Duration(milliseconds: 30)),
      );
      await tester.pump();
    }
  }

  /// Types [value] as the platform's input method would.
  Future<void> type(WidgetTester tester, TextEditingValue value) async {
    tester.testTextInput.updateEditingValue(value);
    await tester.pump();
  }
}

/// Opens a note with [text] in a library of its own and draws the editor on
/// it in a scroll view, focused.
Future<EditorHarness> pumpEditor(
  WidgetTester tester, {
  required String text,
  String title = 'A note',
  SpellCheckService? spellCheckService,
  TargetPlatform platform = TargetPlatform.iOS,
  Size size = const Size(393, 600),
  String themeId = 'kanso-light',
  int? caret,
  int windowUnits = WindowSizes.cap,
  int windowMargin = WindowSizes.margin,
}) async {
  tester.view
    ..physicalSize = size
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final made = (await tester.runAsync(() async {
    await initCoreForTests();
    final library = await openTestLibrary();
    final saved = await library.create(
      note: NewNote(title: title, body: text),
    );
    final editor = await library.edit(id: saved.id);
    final snapshot = await editor.snapshot();
    return (library, saved.id, editor, snapshot);
  }))!;
  final (library, noteId, editor, snapshot) = made;
  final document = RustEditorDocument(editor);
  final controller = KansoMarkdownController(
    document: document,
    snapshot: snapshot,
    themeId: themeId,
    caret: caret,
    windowUnits: windowUnits,
    windowMargin: windowMargin,
  );
  final focusNode = FocusNode();
  addTearDown(() {
    focusNode.dispose();
    controller.dispose();
  });
  await tester.pumpWidget(
    MaterialApp(
      theme: KansoTheme.light.copyWith(platform: platform),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(
        body: SingleChildScrollView(
          child: MarkdownEditor(
            controller: controller,
            focusNode: focusNode,
            spellCheckService: spellCheckService,
          ),
        ),
      ),
    ),
  );
  focusNode.requestFocus();
  await tester.pump();
  return EditorHarness(
    library: library,
    noteId: noteId,
    editor: editor,
    document: document,
    controller: controller,
    focusNode: focusNode,
  );
}

/// The editor on a [FakeEditorDocument], for what the real core does not do
/// on demand. Returns the controller; [onContext], [onLocked], [onFailure]
/// and [onEdited] are the controller's.
Future<KansoMarkdownController> pumpFakeEditor(
  WidgetTester tester,
  FakeEditorDocument document, {
  void Function(EditorContext context)? onContext,
  VoidCallback? onLocked,
  void Function(ApplicationError error)? onFailure,
  VoidCallback? onEdited,
  SpellCheckService? spellCheckService,
}) async {
  tester.view
    ..physicalSize = const Size(393, 600)
    ..devicePixelRatio = 1;
  addTearDown(tester.view.reset);
  final controller = KansoMarkdownController(
    document: document,
    snapshot: document.current,
    themeId: 'kanso-light',
    onContext: onContext,
    onLocked: onLocked,
    onFailure: onFailure,
    onEdited: onEdited,
  );
  final focusNode = FocusNode();
  addTearDown(() {
    focusNode.dispose();
    controller.dispose();
  });
  await tester.pumpWidget(
    MaterialApp(
      theme: KansoTheme.light,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: Scaffold(
        body: SingleChildScrollView(
          child: MarkdownEditor(
            controller: controller,
            focusNode: focusNode,
            spellCheckService: spellCheckService,
          ),
        ),
      ),
    ),
  );
  focusNode.requestFocus();
  await tester.pump();
  return controller;
}

// Kanso — editor_goldens_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart' as core;
import 'package:kanso/editor/state/editor/editor_bloc.dart';
import 'package:kanso/editor/views/controllers/kanso_markdown_controller.dart';
import 'package:kanso/editor/views/widgets/kanso_editable_text.dart';
import 'package:kanso/editor/views/widgets/markdown_editor.dart';
import 'package:kanso/editor/views/widgets/properties_row.dart';
import 'package:kanso/editor/views/widgets/property_row.dart';

import '../../../helpers/editor_fixture.dart';
import '../../../helpers/editor_golden_helpers.dart';
import '../../../helpers/fake_editor_document.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/kanso_viewport.dart';

/// The windows the editor is pictured in: a phone, and the three tablets the
/// issue asks for in both orientations (an iPad, and an Android tablet).
const List<KansoViewport> _viewports = <KansoViewport>[
  KansoViewport.phone,
  KansoViewport.iPadPortrait,
  KansoViewport.iPadLandscape,
  KansoViewport.androidTablet,
  KansoViewport.androidTabletPortrait,
];

/// How tall the keyboard is drawn in [viewport]: a phone's, and a tablet's
/// larger keys, a little less on an Android tablet.
double keyboardOf(KansoViewport viewport) => switch (viewport) {
  KansoViewport.phone => phoneKeyboard,
  KansoViewport.iPadPortrait => 360,
  KansoViewport.iPadLandscape => 410,
  KansoViewport.androidTablet => 330,
  _ => 340,
};

void main() {
  setUp(() => EditableText.debugDeterministicCursor = true);
  tearDown(() => EditableText.debugDeterministicCursor = false);

  for (final brightness in Brightness.values) {
    final name = brightness.name;

    for (final viewport in _viewports) {
      group('on ${viewport.slug}, $name', () {
        testWidgets('a note being edited', (tester) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            fixture.inViewport(viewport, brightness),
            name: '${viewport.slug}_edit_$name',
            viewport: viewport,
            brightness: brightness,
            interact: fixture.settle,
          );
        });

        testWidgets('with the keyboard up, and the bar over it', (
          tester,
        ) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            withKeyboard(
              fixture.inViewport(viewport, brightness),
              keyboardOf(viewport),
            ),
            name: '${viewport.slug}_keyboard_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              await fixture.settle(tester);
              await keyboardUp(tester, keyboardOf(viewport));
            },
          );
        });

        testWidgets('a link to a note being typed', (tester) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
            body: bodyEndingWith('See also [[Sy'),
          );
          await expectGoldenOnBothPlatforms(
            tester,
            withKeyboard(
              fixture.inViewport(viewport, brightness),
              keyboardOf(viewport),
            ),
            name: '${viewport.slug}_wiki_completion_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              await fixture.settle(tester);
              await keyboardUp(tester, keyboardOf(viewport));
              await fixture.settle(tester);
            },
          );
        });

        testWidgets('a tag being typed', (tester) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
            body: bodyEndingWith('Filed under #to'),
          );
          await expectGoldenOnBothPlatforms(
            tester,
            withKeyboard(
              fixture.inViewport(viewport, brightness),
              keyboardOf(viewport),
            ),
            name: '${viewport.slug}_tag_completion_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              await fixture.settle(tester);
              await keyboardUp(tester, keyboardOf(viewport));
              await fixture.settle(tester);
            },
          );
        });

        testWidgets('the properties open', (tester) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            fixture.inViewport(viewport, brightness),
            name: '${viewport.slug}_properties_open_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              await fixture.settle(tester);
              await tester.tap(find.byType(PropertiesRow));
              await fixture.settle(tester);
            },
          );
        });

        testWidgets('a property being edited', (tester) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            withKeyboard(
              fixture.inViewport(viewport, brightness),
              keyboardOf(viewport),
            ),
            name: '${viewport.slug}_properties_editing_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              await fixture.settle(tester);
              await tester.tap(find.byType(PropertiesRow));
              await fixture.settle(tester);
              await tester.tap(
                find.descendant(
                  of: find.byType(PropertyRow).first,
                  matching: find.byType(EditableText),
                ),
              );
              await keyboardUp(tester, keyboardOf(viewport));
              await fixture.settle(tester);
            },
          );
        });

        testWidgets('a change made elsewhere', (tester) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            fixture.inViewport(viewport, brightness),
            name: '${viewport.slug}_conflict_changed_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              await fixture.settle(tester);
              editorBloc(tester).add(
                const EditorEvent.signalReceived(
                  core.EditorEvent.incoming(deleted: false),
                ),
              );
              await fixture.settle(tester);
            },
          );
        });

        testWidgets('a note deleted elsewhere', (tester) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            fixture.inViewport(viewport, brightness),
            name: '${viewport.slug}_conflict_deleted_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              await fixture.settle(tester);
              editorBloc(tester).add(
                const EditorEvent.signalReceived(
                  core.EditorEvent.incoming(deleted: true),
                ),
              );
              await fixture.settle(tester);
            },
          );
        });

        testWidgets('a save that failed', (tester) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            fixture.inViewport(viewport, brightness),
            name: '${viewport.slug}_save_failed_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              await fixture.settle(tester);
              editorBloc(tester).add(
                const EditorEvent.signalReceived(
                  core.EditorEvent.failed(message: 'disk'),
                ),
              );
              await fixture.settle(tester);
            },
          );
        });

        testWidgets('text selected, with its handles and menu', (tester) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            withKeyboard(
              fixture.inViewport(viewport, brightness),
              keyboardOf(viewport),
            ),
            name: '${viewport.slug}_selection_menu_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              await fixture.settle(tester);
              await keyboardUp(tester, keyboardOf(viewport));
              final state = tester.state<EditableTextState>(
                find.byType(KansoEditableText),
              );
              final text = state.textEditingValue.text;
              final start = text.indexOf('conflict');
              state.userUpdateTextEditingValue(
                state.textEditingValue.copyWith(
                  selection: TextSelection(
                    baseOffset: start,
                    extentOffset: start + 'conflict'.length,
                  ),
                ),
                SelectionChangedCause.longPress,
              );
              await tester.pump();
              state.selectionOverlay
                ?..handlesVisible = true
                ..showHandles();
              state.showToolbar();
              await tester.pump(const Duration(milliseconds: 400));
              await tester.pump(const Duration(milliseconds: 400));
            },
          );
        });

        testWidgets('a misspelled word and its menu', (tester) async {
          final document = FakeEditorDocument(
            text: 'Notes on the sync design.',
          );
          final controller = KansoMarkdownController(
            document: document,
            snapshot: document.current,
            themeId: 'kanso-$name',
          );
          final focus = FocusNode();
          addTearDown(() {
            focus.dispose();
            controller.dispose();
          });
          await expectGoldenOnBothPlatforms(
            tester,
            Scaffold(
              body: inPanes(
                viewport,
                brightness,
                SafeArea(
                  child: Padding(
                    padding: const EdgeInsets.all(16),
                    child: MarkdownEditor(
                      controller: controller,
                      focusNode: focus,
                      spellCheckService: TehSpellCheckService(),
                    ),
                  ),
                ),
              ),
            ),
            name: '${viewport.slug}_spell_menu_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              // Each platform starts from the same text and no focus.
              focus.unfocus();
              controller.value = const TextEditingValue(
                text: 'Notes on the sync design.',
              );
              await tester.pump();
              focus.requestFocus();
              await tester.pump();
              tester.testTextInput.updateEditingValue(
                const TextEditingValue(
                  text: 'Notes on teh sync design, and `teh` in code.',
                  selection: TextSelection.collapsed(offset: 10),
                ),
              );
              await tester.pump(const Duration(milliseconds: 50));
              await tester.pump(const Duration(milliseconds: 50));
              tester
                  .state<EditableTextState>(find.byType(KansoEditableText))
                  .showSpellCheckSuggestionsToolbar();
              await tester.pump(const Duration(milliseconds: 400));
            },
          );
        });

        testWidgets('the info of the note', (tester) async {
          final fixture = await EditorFixture.open(
            tester,
            brightness: brightness,
          );
          await expectGoldenOnBothPlatforms(
            tester,
            fixture.inViewport(viewport, brightness),
            name: '${viewport.slug}_info_sheet_$name',
            viewport: viewport,
            brightness: brightness,
            interact: (tester) async {
              await fixture.settle(tester);
              await tester.tap(find.bySemanticsLabel('Note info'));
              await tester.pump(const Duration(milliseconds: 600));
            },
          );
        });
      });
    }
  }
}

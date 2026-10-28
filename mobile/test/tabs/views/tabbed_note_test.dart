import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/models/data/note_drag_data.dart';
import 'package:kanso/tabs/state/tabs/tabs_bloc.dart';
import 'package:kanso/tabs/views/widgets/tabbed_note.dart';

import '../../helpers/browse_blocs.dart';
import '../../helpers/library_fixtures.dart';
import '../../helpers/pump_routed.dart';

void main() {
  testWidgets('a note dropped on the first-tab strip opens in a tab', (
    tester,
  ) async {
    final blocs = await BrowseBlocs.start();
    blocs.library.notes.addAll(readableMockNotes());
    blocs.tabs.add(const TabsEvent.enabledChanged(enabled: true));
    await tester.pump();
    await pumpRouted(
      tester,
      blocs.provide(
        const Scaffold(
          body: TabbedNote(
            child: Center(
              child: Draggable<NoteDragData>(
                data: NoteDragData(
                  noteIds: <String>[MockIds.weeklyReview],
                  titles: <String>['Dragged note'],
                ),
                feedback: Material(child: Text('Dragged note')),
                child: Text('Drag note'),
              ),
            ),
          ),
        ),
      ),
    );

    final gesture = await tester.startGesture(
      tester.getCenter(find.text('Drag note')),
    );
    await gesture.moveTo(
      tester.getCenter(find.byType(DragTarget<NoteDragData>)),
    );
    await tester.pump();
    expect(find.text('Open in New Tab'), findsOneWidget);
    await gesture.up();
    await tester.pumpAndSettle();
    await tester.pump();

    expect(blocs.tabs.state.tabs.metas.map((tab) => tab.id), <String>[
      MockIds.weeklyReview,
    ]);
  });
}

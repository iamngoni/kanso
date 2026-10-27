// Kanso — library_pane_test.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:flutter/material.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:kanso/core/bridge/api/types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/core/models/data/browse_location.dart';
import 'package:kanso/core/models/data/library_index_status.dart';
import 'package:kanso/core/models/data/load_status.dart';
import 'package:kanso/core/state/library_index/library_index_bloc.dart';
import 'package:kanso/core/views/widgets/kanso_badge.dart';
import 'package:kanso/core/views/widgets/kanso_drawer.dart';
import 'package:kanso/core/views/widgets/kanso_icon_button.dart';
import 'package:kanso/core/views/widgets/kanso_sidebar_row.dart';
import 'package:kanso/library/views/widgets/library_pane.dart';
import 'package:kanso/sharing/state/invitations/invitations_bloc.dart';

import '../../../helpers/close_bloc_later.dart';
import '../../../helpers/fixture_library_repository.dart';
import '../../../helpers/fixture_shared_notes_repository.dart';
import '../../../helpers/golden.dart';
import '../../../helpers/library_fixtures.dart';
import '../../../helpers/pump_routed.dart';
import '../../../helpers/register_page_blocs.dart';
import '../../../helpers/sharing_fixtures.dart';

Future<LibraryIndexBloc> _readyIndex(
  WidgetTester tester,
  FixtureLibraryRepository library,
) async {
  final bloc = LibraryIndexBloc(library)
    ..add(const LibraryIndexEvent.started());
  closeBlocLater(bloc);
  await bloc.stream.firstWhere(
    (state) => state.status != LibraryIndexStatus.loading,
  );
  return bloc;
}

Widget _pane(
  LibraryIndexBloc bloc, {
  BrowseLocation? location,
  InvitationsBloc? invitations,
}) {
  final shared = invitations ?? InvitationsBloc(FixtureSharedNotesRepository());
  if (invitations == null) {
    closeBlocLater(shared);
  }
  return MultiBlocProvider(
    providers: <BlocProvider<dynamic>>[
      BlocProvider<LibraryIndexBloc>.value(value: bloc),
      BlocProvider<InvitationsBloc>.value(value: shared),
    ],
    child: LibraryPane(
      location: location ?? const BrowseLocation.place(ListPlace.all()),
    ),
  );
}

Map<String, bool> _selection(WidgetTester tester) => <String, bool>{
  for (final row in tester.widgetList<KansoSidebarRow>(
    find.byType(KansoSidebarRow),
  ))
    row.label: row.selected,
};

void main() {
  late FixtureLibraryRepository library;

  setUp(() {
    library = FixtureLibraryRepository();
    addTearDown(() => unawaited(library.close()));
    registerPageBlocs(library);
  });

  group('LibraryPane', () {
    testWidgets('the notebooks header has a button that asks for a name and '
        'goes to the new notebook', (tester) async {
      final bloc = await _readyIndex(tester, library);
      final router = await pumpRouted(tester, _pane(bloc));
      await tester.tap(
        find.byWidgetPredicate(
          (widget) =>
              widget is KansoIconButton &&
              widget.semanticLabel == 'New notebook',
        ),
      );
      await tester.pumpAndSettle();
      await tester.enterText(find.byType(EditableText), 'Recipes');
      await tester.pump();
      await tester.tap(find.text('Create'));
      await tester.pumpAndSettle();
      expect(library.notebooksCreated.single.$1, 'Recipes');
      expect(topOf(router), startsWith('/notebooks/'));
    });

    testWidgets('lists the places, notebooks and tags with their counts', (
      tester,
    ) async {
      final bloc = await _readyIndex(tester, library);
      await pumpRouted(tester, _pane(bloc));

      expect(find.text('All Notes'), findsOneWidget);
      expect(find.text('142'), findsOneWidget);
      expect(find.text('Pinned'), findsOneWidget);
      expect(find.text('3'), findsOneWidget);
      expect(find.text('Recent'), findsOneWidget);
      expect(find.text('Work'), findsOneWidget);
      expect(find.text('31'), findsOneWidget);
      expect(find.text('Tasks'), findsOneWidget);
      // The tags are below the fold of a phone's library.
      await tester.scrollUntilVisible(find.text('idea'), 120);
      expect(find.text('idea'), findsOneWidget);
      // So is the Trash, at the foot of the pane.
      await tester.scrollUntilVisible(
        find.text('Trash', skipOffstage: false),
        120,
      );
      expect(find.text('Trash'), findsOneWidget);
      expect(find.text('4'), findsOneWidget);
      // There is no Templates notebook in this library.
      expect(find.text('Templates'), findsNothing);
    });

    testWidgets('counts the overdue tasks on the Tasks row, or nothing', (
      tester,
    ) async {
      final bloc = await _readyIndex(tester, library);
      await pumpRouted(tester, _pane(bloc));
      expect(find.text('2'), findsOneWidget);
      expect(
        find.byWidgetPredicate(
          (widget) =>
              widget is Semantics &&
              widget.properties.value == '2 overdue tasks',
        ),
        findsOneWidget,
      );

      library.overdueValue = 0;
      final none = await _readyIndex(tester, library);
      await pumpRouted(tester, _pane(none));
      expect(find.text('2'), findsNothing);
      expect(find.text('Tasks'), findsOneWidget);
    });

    testWidgets('opens the Tasks place, and selects its row there', (
      tester,
    ) async {
      final bloc = await _readyIndex(tester, library);
      final router = await pumpRouted(tester, _pane(bloc));
      expect(_selection(tester)['Tasks'], isFalse);
      await tester.tap(find.text('Tasks'));
      await tester.pumpAndSettle();
      expect(whereIs(router), const BrowseLocation.tasks().location);

      await pumpRouted(
        tester,
        _pane(bloc, location: const BrowseLocation.tasks()),
      );
      final selected = _selection(tester);
      expect(selected['Tasks'], isTrue);
      expect(selected['All Notes'], isFalse);
    });

    testWidgets('indents a notebook inside another', (tester) async {
      final bloc = await _readyIndex(tester, library);
      await pumpRouted(tester, _pane(bloc));
      final depths = <String, int>{
        for (final row in tester.widgetList<KansoSidebarRow>(
          find.byType(KansoSidebarRow),
        ))
          row.label: row.depth,
      };
      expect(depths['Projects'], 0);
      expect(depths['Kanso'], 1);
      expect(depths['Garden'], 1);
    });

    testWidgets('has a Templates row for the notebook with that role', (
      tester,
    ) async {
      library.notebookRows = mockNotebooksWithTemplates;
      final bloc = await _readyIndex(tester, library);
      final router = await pumpRouted(tester, _pane(bloc));
      expect(find.text('Templates'), findsOneWidget);
      // It is the notebook, listed once.
      expect(find.text('Templates'), findsOneWidget);
      await tester.tap(find.text('Templates'));
      await tester.pumpAndSettle();
      expect(
        whereIs(router),
        const BrowseLocation.place(
          ListPlace.notebook(id: MockIds.templates),
        ).location,
      );
    });

    testWidgets('selects the row of the place being listed', (tester) async {
      final bloc = await _readyIndex(tester, library);
      await pumpRouted(
        tester,
        _pane(
          bloc,
          location: const BrowseLocation.place(
            ListPlace.notebook(id: MockIds.garden),
          ),
        ),
      );
      final selected = _selection(tester);
      expect(selected['Garden'], isTrue);
      expect(selected['All Notes'], isFalse);
      expect(
        selected.entries.where((entry) => entry.value).length,
        1,
      );
    });

    testWidgets('selects a tag, and Trash', (tester) async {
      final bloc = await _readyIndex(tester, library);
      await pumpRouted(
        tester,
        _pane(
          bloc,
          location: const BrowseLocation.place(
            ListPlace.tag(id: MockIds.tagIdea),
          ),
        ),
        size: const Size(393, 1400),
      );
      // A tag is a chip, not a row: its selection is told to a screen
      // reader rather than drawn as a sidebar row.
      expect(
        tester
            .widgetList<Semantics>(find.byType(Semantics))
            .any(
              (widget) =>
                  (widget.properties.selected ?? false) &&
                  widget.properties.label == '#idea',
            ),
        isTrue,
      );
      await tester.pumpWidget(const SizedBox.shrink());
      await pumpRouted(
        tester,
        _pane(bloc, location: const BrowseLocation.place(ListPlace.trash())),
        size: const Size(393, 1400),
      );
      expect(_selection(tester)['Trash'], isTrue);
    });

    testWidgets('a row opens its place', (tester) async {
      final bloc = await _readyIndex(tester, library);
      final router = await pumpRouted(tester, _pane(bloc));
      await tester.tap(find.text('Work'));
      await tester.pumpAndSettle();
      expect(
        whereIs(router),
        '/notebooks/${Uri.encodeComponent(MockIds.work)}',
      );
    });

    testWidgets('a tag opens its notes', (tester) async {
      final bloc = await _readyIndex(tester, library);
      final router = await pumpRouted(
        tester,
        _pane(bloc),
        size: const Size(393, 1400),
      );
      await tester.tap(find.text('todo'));
      await tester.pumpAndSettle();
      expect(whereIs(router), '/tags/${Uri.encodeComponent(MockIds.tagTodo)}');
    });

    testWidgets('has Shared with you after Recent, with no count while '
        'nothing waits', (tester) async {
      final bloc = await _readyIndex(tester, library);
      await pumpRouted(tester, _pane(bloc), size: const Size(393, 1400));
      final labels = tester
          .widgetList<KansoSidebarRow>(find.byType(KansoSidebarRow))
          .map((row) => row.label)
          .toList();
      expect(labels.indexOf('Shared with you'), labels.indexOf('Recent') + 1);
      expect(find.byType(KansoBadge), findsNothing);
    });

    testWidgets('Shared with you shows how many invitations wait, and opens '
        'them after reading them again', (tester) async {
      final bloc = await _readyIndex(tester, library);
      final shared = FixtureSharedNotesRepository()
        ..invitationList = <InvitationRow>[
          invitationRow('s_1'),
          invitationRow('s_2'),
        ];
      final invitations = InvitationsBloc(shared)
        ..add(const InvitationsEvent.started());
      closeBlocLater(invitations);
      await invitations.stream.firstWhere(
        (state) => state.status != LoadStatus.loading,
      );
      final router = await pumpRouted(
        tester,
        _pane(bloc, invitations: invitations),
        size: const Size(393, 1400),
      );
      expect(
        tester.widget<KansoBadge>(find.byType(KansoBadge)).label,
        '2',
      );
      shared.invitationList = <InvitationRow>[invitationRow('s_1')];
      await tester.tap(find.text('Shared with you'));
      await tester.pumpAndSettle();
      expect(topOf(router), '/shared');
      expect(invitations.state.invitations.length, 1);
    });

    testWidgets('the search field opens search; settings opens settings', (
      tester,
    ) async {
      final bloc = await _readyIndex(tester, library);
      final router = await pumpRouted(tester, _pane(bloc));
      await tester.tap(find.text('Search notes'));
      await tester.pumpAndSettle();
      expect(whereIs(router), '/search');
      router.go('/start');
      await tester.pumpAndSettle();
      final handle = tester.ensureSemantics();
      await tester.tap(find.bySemanticsLabel('Settings'));
      await tester.pumpAndSettle();
      expect(topOf(router), '/settings');
      handle.dispose();
    });

    testWidgets('says so when there are no notebooks or tags', (tester) async {
      library
        ..notebookRows = const <NotebookRow>[]
        ..tagRows = const <TagRow>[];
      final bloc = await _readyIndex(tester, library);
      await pumpRouted(tester, _pane(bloc));
      expect(find.text('No notebooks yet'), findsOneWidget);
      expect(find.text('No tags yet'), findsOneWidget);
    });

    testWidgets('shows still bars while it loads, and says so', (
      tester,
    ) async {
      final handle = tester.ensureSemantics();
      final bloc = LibraryIndexBloc(library);
      closeBlocLater(bloc);
      await pumpRouted(tester, _pane(bloc));
      expect(bloc.state.status, LibraryIndexStatus.loading);
      expect(find.bySemanticsLabel('Loading the library'), findsOneWidget);
      expect(find.text('All Notes'), findsNothing);
      handle.dispose();
    });

    testWidgets('offers to try again after a failure', (tester) async {
      final failing = _FailingLibrary();
      addTearDown(() => unawaited(failing.close()));
      final bloc = LibraryIndexBloc(failing)
        ..add(const LibraryIndexEvent.started());
      closeBlocLater(bloc);
      await bloc.stream.firstWhere(
        (state) => state.status == LibraryIndexStatus.failed,
      );
      await pumpRouted(tester, _pane(bloc));
      expect(find.text("The library didn't load"), findsOneWidget);
      expect(find.text('All Notes'), findsNothing);

      failing.failing = false;
      await tester.tap(find.text('Try again'));
      await tester.pump();
      await tester.pump();
      await tester.pumpAndSettle();
      expect(find.text('All Notes'), findsOneWidget);
      expect(find.text("The library didn't load"), findsNothing);
    });

    testWidgets('follows the library as it changes', (tester) async {
      final bloc = await _readyIndex(tester, library);
      await pumpRouted(tester, _pane(bloc));
      expect(find.text('142'), findsOneWidget);
      library.overviewValue = const LibraryOverview(
        all: 150,
        pinned: 3,
        recent: 11,
        trash: 4, byStatus: <StatusCount>[],
      );
      library.changeStream.add(<String>['note:new']);
      await tester.pump(const Duration(milliseconds: 150));
      await tester.pumpAndSettle();
      expect(find.text('150'), findsOneWidget);
    });
  });

  group('LibraryPane in a drawer', () {
    testWidgets('closes the drawer as a row is chosen', (tester) async {
      final bloc = await _readyIndex(tester, library);
      final router = await pumpRouted(
        tester,
        _pane(bloc),
        wrap: (page) => KansoDrawer(
          width: 334,
          drawer: page,
          child: const SizedBox.expand(),
        ),
      );
      final drawer = tester.state<KansoDrawerState>(find.byType(KansoDrawer))
        ..open();
      await tester.pumpAndSettle();
      expect(drawer.isOpen, isTrue);
      await tester.tap(find.text('Pinned'));
      await tester.pumpAndSettle();
      expect(whereIs(router), '/pinned');
    });
  });

  group('LibraryPane pictures', () {
    for (final brightness in Brightness.values) {
      testWidgets('the library, ${brightness.name}', (tester) async {
        final bloc = await _readyIndex(tester, library);
        await expectGoldenOnBothPlatforms(
          tester,
          Scaffold(
            body: Align(
              alignment: AlignmentDirectional.centerStart,
              child: SizedBox(
                width: phoneSize.width * 0.85,
                child: _pane(bloc),
              ),
            ),
          ),
          name: 'phone_library_${brightness.name}',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('the library with a Templates notebook, ${brightness.name}', (
        tester,
      ) async {
        library.notebookRows = mockNotebooksWithTemplates;
        final bloc = await _readyIndex(tester, library);
        await expectGoldenOnBothPlatforms(
          tester,
          Scaffold(
            body: Align(
              alignment: AlignmentDirectional.centerStart,
              child: SizedBox(
                width: phoneSize.width * 0.85,
                child: _pane(
                  bloc,
                  location: const BrowseLocation.place(
                    ListPlace.notebook(id: MockIds.kanso),
                  ),
                ),
              ),
            ),
          ),
          name: 'phone_library_templates_${brightness.name}',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('the library while it loads, ${brightness.name}', (
        tester,
      ) async {
        final bloc = LibraryIndexBloc(library);
        closeBlocLater(bloc);
        await expectGoldenOnBothPlatforms(
          tester,
          Scaffold(
            body: Align(
              alignment: AlignmentDirectional.centerStart,
              child: SizedBox(
                width: phoneSize.width * 0.85,
                child: _pane(bloc),
              ),
            ),
          ),
          name: 'phone_library_loading_${brightness.name}',
          size: phoneSize,
          brightness: brightness,
        );
      });

      testWidgets('the library after a failure, ${brightness.name}', (
        tester,
      ) async {
        final failing = _FailingLibrary();
        addTearDown(() => unawaited(failing.close()));
        final bloc = LibraryIndexBloc(failing)
          ..add(const LibraryIndexEvent.started());
        closeBlocLater(bloc);
        await bloc.stream.firstWhere(
          (state) => state.status == LibraryIndexStatus.failed,
        );
        await expectGoldenOnBothPlatforms(
          tester,
          Scaffold(
            body: Align(
              alignment: AlignmentDirectional.centerStart,
              child: SizedBox(
                width: phoneSize.width * 0.85,
                child: _pane(bloc),
              ),
            ),
          ),
          name: 'phone_library_failed_${brightness.name}',
          size: phoneSize,
          brightness: brightness,
        );
      });
    }
  });
}

class _FailingLibrary extends FixtureLibraryRepository {
  bool failing = true;

  @override
  Future<Either<ApplicationError, List<NotebookRow>>> notebooks() async =>
      failing
      ? const Left<ApplicationError, List<NotebookRow>>(
          ApplicationError.storage(message: 'Disk error.'),
        )
      : super.notebooks();
}

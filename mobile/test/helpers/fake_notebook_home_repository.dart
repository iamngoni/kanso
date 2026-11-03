// Kanso — fake_notebook_home_repository.dart
// Created 2026-11-03.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:dartz/dartz.dart';
import 'package:kanso/core/bridge/api/notebook_home_types.dart';
import 'package:kanso/core/bridge/api/people_types.dart';
import 'package:kanso/core/models/data/application_error.dart';
import 'package:kanso/notebook_home/models/repos/abs/notebook_home_repository.dart';

/// The words the core says, as the tests use them.
const HomeWords fakeHomeWords = HomeWords(
  aiOff: 'AI is off',
  locked: 'This notebook is locked',
  firstVisitButton: 'Write this summary',
  noOpenTasks: 'No open tasks',
  homeRow: 'Home',
  askButton: 'Ask about this notebook',
  openHome: 'Open home',
  addHomeNote: 'Add a Home note',
  homeMoved: 'It is no longer the Home note',
  sectionKeys: <String>['about', 'open_tasks'],
);

/// A card that shows nothing: what a notebook has with homepages off.
const HomeCard hiddenHomeCard = HomeCard(
  show_: false,
  name: '',
  aboutLine: '',
  hasHomeNote: false,
  opensDirectly: false,
);

/// A section for a test.
PageSection homeSection(
  String key,
  String title, {
  bool ai = false,
  List<PageItem> items = const <PageItem>[],
  PageSectionState state = PageSectionState.fresh,
  int? total,
  String note = '',
  String emptyLine = '',
  String staleLine = '',
}) => PageSection(
  key: key,
  title: title,
  ai: ai,
  items: items,
  state: state,
  changed: 0,
  writtenMs: 0,
  total: total ?? items.length,
  note: note,
  emptyLine: emptyLine,
  staleLine: staleLine,
);

/// An item for a test.
PageItem homeItem(
  String text, {
  String target = '',
  String tag = '',
  String meta = '',
  String group = '',
  PageTask? task,
  List<PageSource> sources = const <PageSource>[],
}) => PageItem(
  text: text,
  who: '',
  sources: sources,
  meta: meta,
  group: group,
  task: task,
  target: target,
  tag: tag,
  shared: false,
);

/// A panel for a test.
HomePanel homePanel(
  List<PageSection> sections, {
  String name = 'Work',
  String line = '4 notes',
  bool aiOn = false,
  List<String> due = const <String>[],
  String footer = '',
  bool locked = false,
  bool lockedOpen = false,
  List<PageHidden> hidden = const <PageHidden>[],
}) => HomePanel(
  header: HomeHeaderInfo(
    name: name,
    notes: 4,
    notebooksInside: 0,
    line: line,
    hasHomeNote: false,
    homeNoteId: '',
  ),
  sections: sections,
  hidden: hidden,
  aiOn: aiOn,
  due: due,
  footer: footer,
  locked: locked,
  lockedOpen: lockedOpen,
);

/// A [NotebookHomeRepository] that answers from what a test sets and
/// remembers what it was asked. By default there is no card and no page.
class FakeNotebookHomeRepository implements NotebookHomeRepository {
  FakeNotebookHomeRepository({
    this.cardAnswer = hiddenHomeCard,
    this.panelAnswer,
    this.words_ = fakeHomeWords,
    this.mapAnswer,
  });

  HomeCard cardAnswer;
  HomePanel? panelAnswer;
  HomeWords words_;
  HomeMap? mapAnswer;
  HomeSettings settingsAnswer = const HomeSettings(
    showHomepages: true,
    summariesWithAi: true,
    refresh: 'auto',
  );
  List<HomeCandidate> candidatesAnswer = const <HomeCandidate>[];
  List<HomeSectionSetting> sectionsAnswer = const <HomeSectionSetting>[];
  String newHomeId = 'home-note';
  bool afterMoveAnswer = false;

  /// What was asked, in order: `open`, `pin:key`, `refresh:key`...
  final List<String> calls = <String>[];

  /// Holds [words] back until completed, for the ordering tests.
  Completer<void>? wordsGate;

  /// Holds [open] back until completed.
  Completer<void>? openGate;

  /// Holds [refreshSection] back until completed.
  Completer<void>? refreshGate;

  /// What the layout was set to by [customise].
  List<String>? customisedOrder;
  List<String>? customisedHidden;
  PageTask? ticked;
  bool? tickedTo;

  Either<ApplicationError, T> _ok<T>(T value) =>
      Right<ApplicationError, T>(value);

  @override
  Future<Either<ApplicationError, HomeSettings>> settings() async =>
      _ok(settingsAnswer);

  @override
  Future<Either<ApplicationError, HomeSettings>> setSettings(
    HomeSettings settings,
  ) async {
    settingsAnswer = settings;
    return _ok(settings);
  }

  @override
  Future<Either<ApplicationError, HomeWords>> words() async {
    calls.add('words');
    await wordsGate?.future;
    return _ok(words_);
  }

  @override
  Future<Either<ApplicationError, HomePanel?>> open(String bookId) async {
    calls.add('open');
    await openGate?.future;
    return _ok(panelAnswer);
  }

  @override
  Future<Either<ApplicationError, HomeMap>> map(String bookId) async {
    calls.add('map');
    return _ok(
      mapAnswer ??
          const HomeMap(
            width: 320,
            height: 200,
            nodes: <HomeMapNode>[],
            edges: <HomeMapEdge>[],
          ),
    );
  }

  @override
  Future<Either<ApplicationError, HomeCard>> card(String bookId) async {
    calls.add('card');
    return _ok(cardAnswer);
  }

  @override
  Future<Either<ApplicationError, HomePanel?>> showAll(
    String bookId,
    String key,
  ) async {
    calls.add('showAll:$key');
    return _ok(panelAnswer);
  }

  @override
  Future<Either<ApplicationError, HomePanel?>> refreshSection(
    String bookId,
    String key,
  ) async {
    calls.add('refresh:$key');
    await refreshGate?.future;
    return _ok(panelAnswer);
  }

  @override
  Future<Either<ApplicationError, HomePanel?>> refreshAll(
    String bookId,
  ) async {
    calls.add('refreshAll');
    return _ok(panelAnswer);
  }

  @override
  Future<Either<ApplicationError, HomePanel?>> pin(
    String bookId,
    String key, {
    required bool on,
  }) async {
    calls.add('pin:$key:$on');
    return _ok(panelAnswer);
  }

  @override
  Future<Either<ApplicationError, HomePanel?>> hide(
    String bookId,
    String key, {
    required bool hidden,
  }) async {
    calls.add('hide:$key:$hidden');
    return _ok(panelAnswer);
  }

  @override
  Future<Either<ApplicationError, HomePanel?>> showHidden(
    String bookId,
  ) async {
    calls.add('showHidden');
    return _ok(panelAnswer);
  }

  @override
  Future<Either<ApplicationError, HomePanel?>> notRight(
    String bookId,
    String key,
    int index,
  ) async {
    calls.add('notRight:$key:$index');
    return _ok(panelAnswer);
  }

  @override
  Future<Either<ApplicationError, String>> copyIntoHome(
    String bookId,
    String key,
  ) async {
    calls.add('copy:$key');
    return _ok(newHomeId);
  }

  @override
  Future<Either<ApplicationError, List<HomeCandidate>>> candidates(
    String bookId,
  ) async => _ok(candidatesAnswer);

  @override
  Future<Either<ApplicationError, Unit>> chooseHome(
    String bookId,
    String noteId,
  ) async {
    calls.add('chooseHome:$noteId');
    return _ok(unit);
  }

  @override
  Future<Either<ApplicationError, String>> newHome(String bookId) async {
    calls.add('newHome');
    return _ok(newHomeId);
  }

  @override
  Future<Either<ApplicationError, List<HomeSectionSetting>>> sections(
    String bookId,
  ) async => _ok(sectionsAnswer);

  @override
  Future<Either<ApplicationError, HomePanel?>> customise(
    String bookId,
    List<String> order,
    List<String> hidden,
  ) async {
    calls.add('customise');
    customisedOrder = order;
    customisedHidden = hidden;
    return _ok(panelAnswer);
  }

  @override
  Future<Either<ApplicationError, HomePanel?>> resetLayout(
    String bookId,
  ) async {
    calls.add('resetLayout');
    return _ok(panelAnswer);
  }

  @override
  Future<Either<ApplicationError, Unit>> setTask(
    PageTask task, {
    required bool checked,
  }) async {
    calls.add('setTask');
    ticked = task;
    tickedTo = checked;
    return _ok(unit);
  }

  @override
  Future<Either<ApplicationError, bool>> afterMove(String noteId) async =>
      _ok(afterMoveAnswer);
}

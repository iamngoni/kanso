// Kanso — library_fixtures.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:kanso/core/bridge/api/types.dart';
import 'tag_color.dart';

/// A note for tests, in the notebook [notebookId].
NoteView noteView(String id, {String notebookId = 'b_1'}) {
  return NoteView(
    id: id,
    rev: '1-$id',
    title: 'Note $id',
    body: 'The text of $id.',
    lock: const LockState.plain(),
    notebookId: notebookId,
    tags: const <String>[],
    status: 'active',
    pinned: false,
    createdMs: 1700000000000,
    updatedMs: 1700000000000,
  );
}

/// A notebook for tests.
NotebookRow notebookRow(
  String id, {
  String? name,
  int count = 0,
  int depth = 0,
  String? parentId,
  NotebookRole? role,
  bool locked = false,
}) {
  return NotebookRow(
    id: id,
    name: name ?? 'Notebook $id',
    parentId: parentId,
    count: count,
    locked: locked,
    role: role,
    depth: depth, hasChildren: false,
  );
}

/// A tag for tests.
TagRow tagRow(String id, {String? name, int count = 1}) {
  return TagRow(id: id, name: name ?? id, color: 0x2185D0, count: count);
}

// ---------------------------------------------------------------------------
// The content of the approved mock screens, in the library's own types.
//
// The real screens fed with this should look like the mocks. The clock of
// the content is Monday 26 October 2026, 10:30, local time.
// ---------------------------------------------------------------------------

/// "Now" for the fixtures' content: Monday 26 October 2026, 10:30.
final DateTime mockNow = DateTime(2026, 10, 26, 10, 30);

/// The local instant, in milliseconds, of [month]/[day] 2026 at
/// [hour]:[minute].
int mockMs(int month, int day, int hour, int minute) =>
    DateTime(2026, month, day, hour, minute).millisecondsSinceEpoch;

/// Notebook ids of the mock library.
abstract final class MockIds {
  static const String inbox = 'book:inbox';
  static const String journal = 'book:journal';
  static const String projects = 'book:projects';
  static const String kanso = 'book:kanso';
  static const String garden = 'book:garden';
  static const String reading = 'book:reading';
  static const String recipes = 'book:recipes';
  static const String someday = 'book:someday';
  static const String travel = 'book:travel';
  static const String work = 'book:work';
  static const String templates = 'book:templates';

  static const String tagIdea = 'tag:idea';
  static const String tagDraft = 'tag:draft';
  static const String tagReading = 'tag:reading';
  static const String tagTodo = 'tag:todo';

  static const String weeklyReview = 'note:weekly';
  static const String syncDesign = 'note:sync';
  static const String sourdough = 'note:sourdough';
  static const String lisbon = 'note:lisbon';
  static const String readingList = 'note:readinglist';
  static const String gardenBeds = 'note:gardenbeds';
  static const String privateJournal = 'note:private';
  static const String quarterlyGoals = 'note:goals';
  static const String slowSoftware = 'note:slow';
  static const String packingList = 'note:packing';
}

/// The mock library's notebooks in tree order, as the design shows them
/// (the Templates notebook is added by [mockNotebooksWithTemplates]).
final List<NotebookRow> mockNotebooks = <NotebookRow>[
  notebookRow(
    MockIds.inbox,
    name: 'Inbox',
    count: 12,
    role: NotebookRole.inbox,
  ),
  notebookRow(
    MockIds.journal,
    name: 'Journal',
    count: 48,
    role: NotebookRole.journal,
  ),
  notebookRow(MockIds.projects, name: 'Projects', count: 23),
  notebookRow(
    MockIds.kanso,
    name: 'Kanso',
    count: 17,
    depth: 1,
    parentId: MockIds.projects,
  ),
  notebookRow(
    MockIds.garden,
    name: 'Garden',
    count: 6,
    depth: 1,
    parentId: MockIds.projects,
  ),
  notebookRow(MockIds.reading, name: 'Reading', count: 9),
  notebookRow(MockIds.recipes, name: 'Recipes', count: 14),
  notebookRow(MockIds.someday, name: 'Someday'),
  notebookRow(MockIds.travel, name: 'Travel', count: 5),
  notebookRow(MockIds.work, name: 'Work', count: 31),
];

/// [mockNotebooks] with a Templates notebook at the end.
final List<NotebookRow> mockNotebooksWithTemplates = <NotebookRow>[
  ...mockNotebooks,
  notebookRow(
    MockIds.templates,
    name: 'Templates',
    count: 2,
    role: NotebookRole.templates,
  ),
];

/// The mock library's tags.
final List<TagRow> mockTags = <TagRow>[
  TagRow(
    id: MockIds.tagIdea,
    name: 'idea',
    color: TagColor.teal.rgb,
    count: 8,
  ),
  TagRow(
    id: MockIds.tagDraft,
    name: 'draft',
    color: TagColor.orange.rgb,
    count: 5,
  ),
  TagRow(
    id: MockIds.tagReading,
    name: 'reading',
    color: TagColor.violet.rgb,
    count: 11,
  ),
  TagRow(
    id: MockIds.tagTodo,
    name: 'todo',
    color: TagColor.pink.rgb,
    count: 7,
  ),
];

/// The counts of the mock library.
const LibraryOverview mockOverview = LibraryOverview(
  all: 142,
  pinned: 3,
  recent: 11,
  trash: 4, byStatus: <StatusCount>[],
);

/// A row of a list, with the facts the mock list shows.
NoteRow mockRow({
  required String id,
  required String title,
  required String snippet,
  required String notebookId,
  required int updatedMs,
  int? createdMs,
  String status = 'none',
  List<String> tags = const <String>[],
  bool pinned = false,
  bool locked = false,
}) {
  return NoteRow(
    id: id,
    rev: '1-$id',
    title: title,
    snippet: locked ? '' : snippet,
    notebookId: notebookId,
    status: locked ? 'none' : status,
    tags: locked ? const <String>[] : tags,
    pinned: pinned,
    createdMs: createdMs ?? updatedMs,
    updatedMs: updatedMs,
    tasksDone: 0,
    tasksTotal: 0,
    locked: locked,
  );
}

/// The notes of the mock list, in list order: pinned first, then newest
/// first.
final List<NoteRow> mockRows = <NoteRow>[
  mockRow(
    id: MockIds.weeklyReview,
    title: 'Weekly review',
    snippet:
        'Finished the sync conflict screen and moved the Garden notes to '
        'next week. Thursday: call about the allotment, then plan the beds.',
    notebookId: MockIds.journal,
    updatedMs: mockMs(10, 19, 17, 20),
    tags: <String>[MockIds.tagTodo],
    pinned: true,
  ),
  mockRow(
    id: MockIds.syncDesign,
    title: 'Sync design notes',
    snippet:
        'Every device keeps a full copy. A change is encrypted before it '
        'leaves the device, sent once and merged on the others.',
    notebookId: MockIds.kanso,
    updatedMs: mockMs(10, 26, 9, 41),
    createdMs: mockMs(10, 12, 8, 5),
    status: 'active',
    tags: <String>[MockIds.tagIdea, MockIds.tagDraft],
  ),
  mockRow(
    id: MockIds.sourdough,
    title: 'Sourdough starter',
    snippet:
        'Feed twice a day at room temperature, once a day in the fridge. '
        'It is ready when it doubles within six hours.',
    notebookId: MockIds.recipes,
    updatedMs: mockMs(10, 26, 8, 12),
  ),
  mockRow(
    id: MockIds.lisbon,
    title: 'Lisbon in spring',
    snippet:
        'Stay near the river, walk up to the miradouros in the morning. '
        'Book the train to Sintra a day ahead.',
    notebookId: MockIds.travel,
    updatedMs: mockMs(10, 25, 18, 30),
    status: 'onHold',
    tags: <String>[MockIds.tagIdea],
  ),
  mockRow(
    id: MockIds.readingList,
    title: 'Reading list 2026',
    snippet:
        'Slow software, a book on typefaces, the garden almanac. Finish '
        'one before starting the next.',
    notebookId: MockIds.reading,
    updatedMs: mockMs(10, 25, 11, 5),
    tags: <String>[MockIds.tagReading],
  ),
  mockRow(
    id: MockIds.gardenBeds,
    title: 'Garden beds plan',
    snippet:
        'Three raised beds along the south fence: beans, leafy greens and '
        'herbs. Order the soil before the weekend.',
    notebookId: MockIds.garden,
    updatedMs: mockMs(10, 22, 15, 10),
    status: 'completed',
    tags: <String>[MockIds.tagTodo],
  ),
  mockRow(
    id: MockIds.privateJournal,
    title: 'Private journal',
    snippet: 'The secret text of the private journal.',
    notebookId: MockIds.journal,
    updatedMs: mockMs(10, 21, 14, 20),
    createdMs: mockMs(1, 3, 21, 0),
    locked: true,
  ),
  mockRow(
    id: MockIds.quarterlyGoals,
    title: 'Quarterly goals',
    snippet:
        'Ship the mobile beta, write the sync chapter of the manual and '
        'take two proper weeks off.',
    notebookId: MockIds.work,
    updatedMs: mockMs(10, 21, 9, 0),
    status: 'dropped',
  ),
  mockRow(
    id: MockIds.slowSoftware,
    title: 'Notes on slow software',
    snippet:
        'Software that does less, tells you less and stays out of the way. '
        'Examples worth copying, and what to avoid.',
    notebookId: MockIds.inbox,
    updatedMs: mockMs(10, 12, 10, 0),
    tags: <String>[MockIds.tagReading, MockIds.tagIdea],
  ),
  mockRow(
    id: MockIds.packingList,
    title: 'Packing list',
    snippet:
        'Charger, notebook, the small camera, a light jacket. Leave the '
        'laptop at home.',
    notebookId: MockIds.travel,
    updatedMs: mockMs(10, 9, 19, 30),
    status: 'completed',
  ),
];

/// The shape the desktop gives a list sorted by date updated: headers by
/// day, with the pinned notes under their own.
const ListShape mockShape = ListShape(
  sort: SortKey.updated,
  byDate: true,
  date: DateField.updated,
  groupPinned: true,
);

/// The mock list as the library returns it for All Notes.
final NoteList mockNoteList = NoteList(rows: mockRows, shape: mockShape);

/// The Markdown of the open note of the mock note screen.
const String mockNoteBody = '''
# Goals

Every device keeps a full copy. A change is **encrypted on the device**, sent once and *merged* on the others. The first sketch is in `sync.rs`; the reasoning is in the [design write-up](https://kanso.app/design).

## Open questions

- How long a device may stay offline
- What a conflict looks like to a person

- [x] Write down the wire format
- [ ] Try a conflict on three devices
- [ ] Decide how deleted notes are kept

> Calm software says less and remembers more.

```rust
// last write wins, for now
fn merge(a: Note, b: Note) -> Note {
    if b.at > a.at { b } else { a }
}
```

Next: [[Weekly review]]
''';

/// The open note of the mock note screen.
final NoteView mockOpenNote = NoteView(
  id: MockIds.syncDesign,
  rev: '1-note:sync',
  title: 'Sync design notes',
  body: mockNoteBody,
  lock: const LockState.plain(),
  notebookId: MockIds.kanso,
  tags: <String>[MockIds.tagIdea, MockIds.tagDraft],
  status: 'active',
  pinned: false,
  createdMs: mockMs(10, 12, 8, 5),
  updatedMs: mockMs(10, 26, 9, 41),
);

/// The locked note of the mock screens, closed: its title and its lock.
final NoteView mockLockedNote = NoteView(
  id: MockIds.privateJournal,
  rev: '1-note:private',
  title: 'Private journal',
  lock: const LockState.closed(),
  notebookId: MockIds.journal,
  tags: const <String>[],
  status: 'none',
  pinned: false,
  createdMs: mockMs(1, 3, 21, 0),
  updatedMs: mockMs(10, 21, 14, 20),
);

/// The text only the locked note of the mock library holds: no screen may
/// show it while the note is closed.
const String mockSecret = 'The secret text of the private journal.';

/// The ranges of the case-insensitive occurrences of [query] in [text], in
/// UTF-16 units.
List<Range16> mockRanges(String text, String query) {
  final ranges = <Range16>[];
  if (query.isEmpty) {
    return ranges;
  }
  final haystack = text.toLowerCase();
  final needle = query.toLowerCase();
  var from = 0;
  while (true) {
    final at = haystack.indexOf(needle, from);
    if (at < 0) {
      return ranges;
    }
    ranges.add(Range16(start: at, end: at + needle.length));
    from = at + needle.length;
  }
}

/// A hit on [query] in [row], with the words marked in the title and the
/// snippet.
Hit mockHit(NoteRow row, String query) {
  return Hit(
    row: row,
    titleRanges: mockRanges(row.title, query),
    snippetRanges: mockRanges(row.snippet, query),
  );
}

/// The results of the mock search for "sync".
final List<Hit> mockHits = <Hit>[
  mockHit(
    mockRow(
      id: MockIds.syncDesign,
      title: 'Sync design notes',
      snippet:
          '...every device keeps a full copy. A change is encrypted before '
          'it is sent, and sync never reads a note.',
      notebookId: MockIds.kanso,
      updatedMs: mockMs(10, 26, 9, 41),
    ),
    'sync',
  ),
  mockHit(
    mockRow(
      id: MockIds.weeklyReview,
      title: 'Weekly review',
      snippet:
          '...finished the sync conflict screen before the Garden notes. '
          'Moved the sync call to Thursday.',
      notebookId: MockIds.journal,
      updatedMs: mockMs(10, 19, 17, 20),
    ),
    'sync',
  ),
  mockHit(
    mockRow(
      id: 'note:syncing',
      title: 'Syncing across devices',
      snippet:
          '...how apps that sync in the background hide the work from the '
          'person using them.',
      notebookId: MockIds.reading,
      updatedMs: mockMs(10, 14, 12, 0),
    ),
    'sync',
  ),
  mockHit(
    mockRow(
      id: 'note:wishlist',
      title: 'Sync wishlist',
      snippet:
          '...wake this device when notes change elsewhere, and sync again '
          'when the connection comes back.',
      notebookId: MockIds.inbox,
      updatedMs: mockMs(10, 13, 12, 0),
    ),
    'sync',
  ),
  mockHit(
    mockRow(
      id: MockIds.quarterlyGoals,
      title: 'Quarterly goals',
      snippet: '...ship the mobile beta with sync, then write the manual.',
      notebookId: MockIds.work,
      updatedMs: mockMs(10, 21, 9, 0),
    ),
    'sync',
  ),
];

/// A note of the mock library with what a test changes: its [title] and
/// [body], and whether it is [closed] (a locked note that has not been
/// opened) or [open] (one that has). [id] is the sync note's by default.
NoteView mockNote({
  String id = MockIds.syncDesign,
  String title = 'Sync design notes',
  String? body,
  bool closed = false,
  bool open = false,
  String notebookId = MockIds.kanso,
  List<String> tags = const <String>[],
  String status = 'none',
}) {
  return NoteView(
    id: id,
    rev: '1-$id',
    title: title,
    body: body,
    lock: closed
        ? const LockState.closed()
        : (open ? const LockState.open() : const LockState.plain()),
    notebookId: notebookId,
    tags: tags,
    status: status,
    pinned: false,
    createdMs: mockMs(10, 12, 8, 5),
    updatedMs: mockMs(10, 26, 9, 41),
  );
}

/// A plain note for every row of [mockRows] that is not locked, so each can
/// be opened: the row's own title, the open note's text.
Map<String, NoteView> readableMockNotes() => <String, NoteView>{
  for (final row in mockRows)
    if (!row.locked)
      row.id: NoteView(
        id: row.id,
        rev: '1-${row.id}',
        title: row.title,
        body: mockNoteBody,
        lock: const LockState.plain(),
        notebookId: MockIds.kanso,
        tags: const <String>[],
        status: 'none',
        pinned: false,
        createdMs: mockMs(10, 12, 8, 5),
        updatedMs: mockMs(10, 26, 9, 41),
      ),
};

/// How many tasks of the mock library are overdue.
const int mockOverdueTasks = 2;

/// Every group of the Tasks place and every tone of a due chip.
const TaskBoard mockTaskBoard = TaskBoard(
  groups: <TaskSection>[
    TaskSection(
      group: TaskGroupKind.overdue,
      label: 'Overdue',
      items: <TaskItem>[
        TaskItem(
          key: 'a',
          noteId: 'n-a',
          noteTitle: 'Sync design',
          notebookId: 'b-projects',
          notebookName: 'Projects',
          line: 4,
          text: 'Send Sam the draft',
          sourceText: 'Send Sam the draft @2026-10-20',
          checked: false,
          dueLabel: 'Oct 20',
          dueTone: DueTone.overdue,
        ),
      ],
    ),
    TaskSection(
      group: TaskGroupKind.today,
      label: 'Today',
      items: <TaskItem>[
        TaskItem(
          key: 'b',
          noteId: 'n-b',
          noteTitle: 'Weekly review',
          notebookId: 'b-work',
          notebookName: 'Work',
          line: 9,
          text: 'Book the venue',
          sourceText: 'Book the venue @2026-10-27',
          checked: false,
          dueLabel: 'Today',
          dueTone: DueTone.today,
        ),
        TaskItem(
          key: 'c',
          noteId: 'n-c',
          noteTitle: 'Groceries',
          notebookId: 'b-home',
          notebookName: 'Home',
          line: 2,
          text: 'Buy oat milk',
          sourceText: 'Buy oat milk @2026-10-27',
          checked: true,
          dueLabel: 'Today',
          dueTone: DueTone.done,
        ),
      ],
    ),
    TaskSection(
      group: TaskGroupKind.upcoming,
      label: 'Upcoming',
      items: <TaskItem>[
        TaskItem(
          key: 'd',
          noteId: 'n-d',
          noteTitle: 'Trip plan',
          notebookId: 'b-home',
          notebookName: 'Home',
          line: 6,
          text: 'Renew the passport',
          sourceText: 'Renew the passport @2026-11-03',
          checked: false,
          dueLabel: 'Nov 3',
          dueTone: DueTone.plain,
        ),
      ],
    ),
    TaskSection(
      group: TaskGroupKind.noDate,
      label: 'No date',
      items: <TaskItem>[
        TaskItem(
          key: 'e',
          noteId: 'n-e',
          noteTitle: 'Ideas',
          notebookId: 'b-work',
          notebookName: 'Work',
          line: 1,
          text: 'Sketch the onboarding screens',
          sourceText: 'Sketch the onboarding screens',
          checked: false,
        ),
      ],
    ),
    TaskSection(
      group: TaskGroupKind.done,
      label: 'Completed',
      items: <TaskItem>[
        TaskItem(
          key: 'f',
          noteId: 'n-f',
          noteTitle: 'Weekly review',
          notebookId: 'b-work',
          notebookName: 'Work',
          line: 12,
          text: 'File the expenses',
          sourceText: 'File the expenses @2026-10-24',
          checked: true,
          dueLabel: 'Oct 24',
          dueTone: DueTone.done,
        ),
      ],
    ),
  ],
);

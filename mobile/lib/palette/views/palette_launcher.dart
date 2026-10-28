// Kanso — palette_launcher.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:clock/clock.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_bloc/flutter_bloc.dart';
import 'package:go_router/go_router.dart';

import '../../core/bridge/api/linking_types.dart';
import '../../core/bridge/api/types.dart';
import '../../core/models/data/browse_location.dart';
import '../../core/models/repos/abs/journal_repository.dart';
import '../../core/models/repos/abs/library_repository.dart';
import '../../core/router/kanso_routes.dart';
import '../../core/state/library_index/library_index_bloc.dart';
import '../../core/state/sync_status/sync_status_bloc.dart';
import '../../core/utils/journal_time.dart';
import '../../core/views/widgets/kanso_bottom_sheet_route.dart';
import '../../injection.dart';
import '../../journal/utils/open_made_note.dart';
import '../../journal/views/widgets/calendar_sheet.dart';
import '../../library/views/widgets/new_notebook_sheet.dart';
import '../../note/state/note/note_bloc.dart';
import '../../note/state/note_discovery/note_discovery_bloc.dart';
import '../../note/views/widgets/note_html_export.dart';
import '../../note/views/widgets/note_info_sheet.dart';
import '../../note/views/widgets/note_pdf_export.dart';
import '../../note_links/views/widgets/backlinks_sheet.dart';
import '../../note_links/views/widgets/outline_sheet.dart';
import '../../publish/views/widgets/publish_sheet.dart';
import '../../sharing/views/widgets/share_note_sheet.dart';
import '../../shell/state/focus_mode/focus_mode_bloc.dart';
import '../models/data/palette_choice.dart';
import 'widgets/palette_choice_sheet.dart';
import 'widgets/palette_sheet.dart';

/// Opens the command palette for what the app can do where it is opened,
/// runs, goes to, or opens what is chosen.
///
/// The commands offered are the ones in [handlers]: the Rust core lists a
/// command only when the app has a handler for its id, so a row that would
/// do nothing is never shown.
abstract final class PaletteLauncher {
  /// The commands the mobile app can run now.
  ///
  /// The mobile handlers here have an existing surface in the app. Commands
  /// without a mobile surface are left out until that surface is available.
  ///
  /// `publish-notebook` is added only where a notebook is showing: the
  /// catalog offers it with nothing needed, but the sheet needs to know
  /// which notebook to publish.
  static const List<String> handlers = <String>[
    'new-note',
    'new-notebook',
    'search',
    'replace-all',
    'today',
    'import',
    'go-to-notebook',
    'go-to-tag',
    'back-to-notes',
    'show-graph',
    'show-board',
    'history',
    'publish',
    'export-pdf',
    'export-html',
    'copy-note-link',
    'copy-kanso-link',
    'trash',
    'pin',
    'move',
    'add-tag',
    'status',
    'duplicate',
    'toggle-inspector',
    'show-outline',
    'show-backlinks',
    'focus-mode',
    'sync',
    'sign-in',
    'settings',
    'settings-appearance',
    'settings-account',
    'show-task-board',
    'show-local-graph',
    'journal-calendar',
    'previous-day',
    'next-day',
    'share',
  ];

  /// Shows the palette over the whole app.
  static Future<void> open(BuildContext context) async {
    final router = GoRouter.of(context);
    // The overdue badge is extra: a library that cannot be asked costs the
    // count, never the palette.
    final library = getIt.isRegistered<LibraryRepository>()
        ? getIt<LibraryRepository>()
        : null;
    final overdue = library == null
        ? 0
        : (await library.overdueTasks()).fold((_) => 0, (n) => n);
    if (!context.mounted) {
      return;
    }
    final uri = GoRouterState.of(context).uri;
    final noteId = BrowseLocation.parse(uri)?.noteId;
    // The editor already has the full note in memory; use it when present.
    final note = _maybeRead<NoteBloc>(context)?.state.note;
    final path = uri.path;
    final showing = _showing(uri);
    final notebookId = switch (showing) {
      PalettePlace_Notebook(:final id) => id,
      _ => null,
    };
    await showKansoPalette(
      context,
      paletteContext: PaletteContext(
        openNoteId: note?.id ?? noteId,
        signedIn: _maybeRead<SyncStatusBloc>(context)?.state.signedIn ?? false,
        sessionHeld: false,
        aiOn: false,
        settingsOpen: path.startsWith(KansoRoutes.settings),
        imageLine: false,
        audioLine: false,
        sharedNote: path.startsWith(KansoRoutes.shared),
        handlers: <String>[
          ...handlers,
          if (notebookId != null) 'publish-notebook',
        ],
        showing: showing,
        overdueTasks: overdue,
      ),
      onSelected: (target) => unawaited(
        _run(
          context,
          target,
          note: note,
          noteId: noteId,
          notebookId: notebookId,
          router: router,
        ),
      ),
    );
  }

  /// Where the palette is being opened: the place the person is in, if any.
  static PalettePlace? _showing(Uri uri) {
    final location = BrowseLocation.parse(uri);
    if (location == null) {
      return null;
    }
    return switch (location) {
      PlaceLocation(:final place) => switch (place) {
        ListPlace_All() => const PalettePlace.allNotes(),
        ListPlace_Pinned() => const PalettePlace.pinned(),
        ListPlace_Recent() => const PalettePlace.recent(),
        ListPlace_Trash() => const PalettePlace.trash(),
        ListPlace_Notebook(:final id) => PalettePlace.notebook(id: id),
        ListPlace_Status(:final status) => PalettePlace.status(name: status),
        _ => null,
      },
      TasksLocation() => const PalettePlace.tasks(),
      SearchLocation() => null,
    };
  }

  static Future<void> _run(
    BuildContext context,
    PaletteTarget target, {
    required NoteView? note,
    required String? noteId,
    required String? notebookId,
    required GoRouter router,
  }) async {
    switch (target) {
      case PaletteTarget_Place(:final place):
        router.go(_locationOf(place));
      case PaletteTarget_Note(:final id):
        router.go('${KansoRoutes.notes}/note/${Uri.encodeComponent(id)}');
      case PaletteTarget_Heading(:final noteId, :final line):
        router.go(
          '${KansoRoutes.notes}/note/${Uri.encodeComponent(noteId)}'
          '?${KansoRoutes.taskLineParameter}=$line',
        );
      case PaletteTarget_Command(:final id):
        await _runCommand(
          context,
          id,
          note: note,
          noteId: noteId,
          notebookId: notebookId,
          router: router,
        );
    }
  }

  static String _locationOf(PalettePlace place) => switch (place) {
    PalettePlace_AllNotes() => const BrowseLocation.place(
      ListPlace.all(),
    ).location,
    PalettePlace_Pinned() => const BrowseLocation.place(
      ListPlace.pinned(),
    ).location,
    PalettePlace_Recent() => const BrowseLocation.place(
      ListPlace.recent(),
    ).location,
    PalettePlace_Trash() => const BrowseLocation.place(
      ListPlace.trash(),
    ).location,
    PalettePlace_Tasks() => const BrowseLocation.tasks().location,
    PalettePlace_Notebook(:final id) => BrowseLocation.place(
      ListPlace.notebook(id: id),
    ).location,
    PalettePlace_Status(:final name) => BrowseLocation.place(
      ListPlace.status(status: name),
    ).location,
    PalettePlace_Shared() => KansoRoutes.shared,
  };

  static Future<void> _runCommand(
    BuildContext context,
    String id, {
    required NoteView? note,
    required String? noteId,
    required String? notebookId,
    required GoRouter router,
  }) async {
    final library = getIt<LibraryRepository>();
    final needsNote = switch (id) {
      'history' ||
      'export-pdf' ||
      'export-html' ||
      'copy-note-link' ||
      'copy-kanso-link' ||
      'duplicate' ||
      'trash' ||
      'pin' ||
      'status' ||
      'move' ||
      'add-tag' ||
      'publish' ||
      'show-outline' ||
      'show-backlinks' ||
      'toggle-inspector' ||
      'share' => true,
      _ => false,
    };
    final commandNote = needsNote && noteId != null
        ? note ?? await _noteById(context, noteId)
        : note;
    final open = commandNote?.id;
    final openNoteId = note?.id ?? noteId;
    switch (id) {
      case 'new-note':
        router.go(KansoRoutes.newNote);
      case 'new-notebook':
        await NewNotebookSheet.open(context);
      case 'search':
        _leaveFocus(context);
        router.go(KansoRoutes.search);
      case 'replace-all':
        _leaveFocus(context);
        router.go(KansoRoutes.replace);
      case 'today':
        router.go(KansoRoutes.daily);
      case 'import':
        router.go(KansoRoutes.import);
      case 'go-to-notebook':
        _leaveFocus(context);
        router.go(KansoRoutes.notebooks);
      case 'go-to-tag':
        _leaveFocus(context);
        router.go(KansoRoutes.tags);
      case 'back-to-notes':
        _leaveFocus(context);
        router.go(KansoRoutes.notes);
      case 'show-graph':
        _leaveFocus(context);
        router.go(KansoRoutes.graph);
      case 'show-board':
        _leaveFocus(context);
        router.go(
          const BrowseLocation.place(
            ListPlace.all(),
            board: BoardKind.notes,
          ).location,
        );
      case 'settings':
        _leaveFocus(context);
        router.go(KansoRoutes.settings);
      case 'settings-appearance':
        _leaveFocus(context);
        router.go(KansoRoutes.settingsTheme);
      case 'settings-account':
        _leaveFocus(context);
        router.go(KansoRoutes.settingsAccount);
      case 'sign-in':
        _leaveFocus(context);
        router.go(KansoRoutes.settingsSignIn);
      case 'sync':
        final sync = _maybeRead<SyncStatusBloc>(context);
        sync?.add(const SyncStatusEvent.syncRequested());
      case 'history':
        if (open != null) {
          unawaited(
            router.push('${KansoRoutes.history}/${Uri.encodeComponent(open)}'),
          );
        }
      case 'export-pdf':
        if (open != null) {
          await exportNotePdf(context, noteId: open);
        }
      case 'export-html':
        if (open != null) {
          await exportNoteHtml(context, noteId: open);
        }
      case 'copy-note-link':
        if (open != null) {
          await _copyLink(context, open, kanso: false);
        }
      case 'copy-kanso-link':
        if (open != null) {
          await _copyLink(context, open, kanso: true);
        }
      case 'duplicate':
        if (open != null) {
          (await library.duplicate(open)).fold(
            (error) => _say(context, error.message),
            (saved) => router.go(
              '${KansoRoutes.notes}/note/${Uri.encodeComponent(saved.id)}',
            ),
          );
        }
      case 'trash':
        if (open != null) {
          (await library.trash(open)).fold(
            (error) => _say(context, error.message),
            (_) => router.go(KansoRoutes.notes),
          );
        }
      case 'pin':
        if (open != null) {
          await _change(context, const NoteChange.togglePin(), open);
        }
      case 'status':
        if (open != null) {
          await _pickStatus(context, library, open, commandNote?.status);
        }
      case 'move':
        if (open != null) {
          await _pickNotebook(context, library, open);
        }
      case 'add-tag':
        if (open != null) {
          await _pickTag(
            context,
            library,
            open,
            commandNote?.tags ?? const <String>[],
          );
        }
      case 'publish':
        if (open != null) {
          await PublishSheet.showNote(context, open);
        }
      case 'publish-notebook':
        final notebook = notebookId;
        if (notebook != null) {
          await PublishSheet.showNotebook(context, notebook);
        }
      case 'show-outline':
        final view = commandNote;
        final text = view?.body;
        if (view != null && text != null) {
          await OutlineSheet.show(context, noteId: view.id, text: text);
        }
      case 'show-backlinks':
        if (open != null) {
          await BacklinksSheet.show(context, noteId: open);
        }
      case 'focus-mode':
        final focus = _maybeRead<FocusModeBloc>(context);
        focus?.add(const FocusModeEvent.toggled());
      case 'toggle-inspector':
        final index = _maybeRead<LibraryIndexBloc>(context);
        final discovery = _maybeRead<NoteDiscoveryBloc>(context);
        final view = commandNote;
        if (view != null && index != null && discovery != null) {
          await showKansoBottomSheet<void>(
            context,
            builder: (_) => BlocProvider<NoteDiscoveryBloc>.value(
              value: discovery,
              child: NoteInfoSheet(note: view, index: index.state),
            ),
          );
        }
      case 'show-task-board':
        _leaveFocus(context);
        router.go(
          const BrowseLocation.place(
            ListPlace.all(),
            board: BoardKind.tasks,
          ).location,
        );
      case 'show-local-graph':
        if (openNoteId != null) {
          _leaveFocus(context);
          router.go(KansoRoutes.graphLocal(openNoteId));
        }
      case 'journal-calendar':
        final day = await CalendarSheet.show(
          context,
          openNote: openNoteId,
        );
        if (day != null && context.mounted) {
          final result = await getIt<JournalRepository>().openDay(
            day,
            localTimeOf(clock.now()),
          );
          if (!context.mounted) {
            return;
          }
          result.fold(
            (error) => _say(context, error.message),
            (made) => openMadeNote(context, made),
          );
        }
      case 'previous-day' || 'next-day':
        final result = await getIt<JournalRepository>().step(
          openNote: openNoteId,
          now: localTimeOf(clock.now()),
          forward: id == 'next-day',
        );
        if (!context.mounted) {
          return;
        }
        result.fold((error) => _say(context, error.message), (made) {
          if (made != null) {
            openMadeNote(context, made);
          }
        });
      case 'share':
        if (open != null) {
          await ShareNoteSheet.show(context, open);
        }
      default:
        // The core only lists handlers this app declares; anything else is a
        // new catalog entry that has not been given a surface yet.
        break;
    }
  }

  static Future<NoteView?> _noteById(BuildContext context, String id) async {
    final result = await getIt<LibraryRepository>().note(id);
    if (!context.mounted) {
      return null;
    }
    return result.fold((error) {
      _say(context, error.message);
      return null;
    }, (view) => view);
  }

  /// Makes [change] to the open note and says what happened.
  static Future<void> _change(
    BuildContext context,
    NoteChange change,
    String id,
  ) async {
    final library = getIt<LibraryRepository>();
    (await library.change(<String>[id], change)).fold(
      (error) => _say(context, error.message),
      (changed) => _say(context, changed.message),
    );
  }

  static Future<void> _copyLink(
    BuildContext context,
    String id, {
    required bool kanso,
  }) async {
    final library = getIt<LibraryRepository>();
    final result = await library.noteLink(id, kanso: kanso);
    await result.fold(
      (error) async => _say(context, error.message),
      (link) async {
        await Clipboard.setData(ClipboardData(text: link.text));
        _say(context, link.message);
      },
    );
  }

  static Future<void> _pickStatus(
    BuildContext context,
    LibraryRepository library,
    String id,
    String? current,
  ) async {
    final result = await library.statuses();
    final statuses = result.fold(
      (error) {
        _say(context, error.message);
        return const <StatusChoice>[];
      },
      (choices) => choices,
    );
    if (statuses.isEmpty || !context.mounted) {
      return;
    }
    final picked = await PaletteChoiceSheet.show(
      context,
      title: 'Status',
      choices: <PaletteChoice>[
        for (final status in statuses)
          PaletteChoice(
            label: status.label,
            value: status.status,
            checked: status.status == current,
          ),
      ],
    );
    if (picked != null) {
      await _change(context, NoteChange.setStatus(status: picked), id);
    }
  }

  static Future<void> _pickNotebook(
    BuildContext context,
    LibraryRepository library,
    String id,
  ) async {
    final result = await library.notebooks();
    final books = result.fold(
      (error) {
        _say(context, error.message);
        return const <NotebookRow>[];
      },
      (rows) => rows,
    );
    if (books.isEmpty || !context.mounted) {
      return;
    }
    final picked = await PaletteChoiceSheet.show(
      context,
      title: 'Move to',
      choices: <PaletteChoice>[
        for (final book in books)
          PaletteChoice(label: book.name, value: book.id, depth: book.depth),
      ],
    );
    if (picked != null) {
      await _change(context, NoteChange.move(notebookId: picked), id);
    }
  }

  static Future<void> _pickTag(
    BuildContext context,
    LibraryRepository library,
    String id,
    List<String> current,
  ) async {
    final result = await library.tags();
    final tags = result.fold(
      (error) {
        _say(context, error.message);
        return const <TagRow>[];
      },
      (rows) => rows,
    );
    if (tags.isEmpty || !context.mounted) {
      return;
    }
    final picked = await PaletteChoiceSheet.show(
      context,
      title: 'Add tag',
      choices: <PaletteChoice>[
        for (final tag in tags)
          PaletteChoice(
            label: tag.name,
            value: tag.id,
            checked: current.contains(tag.id),
          ),
      ],
    );
    if (picked != null) {
      await _change(context, NoteChange.addTag(tagId: picked), id);
    }
  }

  /// Leaves focus mode before opening a surface that is not the note.
  static void _leaveFocus(BuildContext context) {
    final focus = _maybeRead<FocusModeBloc>(context);
    focus?.add(const FocusModeEvent.exited());
  }

  static void _say(BuildContext context, String message) {
    final messenger = ScaffoldMessenger.maybeOf(context);
    messenger?.showSnackBar(SnackBar(content: Text(message)));
  }

  static T? _maybeRead<T extends Object>(BuildContext context) {
    try {
      return context.read<T>();
    } on ProviderNotFoundException {
      return null;
    }
  }
}

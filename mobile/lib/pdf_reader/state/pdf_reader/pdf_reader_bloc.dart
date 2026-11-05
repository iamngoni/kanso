// Kanso — pdf_reader_bloc.dart
// Created 2026-11-05.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/pdf_types.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/preferences_repository.dart';
import '../../../core/models/repos/abs/session_repository.dart';
import '../../models/data/pdf_opening.dart';
import '../../models/data/pdf_zoom.dart';
import '../../models/repos/abs/pdf_reader_repository.dart';

part 'pdf_reader_bloc.freezed.dart';
part 'pdf_reader_event.dart';
part 'pdf_reader_state.dart';

/// The hits of a search handed to the app at once: the core's
/// `FIND_HITS_SHOWN`.
const int pdfSearchLimit = 500;

/// A PDF attachment open for reading (issue #43): its pages drawn by the
/// Rust core, the selection made on them, a search through its text and the
/// highlights the note holds for it.
///
/// The document itself lives here, not in the state: the pages ask for
/// pictures through [session], and [close] ends the reader's thread.
@injectable
class PdfReaderBloc extends Bloc<PdfReaderEvent, PdfReaderState> {
  PdfReaderBloc(this._repository, this._preferences, this._sessionRepository)
    : super(const PdfReaderState()) {
    on<PdfReaderStartedEvent>(_onStarted, transformer: restartable());
    on<PdfReaderPasswordSubmittedEvent>(
      _onPasswordSubmitted,
      transformer: restartable(),
    );
    on<PdfReaderRetriedEvent>(_onRetried, transformer: restartable());
    on<PdfReaderZoomChangedEvent>((event, emit) {
      emit(state.copyWith(zoom: event.zoom));
    });
    on<PdfReaderInvertChangedEvent>(
      _onInvertChanged,
      transformer: sequential(),
    );
    on<PdfReaderDarkChangedEvent>(_onDarkChanged, transformer: sequential());
    on<PdfReaderPageChangedEvent>((event, emit) {
      final info = state.info;
      if (info == null || event.page == state.page) {
        return;
      }
      emit(state.copyWith(page: _page(event.page, info.pages)));
    });
    on<PdfReaderJumpToEvent>((event, emit) {
      final info = state.info;
      if (info == null) {
        return;
      }
      emit(
        state.copyWith(
          page: _page(event.page, info.pages),
          jump: state.jump + 1,
          selection: null,
        ),
      );
    });
    on<PdfReaderWordSelectedEvent>(_onWordSelected, transformer: sequential());
    on<PdfReaderLineSelectedEvent>(_onLineSelected, transformer: sequential());
    on<PdfReaderSelectionExtendedEvent>(
      _onSelectionExtended,
      transformer: restartable(),
    );
    on<PdfReaderSelectionClearedEvent>((event, emit) {
      emit(state.copyWith(selection: null, focusHighlight: null));
    });
    on<PdfReaderSearchChangedEvent>(_onSearchChanged);
    on<PdfReaderSearchRunEvent>(_onSearchRun, transformer: restartable());
    on<PdfReaderSearchNextEvent>(_onSearchNext);
    on<PdfReaderSearchPreviousEvent>(_onSearchPrevious);
    on<PdfReaderSearchClosedEvent>((event, emit) {
      _searchTimer?.cancel();
      emit(state.copyWith(query: '', found: null, hit: 0));
    });
    on<PdfReaderHighlightRequestedEvent>(
      _onHighlightRequested,
      transformer: sequential(),
    );
    on<PdfReaderHighlightFocusedEvent>((event, emit) async {
      final info = state.info;
      if (info == null) {
        return;
      }
      final id = event.id;
      final mark = id == null
          ? null
          : state.marks?.marks.where((mark) => mark.id == id).firstOrNull;
      emit(
        state.copyWith(
          focusHighlight: id,
          page: mark == null ? state.page : _page(mark.page, info.pages),
          jump: mark == null ? state.jump : state.jump + 1,
        ),
      );
    });
    on<PdfReaderNoticeDismissedEvent>((event, emit) {
      emit(state.copyWith(notice: null));
    });
    on<PdfReaderLockChangedEvent>(_onLockChanged, transformer: restartable());
    _lockEvents = _sessionRepository.lockEvents().listen((event) {
      if (event is LockEvent_Locked) {
        add(const PdfReaderEvent.lockChanged());
      }
    });
  }

  final PdfReaderRepository _repository;
  final PreferencesRepository _preferences;
  final SessionRepository _sessionRepository;
  late final StreamSubscription<LockEvent> _lockEvents;

  /// The document drawn: the pages render through it. Kept out of the state
  /// so a page picture never rebuilds the page for an identity change.
  PdfDocumentSession? _session;

  /// The document the pages read through, once it is open.
  PdfDocumentSession? get session => _session;

  /// The number of the last open begun: only its answer is shown, so a slow
  /// open cannot replace a newer one.
  int _opens = 0;

  /// Whether the reader is in a dark theme, and whether the PDF is kept as
  /// printed there. Together they decide the page rendering without writing
  /// a preference when the theme changes.
  bool _dark = false;
  bool _keepPrinted = false;

  Timer? _searchTimer;

  /// The page [page] (1-based) held to the document's [pages].
  static int _page(int page, int pages) {
    if (pages < 1) {
      return 1;
    }
    if (page < 1) {
      return 1;
    }
    return page > pages ? pages : page;
  }

  Future<void> _onStarted(
    PdfReaderStartedEvent event,
    Emitter<PdfReaderState> emit,
  ) async {
    _searchTimer?.cancel();
    _dark = event.dark;
    _keepPrinted = false;
    emit(
      PdfReaderState(
        noteId: event.noteId,
        file: event.file,
        name: event.name,
        zoom: state.zoom,
        invert: event.dark,
        focusHighlight: event.highlight,
      ),
    );
    final saved = await _preferences.load();
    if (emit.isDone) {
      return;
    }
    _keepPrinted = saved.fold(
      (_) => false,
      (preferences) => preferences.pdfOriginalPages[event.file] ?? false,
    );
    emit(state.copyWith(invert: _dark && !_keepPrinted));
    await _open(emit, page: event.page);
  }

  void _onDarkChanged(
    PdfReaderDarkChangedEvent event,
    Emitter<PdfReaderState> emit,
  ) {
    _dark = event.dark;
    emit(state.copyWith(invert: _dark && !_keepPrinted));
  }

  Future<void> _onInvertChanged(
    PdfReaderInvertChangedEvent event,
    Emitter<PdfReaderState> emit,
  ) async {
    _keepPrinted = !event.invert;
    emit(state.copyWith(invert: event.invert));
    final file = state.file;
    if (file.isEmpty) {
      return;
    }
    await _preferences.update((current) {
      final original = Map<String, bool>.of(current.pdfOriginalPages);
      if (event.invert) {
        original.remove(file);
      } else {
        original[file] = true;
      }
      return current.copyWith(pdfOriginalPages: original);
    });
  }

  Future<void> _onPasswordSubmitted(
    PdfReaderPasswordSubmittedEvent event,
    Emitter<PdfReaderState> emit,
  ) => _open(emit, password: event.password, page: state.page);

  Future<void> _onRetried(
    PdfReaderRetriedEvent event,
    Emitter<PdfReaderState> emit,
  ) => _open(emit, page: state.page);

  /// Opens the attachment, with [password] for an encrypted one, and reads
  /// the note's highlights once it is open.
  Future<void> _open(
    Emitter<PdfReaderState> emit, {
    String? password,
    int? page,
  }) async {
    final open = ++_opens;
    _session?.dispose();
    _session = null;
    emit(
      state.copyWith(
        status: PdfReaderStatus.opening,
        declined: null,
        error: null,
        marksError: null,
        marks: null,
        selection: null,
        found: null,
        hit: 0,
        passwordWrong: false,
        notice: null,
      ),
    );

    final result = await _repository.open(
      noteId: state.noteId,
      file: state.file,
      password: password,
    );
    if (emit.isDone) {
      _discard(result);
      return;
    }
    if (open != _opens) {
      _discard(result);
      return;
    }

    switch (result) {
      case Left(:final value):
        emit(
          state.copyWith(
            status: PdfReaderStatus.failed,
            error: value,
          ),
        );
      case Right(value: PdfOpened(:final session, :final info)):
        _session = session;
        emit(
          state.copyWith(
            status: PdfReaderStatus.opened,
            info: info,
            page: _page(page ?? state.page, info.pages),
          ),
        );
        await _readMarks(emit);
      case Right(value: PdfDeclinedOpening(:final reason)):
        emit(
          state.copyWith(
            status: PdfReaderStatus.declined,
            declined: reason,
            passwordWrong: reason is PdfDeclined_WrongPassword,
          ),
        );
    }
  }

  /// A session that arrived after its open was replaced is let go.
  void _discard(Either<ApplicationError, PdfOpening> result) {
    switch (result) {
      case Right(value: PdfOpened(:final session)):
        session.dispose();
      case Left() || Right():
        break;
    }
  }

  Future<void> _readMarks(Emitter<PdfReaderState> emit) async {
    if (_session == null || emit.isDone) {
      return;
    }
    final marks = await _repository.marks(
      noteId: state.noteId,
      file: state.file,
    );
    if (emit.isDone || state.status != PdfReaderStatus.opened) {
      return;
    }
    marks.fold(
      (error) => emit(state.copyWith(marksError: error)),
      (marks) {
        final info = state.info;
        final id = state.focusHighlight;
        final focused = id == null
            ? null
            : marks.marks.where((mark) => mark.id == id).firstOrNull;
        emit(
          state.copyWith(
            marks: marks,
            marksError: null,
            page: info == null || focused == null
                ? state.page
                : _page(focused.page, info.pages),
            jump: focused == null ? state.jump : state.jump + 1,
          ),
        );
      },
    );
  }

  Future<void> _onWordSelected(
    PdfReaderWordSelectedEvent event,
    Emitter<PdfReaderState> emit,
  ) async {
    final session = _session;
    if (session == null) {
      return;
    }
    final selection = await session.wordAt(event.page, event.x, event.y);
    if (emit.isDone) {
      return;
    }
    switch (selection) {
      case Left(:final value):
        emit(state.copyWith(notice: value));
      case Right(value: final selected):
        emit(
          state.copyWith(
            selection: selected,
            focusHighlight: null,
            notice: null,
          ),
        );
    }
  }

  Future<void> _onLineSelected(
    PdfReaderLineSelectedEvent event,
    Emitter<PdfReaderState> emit,
  ) async {
    final session = _session;
    if (session == null) {
      return;
    }
    final selection = await session.lineAt(event.page, event.x, event.y);
    if (emit.isDone) {
      return;
    }
    switch (selection) {
      case Left(:final value):
        emit(state.copyWith(notice: value));
      case Right(value: final selected):
        emit(
          state.copyWith(
            selection: selected,
            focusHighlight: null,
            notice: null,
          ),
        );
    }
  }

  Future<void> _onSelectionExtended(
    PdfReaderSelectionExtendedEvent event,
    Emitter<PdfReaderState> emit,
  ) async {
    final session = _session;
    final selection = state.selection;
    if (session == null || selection == null) {
      return;
    }
    final anchor = event.fromStart ? selection.end : selection.start;
    final selected = await session.extend(
      anchor,
      event.page,
      event.x,
      event.y,
    );
    if (emit.isDone) {
      return;
    }
    switch (selected) {
      case Left(:final value):
        emit(state.copyWith(notice: value));
      case Right(value: final selected):
        if (selected != null) {
          emit(state.copyWith(selection: selected));
        }
    }
  }

  void _onSearchChanged(
    PdfReaderSearchChangedEvent event,
    Emitter<PdfReaderState> emit,
  ) {
    _searchTimer?.cancel();
    final query = event.query.trim();
    emit(state.copyWith(query: event.query));
    if (query.isEmpty) {
      emit(state.copyWith(found: null, hit: 0));
      return;
    }
    _searchTimer = Timer(
      const Duration(milliseconds: 220),
      () => add(PdfReaderEvent.searchRun(query)),
    );
  }

  Future<void> _onSearchRun(
    PdfReaderSearchRunEvent event,
    Emitter<PdfReaderState> emit,
  ) async {
    final session = _session;
    if (session == null || event.query != state.query.trim()) {
      return;
    }
    final found = await session.find(event.query, pdfSearchLimit);
    if (emit.isDone || event.query != state.query.trim()) {
      return;
    }
    switch (found) {
      case Left(:final value):
        emit(state.copyWith(found: null, notice: value));
      case Right(value: final hits):
        final info = state.info;
        final first = hits.hits.firstOrNull;
        emit(
          state.copyWith(
            found: hits,
            hit: 0,
            page: info == null || first == null
                ? state.page
                : _page(first.page, info.pages),
            jump: first == null ? state.jump : state.jump + 1,
            selection: null,
          ),
        );
    }
  }

  void _onSearchNext(
    PdfReaderSearchNextEvent event,
    Emitter<PdfReaderState> emit,
  ) => _stepHit(1, emit);

  void _onSearchPrevious(
    PdfReaderSearchPreviousEvent event,
    Emitter<PdfReaderState> emit,
  ) => _stepHit(-1, emit);

  void _stepHit(int by, Emitter<PdfReaderState> emit) {
    final found = state.found;
    final info = state.info;
    if (found == null || info == null || found.hits.isEmpty) {
      return;
    }
    final total = found.hits.length;
    final hit = ((state.hit + by) % total + total) % total;
    emit(
      state.copyWith(
        hit: hit,
        page: _page(found.hits[hit].page, info.pages),
        jump: state.jump + 1,
        selection: null,
      ),
    );
  }

  Future<void> _onHighlightRequested(
    PdfReaderHighlightRequestedEvent event,
    Emitter<PdfReaderState> emit,
  ) async {
    final session = _session;
    final selection = state.selection;
    final marks = state.marks;
    if (session == null || selection == null || marks == null) {
      return;
    }
    if (!marks.editable || state.adding) {
      return;
    }
    emit(state.copyWith(adding: true, notice: null));
    final edited = await session.addHighlight(
      baseRev: marks.rev,
      start: selection.start,
      end: selection.end,
      color: event.color,
      quote: event.quote,
    );
    if (emit.isDone) {
      return;
    }
    switch (edited) {
      case Left(:final value):
        emit(state.copyWith(adding: false, notice: value));
      case Right(value: final saved):
        emit(
          state.copyWith(
            adding: false,
            selection: null,
            focusHighlight: saved.id,
          ),
        );
        await _readMarks(emit);
    }
  }

  @override
  Future<void> close() async {
    _searchTimer?.cancel();
    await _lockEvents.cancel();
    _session?.dispose();
    _session = null;
    return super.close();
  }

  Future<void> _onLockChanged(
    PdfReaderLockChangedEvent event,
    Emitter<PdfReaderState> emit,
  ) async {
    _opens++;
    _searchTimer?.cancel();
    if (_session == null) {
      return;
    }
    final unlocked = await _sessionRepository.isNoteUnlocked(state.noteId);
    if (emit.isDone || unlocked.getOrElse(() => false)) {
      return;
    }
    _session?.dispose();
    _session = null;
    emit(
      state.copyWith(
        status: PdfReaderStatus.declined,
        info: null,
        marks: null,
        declined: const PdfDeclined.lockedNote(
          message: 'Unlock this note to open its PDF.',
        ),
        error: null,
        marksError: null,
        selection: null,
        query: '',
        found: null,
        hit: 0,
        adding: false,
        passwordWrong: false,
        focusHighlight: null,
        notice: null,
      ),
    );
  }
}

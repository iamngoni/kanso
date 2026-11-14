// Kanso — shared_notebooks_bloc.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/shared_notebooks_repository.dart';
import '../../../core/models/repos/abs/shared_notes_repository.dart';
import '../../models/data/share_problem.dart';
import '../../services/book_notice_inbox.dart';
import '../../utils/share_problem_of.dart';

part 'shared_notebooks_bloc.freezed.dart';
part 'shared_notebooks_event.dart';
part 'shared_notebooks_state.dart';

/// The library's Shared section: the notebooks shared here, with their
/// people and whether each is up to date, and the notebook invitations
/// waiting, to accept or decline. The desktop sidebar's Shared section.
///
/// Read when it starts and again whenever a sync of shared notes or a
/// notebook's room says something changed. What a sync says of a notebook
/// that ended here ("You were removed from "Studio". Your copy is in …"),
/// or of a note deleted here that an edit brought back, is kept by the
/// [BookNoticeInbox] (which hears syncs while this section isn't built) and
/// shown in [SharedNotebooksState.notices] until it is read.
@injectable
class SharedNotebooksBloc
    extends Bloc<SharedNotebooksEvent, SharedNotebooksState> {
  SharedNotebooksBloc(this._books, this._shared, this._inbox)
    : super(const SharedNotebooksState()) {
    on<SharedNotebooksStartedEvent>(_onStarted, transformer: restartable());
    on<SharedNotebooksRefreshedEvent>(
      (event, emit) => _read(emit),
      transformer: restartable(),
    );
    on<SharedNotebooksAcceptedEvent>(_onAccepted, transformer: concurrent());
    on<SharedNotebooksDeclinedEvent>(_onDeclined, transformer: concurrent());
    on<SharedNotebooksNoticeReadEvent>((event, emit) {
      if (state.notices.isNotEmpty) {
        _inbox.dismissFirst();
      }
      emit(
        state.copyWith(notices: _inbox.notices, joined: null, problem: null),
      );
    });
  }

  final SharedNotebooksRepository _books;
  final SharedNotesRepository _shared;
  final BookNoticeInbox _inbox;

  /// The invitations answered here.
  final Set<String> _answered = <String>{};

  Future<void> _onStarted(
    SharedNotebooksStartedEvent event,
    Emitter<SharedNotebooksState> emit,
  ) async {
    _inbox.start();
    await _read(emit);
    await Future.wait(<Future<void>>[
      emit.onEach<SharedEvent>(
        _shared.events(),
        onData: (news) {
          if (news.bookNotices.isNotEmpty ||
              news.notebooks.isNotEmpty ||
              news.bookInvitations != state.invitations.length) {
            add(const SharedNotebooksEvent.refreshed());
          }
        },
      ),
      emit.onEach<List<String>>(
        _inbox.changes,
        onData: (notices) => emit(state.copyWith(notices: notices)),
      ),
    ]);
  }

  Future<void> _read(Emitter<SharedNotebooksState> emit) async {
    final books = await _books.sharedBooks();
    final invitations = await _books.invitations();
    final kept = _inbox.notices;
    if (books.fold((error) => error is SignedOutError, (_) => false)) {
      emit(
        state.copyWith(
          status: LoadStatus.ready,
          books: const <SharedBookRow>[],
          invitations: const <BookInvitationRow>[],
          notices: kept,
          updateLine: null,
        ),
      );
      return;
    }
    emit(
      state.copyWith(
        status: LoadStatus.ready,
        books: books.fold((_) => state.books, (rows) => rows),
        invitations: invitations.fold(
          (error) => error is SignedOutError
              ? const <BookInvitationRow>[]
              : state.invitations,
          (rows) => <BookInvitationRow>[
            for (final row in rows)
              if (!_answered.contains(row.shareId)) row,
          ],
        ),
        notices: kept,
        // Asked after the server was: its refusal is what sets it.
        updateLine: _books.updateLine(),
      ),
    );
  }

  Future<void> _onAccepted(
    SharedNotebooksAcceptedEvent event,
    Emitter<SharedNotebooksState> emit,
  ) async {
    if (state.working.contains(event.shareId)) {
      return;
    }
    emit(
      state.copyWith(
        working: <String>{...state.working, event.shareId},
        problem: null,
      ),
    );
    final accepted = await _books.accept(event.shareId);
    final working = <String>{...state.working}..remove(event.shareId);
    await accepted.fold(
      (error) async => emit(
        state.copyWith(working: working, problem: shareProblemOf(error)),
      ),
      (notebookId) async {
        _answered.add(event.shareId);
        emit(
          state.copyWith(
            working: working,
            joined: notebookId,
            invitations: <BookInvitationRow>[
              for (final row in state.invitations)
                if (row.shareId != event.shareId) row,
            ],
          ),
        );
        await _read(emit);
      },
    );
  }

  Future<void> _onDeclined(
    SharedNotebooksDeclinedEvent event,
    Emitter<SharedNotebooksState> emit,
  ) async {
    if (state.working.contains(event.shareId)) {
      return;
    }
    emit(
      state.copyWith(
        working: <String>{...state.working, event.shareId},
        problem: null,
      ),
    );
    final declined = await _books.decline(event.shareId);
    final working = <String>{...state.working}..remove(event.shareId);
    emit(
      declined.fold(
        (error) =>
            state.copyWith(working: working, problem: shareProblemOf(error)),
        (_) {
          _answered.add(event.shareId);
          return state.copyWith(
            working: working,
            invitations: <BookInvitationRow>[
              for (final row in state.invitations)
                if (row.shareId != event.shareId) row,
            ],
          );
        },
      ),
    );
  }
}

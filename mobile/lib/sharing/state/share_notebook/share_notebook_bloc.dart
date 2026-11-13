// Kanso — share_notebook_bloc.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/shared_notebooks_repository.dart';
import '../../../core/models/repos/abs/shared_notes_repository.dart';
import '../../models/data/book_prompt.dart';
import '../../models/data/share_problem.dart';
import '../../models/data/share_sheet_status.dart';
import '../../utils/share_problem_of.dart';

part 'share_notebook_bloc.freezed.dart';
part 'share_notebook_event.dart';
part 'share_notebook_state.dart';

/// The sharing of one notebook, as its Share sheet shows it: the desktop's
/// Share notebook sheet.
///
/// A notebook that isn't shared yet says what sharing it would do (how many
/// notes, the locked ones that stay in a private notebook) and offers an
/// invite field: the first person invited shares it, and the sheet shows
/// "Sharing Studio… 120 of 340" while its notes go up. A shared notebook
/// shows its people with the roles each one's menu offers (the Rust core
/// decides which), an invite field when this account may invite, and Leave
/// or Stop sharing. A person's keys are looked up before they are invited,
/// and their fingerprint is compared first when they are new or their keys
/// changed.
///
/// Steps run one after another, in the order they came; how far sharing
/// has got arrives while one runs.
@injectable
class ShareNotebookBloc extends Bloc<ShareNotebookEvent, ShareNotebookState> {
  ShareNotebookBloc(this._books, this._people)
    : super(const ShareNotebookState()) {
    on<ShareNotebookProgressedEvent>(
      (event, emit) => emit(
        event.progress.notebookId == state.notebookId && state.busy
            ? state.copyWith(progress: event.progress)
            : state,
      ),
    );
    on<ShareNotebookEvent>(
      (event, emit) => switch (event) {
        ShareNotebookStartedEvent(:final notebookId) => _onStarted(
          notebookId,
          emit,
        ),
        ShareNotebookRetriedEvent() => _load(emit),
        ShareNotebookInviteRoleChosenEvent(:final role) => _now(
          emit,
          state.copyWith(inviteRole: role),
        ),
        ShareNotebookInvitedEvent(:final username) => _onInvited(
          username,
          emit,
        ),
        ShareNotebookFingerprintConfirmedEvent() => _onFingerprintConfirmed(
          emit,
        ),
        ShareNotebookInvitedUnverifiedEvent() => _onInvitedUnverified(emit),
        ShareNotebookNewKeyAcceptedEvent() => _onNewKeyAccepted(emit),
        ShareNotebookPromptDismissedEvent() => _now(
          emit,
          state.copyWith(prompt: null, problem: null),
        ),
        ShareNotebookMemberRoleChangedEvent(:final accountId, :final role) =>
          _act(emit, (id) => _books.setRole(id, accountId, role)),
        ShareNotebookMemberRemovedEvent(:final member) => _act(
          emit,
          (id) => _books.remove(id, member.accountId),
          done: BookDone.removed(member.username),
        ),
        ShareNotebookEndedEvent() => _onEnded(emit),
        // Handled above, while another step runs.
        ShareNotebookProgressedEvent() => Future<void>.value(),
      },
      transformer: sequential(),
    );
    _progress = _books.progress().listen(
      (progress) => add(ShareNotebookEvent.progressed(progress)),
    );
  }

  final SharedNotebooksRepository _books;
  final SharedNotesRepository _people;
  late final StreamSubscription<BookShareProgress> _progress;

  Future<void> _now(
    Emitter<ShareNotebookState> emit,
    ShareNotebookState next,
  ) {
    emit(next);
    return Future<void>.value();
  }

  Future<void> _onStarted(
    String notebookId,
    Emitter<ShareNotebookState> emit,
  ) {
    emit(ShareNotebookState(notebookId: notebookId));
    return _load(emit);
  }

  /// Reads the notebook's sharing, or what sharing it would do.
  Future<void> _load(Emitter<ShareNotebookState> emit) async {
    emit(
      ShareNotebookState(
        notebookId: state.notebookId,
        inviteRole: state.inviteRole,
      ),
    );
    final read = await _books.info(state.notebookId);
    switch (read) {
      case Left(value: SignedOutError()):
        emit(state.copyWith(status: ShareSheetStatus.signedOut));
      case Left(value: LockedError()):
        emit(state.copyWith(status: ShareSheetStatus.locked));
      case Left():
        emit(state.copyWith(status: ShareSheetStatus.failed));
      case Right(value: final info?):
        emit(
          state.copyWith(
            status: ShareSheetStatus.ready,
            info: info,
            inviteRole: _allowed(info, state.inviteRole),
          ),
        );
      case Right():
        await _check(emit);
    }
  }

  Future<void> _check(Emitter<ShareNotebookState> emit) async {
    final can = await _books.canShare(state.notebookId);
    emit(
      can.fold(
        (error) => switch (error) {
          SignedOutError() => state.copyWith(
            status: ShareSheetStatus.signedOut,
          ),
          LockedError() => state.copyWith(status: ShareSheetStatus.locked),
          InvalidError(:final message) => state.copyWith(
            status: ShareSheetStatus.notShareable,
            reason: message,
          ),
          _ => state.copyWith(status: ShareSheetStatus.failed),
        },
        (check) => check.notes == 0 && check.locked == 0
            ? state.copyWith(
                status: ShareSheetStatus.notShareable,
                check: check,
              )
            : state.copyWith(status: ShareSheetStatus.ready, check: check),
      ),
    );
  }

  /// [role], when this account may invite with it; else the first it may.
  BookRole _allowed(BookShareInfo info, BookRole role) {
    final roles = info.inviteRoles;
    if (roles.isEmpty || roles.contains(role)) {
      return role;
    }
    return roles.contains(BookRole.editor) ? BookRole.editor : roles.first;
  }

  Future<void> _act(
    Emitter<ShareNotebookState> emit,
    Future<Either<ApplicationError, BookShareInfo>> Function(String id) work, {
    BookDone? done,
  }) async {
    if (state.info == null || state.busy) {
      return;
    }
    emit(state.copyWith(busy: true, problem: null, done: null));
    final result = await work(state.notebookId);
    emit(
      result.fold(
        (error) => state.copyWith(busy: false, problem: shareProblemOf(error)),
        (info) => state.copyWith(busy: false, info: info, done: done),
      ),
    );
  }

  Future<void> _onInvited(
    String username,
    Emitter<ShareNotebookState> emit,
  ) async {
    if (state.busy) {
      return;
    }
    final name = username.trim();
    if (name.isEmpty) {
      emit(state.copyWith(problem: const ShareProblem.nameMissing()));
      return;
    }
    emit(state.copyWith(busy: true, problem: null, done: null, prompt: null));
    final role = state.inviteRole;
    final looked = await _people.lookupPerson(name);
    await looked.fold(
      (error) async =>
          emit(state.copyWith(busy: false, problem: shareProblemOf(error))),
      (lookup) async {
        switch (lookup) {
          case PersonLookup_Found(field0: final person) when person.verified:
            await _invite(person.username, role, emit);
          case PersonLookup_Found(field0: final person):
            emit(
              state.copyWith(
                busy: false,
                prompt: BookPrompt.verify(person: person, role: role),
              ),
            );
          case PersonLookup_Changed(:final pinned, :final offered):
            emit(
              state.copyWith(
                busy: false,
                prompt: BookPrompt.keyChanged(
                  offered: offered,
                  pinned: pinned,
                  role: role,
                ),
              ),
            );
        }
      },
    );
  }

  /// Invites [username], whose keys are pinned: the first one shares the
  /// notebook.
  Future<void> _invite(
    String username,
    BookRole role,
    Emitter<ShareNotebookState> emit,
  ) async {
    final first = state.info == null;
    emit(
      state.copyWith(busy: true, problem: null, prompt: null, progress: null),
    );
    final result = first
        ? await _books.share(state.notebookId, <(String, BookRole)>[
            (username, role),
          ])
        : await _books.invite(state.notebookId, username, role);
    emit(
      result.fold(
        (error) => state.copyWith(
          busy: false,
          progress: null,
          problem: shareProblemOf(error),
        ),
        (info) => state.copyWith(
          busy: false,
          progress: null,
          info: info,
          check: null,
          inviteRole: _allowed(info, role),
          done: first ? BookDone.shared(username) : BookDone.invited(username),
        ),
      ),
    );
  }

  Future<void> _onFingerprintConfirmed(
    Emitter<ShareNotebookState> emit,
  ) async {
    final prompt = state.prompt;
    if (prompt is! BookPromptVerify || state.busy) {
      return;
    }
    emit(state.copyWith(busy: true, problem: null));
    final verified = await _people.verifyPerson(prompt.person.accountId);
    await verified.fold(
      (error) async =>
          emit(state.copyWith(busy: false, problem: shareProblemOf(error))),
      (_) => _invite(prompt.person.username, prompt.role, emit),
    );
  }

  Future<void> _onInvitedUnverified(Emitter<ShareNotebookState> emit) async {
    final prompt = state.prompt;
    if (prompt is! BookPromptVerify || state.busy) {
      return;
    }
    await _invite(prompt.person.username, prompt.role, emit);
  }

  Future<void> _onNewKeyAccepted(Emitter<ShareNotebookState> emit) async {
    final prompt = state.prompt;
    if (prompt is! BookPromptKeyChanged || state.busy) {
      return;
    }
    emit(state.copyWith(busy: true, problem: null));
    final pinned = await _people.acceptNewKey(
      prompt.offered.username,
      prompt.offered.fingerprint,
    );
    emit(
      pinned.fold(
        (error) => state.copyWith(busy: false, problem: shareProblemOf(error)),
        // Trusted, not verified: the fingerprint is compared next.
        (person) => state.copyWith(
          busy: false,
          prompt: BookPrompt.verify(person: person, role: prompt.role),
        ),
      ),
    );
  }

  Future<void> _onEnded(Emitter<ShareNotebookState> emit) async {
    final info = state.info;
    if (info == null || state.busy) {
      return;
    }
    emit(state.copyWith(busy: true, problem: null, done: null));
    final ended = info.owned
        ? await _books.stop(state.notebookId)
        : await _books.leave(state.notebookId);
    emit(
      ended.fold(
        (error) => state.copyWith(busy: false, problem: shareProblemOf(error)),
        (row) =>
            state.copyWith(busy: false, info: null, done: BookDone.ended(row)),
      ),
    );
  }

  @override
  Future<void> close() async {
    await _progress.cancel();
    return super.close();
  }
}

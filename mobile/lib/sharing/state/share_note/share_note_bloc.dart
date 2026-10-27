// Kanso — share_note_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:ui';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:dartz/dartz.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../../core/models/repos/abs/shared_notes_repository.dart';
import '../../../core/models/repos/abs/system_share_repository.dart';
import '../../models/data/share_done.dart';
import '../../models/data/share_problem.dart';
import '../../models/data/share_prompt.dart';
import '../../models/data/share_sheet_status.dart';
import '../../utils/share_problem_of.dart';

part 'share_note_bloc.freezed.dart';
part 'share_note_event.dart';
part 'share_note_state.dart';

/// The sharing of one note, as its share sheet shows it.
///
/// A note that is not shared yet offers an invite field: the first person
/// invited makes it shared. A shared note shows its people, with their roles
/// for the owner to change, the people who asked through a link, and the
/// links. The keys of a person are looked up before they are invited, and
/// their fingerprint is compared first when they are new, or when their keys
/// changed. Locked notes can't be shared; the state is `locked` and nothing
/// more.
///
/// Steps run one after another, in the order they came.
@injectable
class ShareNoteBloc extends Bloc<ShareNoteEvent, ShareNoteState> {
  ShareNoteBloc(this._shared, this._system) : super(const ShareNoteState()) {
    // One handler for every event, so they all run one after another.
    on<ShareNoteEvent>(
      (event, emit) => switch (event) {
        ShareNoteStartedEvent(:final noteId) => _onStarted(noteId, emit),
        ShareNoteRetriedEvent() => _load(emit),
        ShareNoteInviteRoleChosenEvent(:final role) => _emitNow(
          emit,
          state.copyWith(inviteRole: role),
        ),
        ShareNoteInvitedEvent(:final username) => _onInvited(username, emit),
        ShareNoteFingerprintConfirmedEvent() => _onFingerprintConfirmed(emit),
        ShareNoteInvitedUnverifiedEvent() => _onInvitedUnverified(emit),
        ShareNoteNewKeyAcceptedEvent() => _onNewKeyAccepted(emit),
        ShareNotePromptDismissedEvent() => _emitNow(
          emit,
          state.copyWith(prompt: null, problem: null),
        ),
        ShareNoteKeyReviewedEvent(:final username) => _onKeyReviewed(
          username,
          emit,
        ),
        ShareNoteMemberRoleChangedEvent(:final accountId, :final role) => _act(
          emit,
          (noteId) => _shared.setRole(noteId, accountId, role),
        ),
        ShareNoteMemberRemovedEvent(:final accountId) => _act(
          emit,
          (noteId) => _shared.remove(noteId, accountId),
        ),
        ShareNoteRequestApprovedEvent(:final accountId) => _act(
          emit,
          (noteId) => _shared.approveRequest(noteId, accountId),
        ),
        ShareNoteRequestDeclinedEvent(:final accountId) => _act(
          emit,
          (noteId) => _shared.declineRequest(noteId, accountId),
        ),
        ShareNoteLinkRoleChosenEvent(:final role) => _emitNow(
          emit,
          state.copyWith(linkRole: role),
        ),
        ShareNoteLinkDaysChosenEvent(:final days) => _emitNow(
          emit,
          state.copyWith(linkDays: days),
        ),
        ShareNoteLinkMadeEvent() => _onLinkMade(emit),
        ShareNoteLinkCancelledEvent(:final linkId) => _act(
          emit,
          (noteId) => _shared.cancelLink(noteId, linkId),
        ),
        ShareNoteLinkCopiedEvent() => _onLinkCopied(emit),
        ShareNoteLinkSentEvent(:final origin) => _onLinkSent(origin, emit),
        ShareNoteEndAskedEvent() => _emitNow(
          emit,
          state.info == null || state.busy
              ? state
              : state.copyWith(confirmEnd: true),
        ),
        ShareNoteEndDismissedEvent() => _emitNow(
          emit,
          state.copyWith(confirmEnd: false),
        ),
        ShareNoteEndConfirmedEvent() => _onEndConfirmed(emit),
      },
      transformer: sequential(),
    );
  }

  final SharedNotesRepository _shared;
  final SystemShareRepository _system;

  Future<void> _emitNow(Emitter<ShareNoteState> emit, ShareNoteState next) {
    emit(next);
    return Future<void>.value();
  }

  Future<void> _onStarted(String noteId, Emitter<ShareNoteState> emit) {
    emit(ShareNoteState(noteId: noteId));
    return _load(emit);
  }

  /// Reads the note's sharing, or why there is none to show.
  Future<void> _load(Emitter<ShareNoteState> emit) async {
    emit(
      ShareNoteState(
        noteId: state.noteId,
        inviteRole: state.inviteRole,
        linkRole: state.linkRole,
        linkDays: state.linkDays,
      ),
    );
    final read = await _shared.info(state.noteId);
    switch (read) {
      case Left(value: LockedError()):
        emit(state.copyWith(status: ShareSheetStatus.locked));
      case Left(value: SignedOutError()):
        emit(state.copyWith(status: ShareSheetStatus.signedOut));
      case Left():
        emit(state.copyWith(status: ShareSheetStatus.failed));
      case Right(value: final shared?):
        emit(state.copyWith(status: ShareSheetStatus.ready, info: shared));
      case Right():
        await _checkShareable(emit);
    }
  }

  /// A note that is not shared: whether it can be.
  Future<void> _checkShareable(Emitter<ShareNoteState> emit) async {
    final can = await _shared.canShare(state.noteId);
    emit(
      can.fold(
        (error) => switch (error) {
          LockedError() => state.copyWith(status: ShareSheetStatus.locked),
          SignedOutError() => state.copyWith(
            status: ShareSheetStatus.signedOut,
          ),
          InvalidError(:final message) => state.copyWith(
            status: ShareSheetStatus.notShareable,
            reason: message,
          ),
          _ => state.copyWith(status: ShareSheetStatus.failed),
        },
        (_) => state.copyWith(status: ShareSheetStatus.ready),
      ),
    );
  }

  /// Runs [work] on the shared note and shows its sharing as it is after.
  Future<void> _act(
    Emitter<ShareNoteState> emit,
    Future<Either<ApplicationError, ShareInfo>> Function(String noteId) work,
  ) async {
    if (state.info == null || state.busy) {
      return;
    }
    emit(state.copyWith(busy: true, problem: null, done: null));
    final result = await work(state.noteId);
    emit(
      result.fold(
        (error) => state.copyWith(busy: false, problem: shareProblemOf(error)),
        (info) => state.copyWith(busy: false, info: info, confirmEnd: false),
      ),
    );
  }

  Future<void> _onInvited(String username, Emitter<ShareNoteState> emit) async {
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
    final looked = await _shared.lookupPerson(name);
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
                prompt: SharePrompt.verify(person: person, role: role),
              ),
            );
          case PersonLookup_Changed(:final pinned, :final offered):
            emit(
              state.copyWith(
                busy: false,
                prompt: SharePrompt.keyChanged(
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

  /// Invites [username], whose keys are pinned: the first one makes the
  /// note shared.
  Future<void> _invite(
    String username,
    ShareRole role,
    Emitter<ShareNoteState> emit,
  ) async {
    emit(state.copyWith(busy: true, problem: null, prompt: null));
    final result = state.info == null
        ? await _shared.share(state.noteId, <(String, ShareRole)>[
            (username, role),
          ])
        : await _shared.invite(state.noteId, username, role);
    emit(
      result.fold(
        (error) => state.copyWith(busy: false, problem: shareProblemOf(error)),
        (info) => state.copyWith(
          busy: false,
          info: info,
          done: ShareDone.invited(username),
          confirmEnd: false,
        ),
      ),
    );
  }

  Future<void> _onFingerprintConfirmed(Emitter<ShareNoteState> emit) async {
    final prompt = state.prompt;
    if (prompt is! SharePromptVerify || state.busy) {
      return;
    }
    emit(state.copyWith(busy: true, problem: null));
    final verified = await _shared.verifyPerson(prompt.person.accountId);
    await verified.fold(
      (error) async =>
          emit(state.copyWith(busy: false, problem: shareProblemOf(error))),
      (_) => _invite(prompt.person.username, prompt.role, emit),
    );
  }

  Future<void> _onInvitedUnverified(Emitter<ShareNoteState> emit) async {
    final prompt = state.prompt;
    if (prompt is! SharePromptVerify || state.busy) {
      return;
    }
    await _invite(prompt.person.username, prompt.role, emit);
  }

  Future<void> _onNewKeyAccepted(Emitter<ShareNoteState> emit) async {
    final prompt = state.prompt;
    if (prompt is! SharePromptKeyChanged || state.busy) {
      return;
    }
    emit(state.copyWith(busy: true, problem: null));
    final pinned = await _shared.acceptNewKey(
      prompt.offered.username,
      prompt.offered.fingerprint,
    );
    await pinned.fold(
      (error) async =>
          emit(state.copyWith(busy: false, problem: shareProblemOf(error))),
      (person) async {
        final role = prompt.role;
        if (role != null) {
          // The new keys are trusted, not verified: their fingerprint is
          // compared next, as for someone new.
          emit(
            state.copyWith(
              busy: false,
              prompt: SharePrompt.verify(person: person, role: role),
            ),
          );
          return;
        }
        // A member of the note: its sharing, with their keys trusted.
        emit(state.copyWith(busy: false, prompt: null));
        final info = await _shared.info(state.noteId);
        emit(
          info.fold(
            (error) => state.copyWith(problem: shareProblemOf(error)),
            (shared) => shared == null ? state : state.copyWith(info: shared),
          ),
        );
      },
    );
  }

  Future<void> _onKeyReviewed(
    String username,
    Emitter<ShareNoteState> emit,
  ) async {
    if (state.busy) {
      return;
    }
    emit(state.copyWith(busy: true, problem: null, done: null));
    final looked = await _shared.lookupPerson(username);
    emit(
      looked.fold(
        (error) => state.copyWith(busy: false, problem: shareProblemOf(error)),
        (lookup) => switch (lookup) {
          PersonLookup_Changed(:final pinned, :final offered) => state.copyWith(
            busy: false,
            prompt: SharePrompt.keyChanged(offered: offered, pinned: pinned),
          ),
          // The keys are the pinned ones: nothing to review.
          PersonLookup_Found() => state.copyWith(busy: false),
        },
      ),
    );
  }

  Future<void> _onLinkMade(Emitter<ShareNoteState> emit) async {
    if (state.info == null || state.busy) {
      return;
    }
    emit(
      state.copyWith(
        busy: true,
        problem: null,
        done: null,
        linkCopied: false,
        madeLink: null,
      ),
    );
    final made = await _shared.createLink(
      state.noteId,
      state.linkRole,
      state.linkDays,
    );
    await made.fold(
      (error) async =>
          emit(state.copyWith(busy: false, problem: shareProblemOf(error))),
      (url) async {
        final info = await _shared.info(state.noteId);
        emit(
          state.copyWith(
            busy: false,
            madeLink: url,
            // The link stands even if reading the sharing again fails.
            info:
                info.fold((_) => state.info, (shared) => shared) ?? state.info,
          ),
        );
      },
    );
  }

  Future<void> _onLinkCopied(Emitter<ShareNoteState> emit) async {
    final link = state.madeLink;
    if (link == null) {
      return;
    }
    final copied = await _system.copy(link);
    emit(
      copied.fold(
        (_) => state.copyWith(problem: const ShareProblem.unexpected()),
        (_) => state.copyWith(linkCopied: true, problem: null),
      ),
    );
  }

  Future<void> _onLinkSent(Rect? origin, Emitter<ShareNoteState> emit) async {
    final link = state.madeLink;
    if (link == null) {
      return;
    }
    final sent = await _system.send(link, origin: origin);
    emit(
      sent.fold(
        (_) => state.copyWith(problem: const ShareProblem.unexpected()),
        (_) => state.copyWith(problem: null),
      ),
    );
  }

  Future<void> _onEndConfirmed(Emitter<ShareNoteState> emit) async {
    final info = state.info;
    if (info == null || state.busy || !state.confirmEnd) {
      return;
    }
    final owner = info.myRole == ShareRole.owner;
    emit(state.copyWith(busy: true, problem: null, done: null));
    final ended = owner
        ? await _shared.stop(state.noteId)
        : await _shared.leave(state.noteId);
    emit(
      ended.fold(
        (error) => state.copyWith(
          busy: false,
          confirmEnd: false,
          problem: shareProblemOf(error),
        ),
        (_) => state.copyWith(
          busy: false,
          confirmEnd: false,
          info: null,
          madeLink: null,
          linkCopied: false,
          done: owner ? const ShareDone.stopped() : const ShareDone.left(),
        ),
      ),
    );
  }
}

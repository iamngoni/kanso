// Kanso — shared_notebooks_event.dart
// Created 2026-11-13.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'shared_notebooks_bloc.dart';

@freezed
sealed class SharedNotebooksEvent with _$SharedNotebooksEvent {
  /// Read the section, then again as syncs report news.
  const factory SharedNotebooksEvent.started() = SharedNotebooksStartedEvent;

  /// Read it again: a sync or a notebook's room said something changed.
  const factory SharedNotebooksEvent.refreshed() =
      SharedNotebooksRefreshedEvent;

  /// Join the notebook of an invitation.
  const factory SharedNotebooksEvent.accepted(String shareId) =
      SharedNotebooksAcceptedEvent;

  const factory SharedNotebooksEvent.declined(String shareId) =
      SharedNotebooksDeclinedEvent;

  /// The first notice (or the problem, or the joined notebook) was shown.
  const factory SharedNotebooksEvent.noticeRead() =
      SharedNotebooksNoticeReadEvent;
}

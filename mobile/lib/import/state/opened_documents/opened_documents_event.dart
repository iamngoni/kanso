// Kanso: opened_documents_event.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'opened_documents_bloc.dart';

@freezed
sealed class OpenedDocumentsEvent with _$OpenedDocumentsEvent {
  /// The app started: listen for documents the system opens, and open any
  /// that already arrived.
  const factory OpenedDocumentsEvent.started() = OpenedDocumentsStartedEvent;

  /// The system opened another document in the app.
  const factory OpenedDocumentsEvent.arrived() = OpenedDocumentsArrivedEvent;

  /// The person has seen what came of opening them.
  const factory OpenedDocumentsEvent.dismissed() =
      OpenedDocumentsDismissedEvent;
}

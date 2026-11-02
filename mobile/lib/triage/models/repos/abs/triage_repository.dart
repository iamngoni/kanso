// Kanso — triage_repository.dart
// Created 2026-11-02.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:dartz/dartz.dart';

import '../../../../core/bridge/api/triage_types.dart';
import '../../../../core/models/data/application_error.dart';

/// Inbox triage over the open library: cards of suggestions for the Inbox's
/// notes, rejecting and editing chips, the model's ranking, filing what was
/// accepted, and the settings. Nothing changes a note but [file].
abstract interface class TriageRepository {
  /// How many notes wait, and what is switched on.
  Future<Either<ApplicationError, TriageStatus>> status();

  Future<Either<ApplicationError, TriageSettings>> settings();

  Future<Either<ApplicationError, TriageSettings>> setSettings(
    TriageSettings settings,
  );

  /// Cards for the notes Triage shows, newest first.
  Future<Either<ApplicationError, List<TriageCard>>> cards({
    required int limit,
  });

  /// The chips of the card for [noteId] ranked by the person's AI. Each
  /// cites notes; the error is the plain reason it is unavailable.
  Future<Either<ApplicationError, List<TriageChip>>> refine(String noteId);

  /// Remembers a rejection so the chip does not come back.
  Future<Either<ApplicationError, Unit>> reject(
    String noteId,
    TriageField field,
    String value,
  );

  /// Takes a rejection back.
  Future<Either<ApplicationError, Unit>> unreject(
    String noteId,
    TriageField field,
    String value,
  );

  /// The chip a typed value makes for [field], or why it cannot.
  Future<Either<ApplicationError, TriageChip>> resolve(
    TriageField field,
    String text,
  );

  /// What filing would do, without writing anything.
  Future<Either<ApplicationError, TriageFiled>> plan(
    List<TriageFiling> filings,
  );

  /// Files the accepted chips in one write. The ticket in the result takes
  /// the batch back (`LibraryRepository.undo`).
  Future<Either<ApplicationError, TriageFiled>> file(
    List<TriageFiling> filings,
  );

  /// Turns Related notes on, which Triage's notebook and tag suggestions
  /// need.
  Future<Either<ApplicationError, Unit>> turnOnRelatedNotes();

  /// The ids of the notes the library changed, as it changes.
  Stream<List<String>> changes();
}

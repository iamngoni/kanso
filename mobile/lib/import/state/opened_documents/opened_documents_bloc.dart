// Kanso: opened_documents_bloc.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../models/repos/abs/import_files_repository.dart';
import '../../models/repos/abs/import_repository.dart';

part 'opened_documents_bloc.freezed.dart';
part 'opened_documents_event.dart';
part 'opened_documents_state.dart';

/// Documents other apps open in Kanso (Markdown and text files).
///
/// The system's side queues each document it hands over; this takes the
/// queue, opens the documents into the Inbox as the desktop's File › Open
/// does (a document opened again updates its note), and says which note to
/// show. Every arrival is handled after the one before it finished.
@injectable
class OpenedDocumentsBloc
    extends Bloc<OpenedDocumentsEvent, OpenedDocumentsState> {
  OpenedDocumentsBloc(this._import, this._files)
    : super(const OpenedDocumentsState()) {
    on<OpenedDocumentsStartedEvent>(_onStarted);
    on<OpenedDocumentsArrivedEvent>(_onArrived, transformer: sequential());
    on<OpenedDocumentsDismissedEvent>(
      (event, emit) => emit(const OpenedDocumentsState()),
    );
  }

  final ImportRepository _import;
  final ImportFilesRepository _files;
  StreamSubscription<Object?>? _arrivals;

  void _onStarted(
    OpenedDocumentsStartedEvent event,
    Emitter<OpenedDocumentsState> emit,
  ) {
    _arrivals ??= _files.arrivals.listen(
      (_) => add(const OpenedDocumentsEvent.arrived()),
    );
    add(const OpenedDocumentsEvent.arrived());
  }

  Future<void> _onArrived(
    OpenedDocumentsArrivedEvent event,
    Emitter<OpenedDocumentsState> emit,
  ) async {
    final taken = await _files.takeOpened();
    final batch = taken.fold((_) => null, (batch) => batch);
    if (batch == null) {
      emit(const OpenedDocumentsState(failed: true));
      return;
    }
    if (batch.files.isEmpty) {
      return;
    }
    emit(state.copyWith(opening: true, noteId: null));
    final opened = await _import.openDocuments(batch.files, batch.staging);
    await _files.discardOpened();
    emit(
      opened.fold(
        (_) => const OpenedDocumentsState(failed: true),
        (documents) => OpenedDocumentsState(
          noteId: documents.noteId,
          problems: documents.problems,
        ),
      ),
    );
  }

  @override
  Future<void> close() async {
    await _arrivals?.cancel();
    return super.close();
  }
}

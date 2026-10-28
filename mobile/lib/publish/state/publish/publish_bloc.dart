// Kanso — publish_bloc.dart
// Created 2026-10-28.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/bridge/api/publish.dart';
import '../../../core/bridge/api/types.dart';
import '../../../core/models/data/application_error.dart';
import '../../models/data/publish_view_model.dart';
import '../../models/repos/abs/publish_repository.dart';

part 'publish_bloc.freezed.dart';
part 'publish_event.dart';
part 'publish_state.dart';

/// Note and notebook publishing, with the same visible states for both.
@injectable
class PublishBloc extends Bloc<PublishEvent, PublishState> {
  PublishBloc(this._repository) : super(const PublishState.idle()) {
    on<PublishOpenNoteEvent>(_openNote, transformer: restartable());
    on<PublishOpenNotebookEvent>(_openNotebook, transformer: restartable());
    on<PublishSubmitEvent>(_publish, transformer: droppable());
    on<PublishUnpublishEvent>(_unpublish, transformer: droppable());
    on<PublishExpiryPickedEvent>(_expiryPicked);
  }

  final PublishRepository _repository;
  PaletteDto _palette = const PaletteDto(
    dark: false,
    background: 0xfcfaf4,
    text: 0x26241f,
    muted: 0x857c6e,
    accent: 0x4d6684,
    border: 0xddd6c9,
    surface: 0xf4f1ea,
    quote: 0x655f55,
    taskDone: 0x655f55,
    inlineCodeBackground: 0xf4f1ea,
  );
  PreviewOptions _options = const PreviewOptions(
    size: 16,
    utcOffsetSecs: 0,
    code: <SyntaxStyleRow>[],
  );
  bool _notebook = false;

  Future<void> _openNote(
    PublishOpenNoteEvent event,
    Emitter<PublishState> emit,
  ) async {
    _notebook = false;
    _palette = event.palette;
    _options = event.options;
    final optionsResult = await _repository.expiryOptions();
    final expiryOptions = optionsResult.fold(
      (_) => <PublishExpiryOption>[],
      (value) => value,
    );
    final result = await _repository.note(event.id);
    // The two branches deliberately emit different union states.
    // ignore: cascade_invocations
    result.fold(
      (error) => emit(
        PublishState.failed(
          view: PublishViewModel(
            id: event.id,
            title: '',
            notebook: false,
            summary: '',
          ),
          text: _errorText(error),
        ),
      ),
      (note) {
        final view = PublishViewModel(
          id: note.id,
          title: note.title,
          notebook: false,
          summary: note.body,
          contentPreview: note.body,
          needsUpdate: note.needsUpdate,
          refusal: note.refusal,
          link: note.published,
          expiryOptions: expiryOptions,
        );
        emit(
          note.published == null
              ? PublishState.idle(view: view)
              : PublishState.published(view: view, link: note.published!),
        );
      },
    );
  }

  Future<void> _openNotebook(
    PublishOpenNotebookEvent event,
    Emitter<PublishState> emit,
  ) async {
    _notebook = true;
    _palette = event.palette;
    _options = event.options;
    final optionsResult = await _repository.expiryOptions();
    final expiryOptions = optionsResult.fold(
      (_) => <PublishExpiryOption>[],
      (value) => value,
    );
    final result = await _repository.notebook(event.id);
    // The two branches deliberately emit different union states.
    // ignore: cascade_invocations
    result.fold(
      (error) => emit(
        PublishState.failed(
          view: PublishViewModel(
            id: event.id,
            title: '',
            notebook: true,
            summary: '',
          ),
          text: _errorText(error),
        ),
      ),
      (notebook) {
        final view = PublishViewModel(
          id: notebook.id,
          title: notebook.name,
          notebook: true,
          summary: notebook.contents,
          locked: notebook.locked,
          refusal: notebook.refusal,
          link: notebook.published,
          needsUpdate: notebook.needsUpdate,
          expiryOptions: expiryOptions,
        );
        emit(
          notebook.published == null
              ? PublishState.idle(view: view)
              : PublishState.published(
                  view: view,
                  link: notebook.published!,
                ),
        );
      },
    );
  }

  Future<void> _publish(
    PublishSubmitEvent event,
    Emitter<PublishState> emit,
  ) async {
    final view = _view;
    if (view == null) {
      return;
    }
    if (view.refusal case final refusal?) {
      emit(PublishState.failed(view: view, text: refusal));
      return;
    }
    emit(PublishState.publishing(view: view));
    final result = _notebook
        ? await _repository.publishNotebook(
            id: view.id,
            expiry: event.expiry,
            palette: _palette,
            options: _options,
          )
        : await _repository.publishNote(
            id: view.id,
            expiry: event.expiry,
            palette: _palette,
            options: _options,
          );
    emit(
      result.fold<PublishState>(
        (error) => PublishState.failed(view: view, text: _errorText(error)),
        (link) => PublishState.published(view: view, link: link),
      ),
    );
  }

  Future<void> _unpublish(
    PublishUnpublishEvent event,
    Emitter<PublishState> emit,
  ) async {
    final view = _view;
    if (view == null) {
      return;
    }
    emit(PublishState.publishing(view: view));
    final result = _notebook
        ? await _repository.unpublishNotebook(view.id)
        : await _repository.unpublishNote(view.id);
    emit(
      result.fold<PublishState>(
        (error) => PublishState.failed(view: view, text: _errorText(error)),
        (_) => PublishState.idle(view: view.copyWith(link: null)),
      ),
    );
  }

  void _expiryPicked(
    PublishExpiryPickedEvent event,
    Emitter<PublishState> emit,
  ) {
    final view = _view;
    if (view == null) {
      return;
    }
    final updated = view.copyWith(expiry: event.expiry);
    emit(
      state.map(
        idle: (_) => PublishState.idle(view: updated),
        publishing: (_) => PublishState.publishing(view: updated),
        published: (value) =>
            PublishState.published(view: updated, link: value.link),
        failed: (value) => PublishState.failed(view: updated, text: value.text),
      ),
    );
  }

  PublishViewModel? get _view => switch (state) {
    PublishIdleState(:final view) => view,
    PublishPublishingState(:final view) => view,
    PublishPublishedState(:final view) => view,
    PublishFailedState(:final view) => view,
  };

  static String _errorText(ApplicationError error) => switch (error) {
    SignedOutError(:final message) =>
      message == 'Not signed in.' ? 'Sign in to publish.' : message,
    _ => error.message,
  };
}

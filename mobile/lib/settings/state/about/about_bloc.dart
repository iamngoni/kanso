// Kanso — about_bloc.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:bloc/bloc.dart';
import 'package:bloc_concurrency/bloc_concurrency.dart';
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:injectable/injectable.dart';

import '../../../core/models/data/app_version.dart';
import '../../../core/models/data/load_status.dart';
import '../../../core/models/repos/abs/about_repository.dart';

part 'about_bloc.freezed.dart';
part 'about_event.dart';
part 'about_state.dart';

/// What the About section shows: the version of the app, and the ways to
/// Kanso's website, release notes, help and store page.
@injectable
class AboutBloc extends Bloc<AboutEvent, AboutState> {
  AboutBloc(this._about) : super(const AboutState()) {
    on<AboutStartedEvent>(_onStarted, transformer: restartable());
    on<AboutWebsiteOpenedEvent>(
      (event, emit) => _about.openWebsite(),
      transformer: droppable(),
    );
    on<AboutReleaseNotesOpenedEvent>(
      (event, emit) => _about.openReleaseNotes(),
      transformer: droppable(),
    );
    on<AboutHelpOpenedEvent>(
      (event, emit) => _about.openHelp(),
      transformer: droppable(),
    );
    on<AboutRateOpenedEvent>(
      (event, emit) => _about.openRatePage(),
      transformer: droppable(),
    );
  }

  final AboutRepository _about;

  Future<void> _onStarted(
    AboutStartedEvent event,
    Emitter<AboutState> emit,
  ) async {
    emit(const AboutState());
    final result = await _about.version();
    emit(
      result.fold(
        (error) => const AboutState(status: LoadStatus.failed),
        (version) => AboutState(status: LoadStatus.ready, version: version),
      ),
    );
  }
}

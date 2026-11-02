// Kanso — about_event.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

part of 'about_bloc.dart';

@freezed
sealed class AboutEvent with _$AboutEvent {
  /// The section was opened: read the version.
  const factory AboutEvent.started() = AboutStartedEvent;

  /// The person chose the website row.
  const factory AboutEvent.websiteOpened() = AboutWebsiteOpenedEvent;

  /// The person chose the release notes row.
  const factory AboutEvent.releaseNotesOpened() = AboutReleaseNotesOpenedEvent;

  /// The person chose the help row.
  const factory AboutEvent.helpOpened() = AboutHelpOpenedEvent;

  /// The person chose to rate the app.
  const factory AboutEvent.rateOpened() = AboutRateOpenedEvent;
}

// Kanso — recording_audio_session.dart
// Created 2026-11-04.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:io';

import 'package:flutter/services.dart';
import 'package:injectable/injectable.dart';

/// The name of the channel the phones answer on for the microphone.
const String recordingChannelName = 'app.kanso.kanso/recording';

/// The phone's side of a recording (issue #38). The recorder is the Rust
/// core's, which can neither ask for the microphone nor, on iOS, set the
/// audio session to record. [prepare] does both before it starts: iOS asks
/// once and sets play-and-record, Android asks for RECORD_AUDIO. The native
/// halves are in `AppDelegate.swift` and `KansoRecordingSession.kt`.
@lazySingleton
class RecordingAudioSession {
  static const MethodChannel _channel = MethodChannel(recordingChannelName);

  /// Whether the microphone may be used. Off a phone, or on a host without
  /// the channel, the recorder is left to find out itself.
  Future<bool> prepare() async {
    if (!Platform.isIOS && !Platform.isAndroid) {
      return true;
    }
    try {
      return await _channel.invokeMethod<bool>('prepare') ?? false;
    } on MissingPluginException {
      return true;
    } on PlatformException {
      return false;
    }
  }

  /// Sets the phone up to play a recording in a note: iOS gets the playback
  /// category, so it is heard with the ringer switch off. Android needs
  /// nothing. False only when the session can't be set.
  Future<bool> playback() async {
    if (!Platform.isIOS) {
      return true;
    }
    try {
      return await _channel.invokeMethod<bool>('playback') ?? false;
    } on MissingPluginException {
      return true;
    } on PlatformException {
      return false;
    }
  }

  /// Gives the audio back to other apps once the recording has ended.
  Future<void> release() async {
    if (!Platform.isIOS && !Platform.isAndroid) {
      return;
    }
    try {
      await _channel.invokeMethod<void>('release');
    } on MissingPluginException {
      // Nothing was taken.
    } on PlatformException {
      // The session is the system's to end.
    }
  }
}

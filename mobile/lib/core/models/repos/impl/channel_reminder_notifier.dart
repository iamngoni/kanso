// Kanso — channel_reminder_notifier.dart
// Created 2026-10-27.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:async';

import 'package:flutter/services.dart';
import 'package:injectable/injectable.dart';

import '../../../bridge/api/types.dart';
import '../../../utils/reminder_channel.dart';
import '../../data/reminder_permission.dart';
import '../../data/reminder_response.dart';
import '../abs/reminder_notifier.dart';

/// The name of the method channel to the native side of reminders
/// (`KansoReminders.swift`, `KansoReminders.kt`).
const String remindersChannelName = 'app.kanso/reminders';

/// [ReminderNotifier] over the `app.kanso/reminders` method channel.
///
/// A platform without the channel (a test, the desktop) has no permission
/// and shows and schedules nothing, rather than failing.
@LazySingleton(as: ReminderNotifier)
class ChannelReminderNotifier implements ReminderNotifier {
  ChannelReminderNotifier()
    : _channel = const MethodChannel(remindersChannelName) {
    _channel.setMethodCallHandler(_onCall);
  }

  final MethodChannel _channel;
  // Native marks a response live only when it can send it over this channel.
  // A single-subscription controller buffers callbacks until the coordinator
  // subscribes, including a cold-start tap delivered during bootstrap.
  final StreamController<ReminderResponse> _responses =
      StreamController<ReminderResponse>();

  @override
  Stream<ReminderResponse> get responses => _responses.stream;

  Future<Object?> _onCall(MethodCall call) async {
    if (call.method == 'response') {
      final response = reminderResponseOf(call.arguments);
      if (response != null) {
        _responses.add(response);
      }
    }
    return null;
  }

  @override
  Future<ReminderPermission> permission() async {
    try {
      final answer = await _invoke<String>('permission');
      return ReminderPermission.parse(answer);
    } on PlatformException {
      return ReminderPermission.denied;
    }
  }

  @override
  Future<bool> requestPermission() async {
    try {
      return await _invoke<bool>('requestPermission') ?? false;
    } on PlatformException {
      return false;
    }
  }

  @override
  Future<void> schedule(List<ReminderNote> reminders) =>
      _invoke<void>('schedule', _maps(reminders));

  @override
  Future<void> show(List<ReminderNote> reminders) =>
      _invoke<void>('show', _maps(reminders));

  @override
  Future<void> cancelAll() => _invoke<void>('cancelAll');

  @override
  Future<List<ReminderResponse>> takeResponses() async {
    final answer = await _invoke<List<Object?>>('takeResponses');
    return <ReminderResponse>[
      for (final item in answer ?? const <Object?>[]) ?reminderResponseOf(item),
    ];
  }

  static List<Map<String, Object>> _maps(List<ReminderNote> reminders) =>
      <Map<String, Object>>[
        for (final reminder in reminders) reminderToChannel(reminder),
      ];

  Future<T?> _invoke<T>(String method, [Object? arguments]) async {
    try {
      return await _channel.invokeMethod<T>(method, arguments);
    } on MissingPluginException {
      return null;
    }
  }
}

// Kanso — system_calendar.dart
// Created 2026-11-06.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'package:flutter/services.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

part 'system_calendar.freezed.dart';

/// Read-only access to calendars the person has already added to iOS or
/// Android. Network calendar providers are deliberately not supported here.
class SystemCalendarGateway {
  SystemCalendarGateway({MethodChannel? channel})
    : _channel = channel ?? const MethodChannel(_channelName);

  static const String _channelName = 'app.kanso.kanso/system_calendar';
  final MethodChannel _channel;

  /// Ask the OS for calendar access after the Calendar settings page explains
  /// why Overview needs it. No event is read until this returns true.
  Future<bool> requestAccess() async =>
      await _channel.invokeMethod<bool>('requestAccess') ?? false;

  /// The calendars the system account has made visible.
  Future<List<SystemCalendar>> calendars() async {
    final rows =
        await _channel.invokeListMethod<Object?>('calendars') ??
        const <Object?>[];
    return rows
        .whereType<Map<Object?, Object?>>()
        .map(SystemCalendar.fromMap)
        .toList(growable: false);
  }

  Future<void> setCalendarEnabled({
    required String id,
    required bool enabled,
  }) => _channel.invokeMethod<void>(
    'setCalendarEnabled',
    <String, Object>{'id': id, 'enabled': enabled},
  );

  Future<void> openEvent({required int start}) => _channel.invokeMethod<void>(
    'openEvent',
    <String, Object>{'start': start},
  );

  /// Occurrences in `[start, end)`, with recurring events expanded by the OS.
  Future<List<SystemCalendarEvent>> events({
    required int start,
    required int end,
  }) async {
    if (start >= end) return const <SystemCalendarEvent>[];
    final rows =
        await _channel.invokeListMethod<Object?>(
          'events',
          <String, Object>{'start': start, 'end': end},
        ) ??
        const <Object?>[];
    return rows
        .whereType<Map<Object?, Object?>>()
        .map(SystemCalendarEvent.fromMap)
        .toList(growable: false);
  }
}

@freezed
abstract class SystemCalendar with _$SystemCalendar {
  const factory SystemCalendar({
    required String id,
    required String name,
    required String color,
    required bool enabled,
  }) = _SystemCalendar;

  factory SystemCalendar.fromMap(Map<Object?, Object?> map) => SystemCalendar(
    id: map['id'] as String? ?? '',
    name: map['name'] as String? ?? '',
    color: map['color'] as String? ?? '',
    enabled: map['enabled'] as bool? ?? false,
  );
}

@freezed
abstract class SystemCalendarEvent with _$SystemCalendarEvent {
  const factory SystemCalendarEvent({
    required String uid,
    required int start,
    required int end,
    required String title,
    required String location,
    required String calendarId,
    required String calendar,
    required String color,
    required String conferenceUrl,
    required List<SystemCalendarAttendee> attendees,
    required bool allDay,
    required bool free,
    required bool declined,
  }) = _SystemCalendarEvent;

  factory SystemCalendarEvent.fromMap(Map<Object?, Object?> map) =>
      SystemCalendarEvent(
        uid: map['uid'] as String? ?? '',
        start: (map['start'] as num?)?.toInt() ?? 0,
        end: (map['end'] as num?)?.toInt() ?? 0,
        title: map['title'] as String? ?? '',
        location: map['location'] as String? ?? '',
        calendarId: map['calendarId'] as String? ?? '',
        calendar: map['calendar'] as String? ?? '',
        color: map['color'] as String? ?? '',
        conferenceUrl: map['conferenceUrl'] as String? ?? '',
        attendees: (map['attendees'] as List<Object?>? ?? const <Object?>[])
            .whereType<Map<Object?, Object?>>()
            .map(SystemCalendarAttendee.fromMap)
            .toList(growable: false),
        allDay: map['allDay'] as bool? ?? false,
        free: map['free'] as bool? ?? false,
        declined: map['declined'] as bool? ?? false,
      );
}

@freezed
abstract class SystemCalendarAttendee with _$SystemCalendarAttendee {
  const factory SystemCalendarAttendee({
    required String name,
    required String email,
  }) = _SystemCalendarAttendee;

  factory SystemCalendarAttendee.fromMap(Map<Object?, Object?> map) =>
      SystemCalendarAttendee(
        name: map['name'] as String? ?? '',
        email: map['email'] as String? ?? '',
      );
}

extension SystemCalendarEventEngineMap on SystemCalendarEvent {
  Map<String, Object?> toEngineMap() => <String, Object?>{
    'uid': uid,
    'start': start,
    'end': end,
    'title': title,
    'location': location,
    'conference_url': conferenceUrl,
    'attendees': attendees.map((attendee) => attendee.toEngineMap()).toList(),
    'calendar': calendar,
    'calendar_id': calendarId,
    'color': color,
    'all_day': allDay,
    'free': free,
    'declined': declined,
  };
}

extension SystemCalendarAttendeeEngineMap on SystemCalendarAttendee {
  Map<String, String> toEngineMap() => <String, String>{
    'name': name,
    'email': email,
  };
}

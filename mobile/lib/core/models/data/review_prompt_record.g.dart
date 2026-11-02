// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'review_prompt_record.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_ReviewPromptRecord _$ReviewPromptRecordFromJson(Map<String, dynamic> json) =>
    _ReviewPromptRecord(
      firstOpenedAt: json['firstOpenedAt'] == null
          ? null
          : DateTime.parse(json['firstOpenedAt'] as String),
      activeDays: (json['activeDays'] as num?)?.toInt() ?? 0,
      lastActiveDay: json['lastActiveDay'] as String?,
      notesEdited: (json['notesEdited'] as num?)?.toInt() ?? 0,
      lastTroubleAt: json['lastTroubleAt'] == null
          ? null
          : DateTime.parse(json['lastTroubleAt'] as String),
      lastAskedAt: json['lastAskedAt'] == null
          ? null
          : DateTime.parse(json['lastAskedAt'] as String),
      askedVersions:
          (json['askedVersions'] as List<dynamic>?)
              ?.map((e) => e as String)
              .toList() ??
          const <String>[],
      firstSyncSeen: json['firstSyncSeen'] as bool? ?? false,
    );

Map<String, dynamic> _$ReviewPromptRecordToJson(_ReviewPromptRecord instance) =>
    <String, dynamic>{
      'firstOpenedAt': ?instance.firstOpenedAt?.toIso8601String(),
      'activeDays': instance.activeDays,
      'lastActiveDay': ?instance.lastActiveDay,
      'notesEdited': instance.notesEdited,
      'lastTroubleAt': ?instance.lastTroubleAt?.toIso8601String(),
      'lastAskedAt': ?instance.lastAskedAt?.toIso8601String(),
      'askedVersions': instance.askedVersions,
      'firstSyncSeen': instance.firstSyncSeen,
    };

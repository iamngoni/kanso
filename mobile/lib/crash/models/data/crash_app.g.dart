// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'crash_app.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_CrashApp _$CrashAppFromJson(Map<String, dynamic> json) => _CrashApp(
  version: json['version'] as String,
  build: json['build'] as String,
  patch: (json['patch'] as num?)?.toInt(),
);

Map<String, dynamic> _$CrashAppToJson(_CrashApp instance) => <String, dynamic>{
  'version': instance.version,
  'build': instance.build,
  'patch': instance.patch,
};

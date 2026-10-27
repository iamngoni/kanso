// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'theme_selection.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

_ThemeSelection _$ThemeSelectionFromJson(Map<String, dynamic> json) =>
    _ThemeSelection(
      followSystem: json['followSystem'] as bool? ?? true,
      ui: json['ui'] as String? ?? 'kanso-dark',
      light: json['light'] as String? ?? 'kanso-light',
      dark: json['dark'] as String? ?? 'kanso-dark',
      syntax: json['syntax'] as String?,
      preview: json['preview'] as String?,
    );

Map<String, dynamic> _$ThemeSelectionToJson(_ThemeSelection instance) =>
    <String, dynamic>{
      'followSystem': instance.followSystem,
      'ui': instance.ui,
      'light': instance.light,
      'dark': instance.dark,
      'syntax': ?instance.syntax,
      'preview': ?instance.preview,
    };

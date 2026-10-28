// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'theme_selection.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$ThemeSelection {

/// Whether the system's light or dark mode picks the theme.
 bool get followSystem;/// The theme shown while the system is not followed.
 String get ui;/// The theme shown in light mode while the system is followed.
 String get light;/// The theme shown in dark mode while the system is followed.
 String get dark;/// The theme that colours the Markdown source and the code in it; null
/// for the interface's own.
 String? get syntax;/// The theme that colours the rendered note; null for the interface's
/// own.
 String? get preview;
/// Create a copy of ThemeSelection
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemeSelectionCopyWith<ThemeSelection> get copyWith => _$ThemeSelectionCopyWithImpl<ThemeSelection>(this as ThemeSelection, _$identity);

  /// Serializes this ThemeSelection to a JSON map.
  Map<String, dynamic> toJson();


@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemeSelection&&(identical(other.followSystem, followSystem) || other.followSystem == followSystem)&&(identical(other.ui, ui) || other.ui == ui)&&(identical(other.light, light) || other.light == light)&&(identical(other.dark, dark) || other.dark == dark)&&(identical(other.syntax, syntax) || other.syntax == syntax)&&(identical(other.preview, preview) || other.preview == preview));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,followSystem,ui,light,dark,syntax,preview);

@override
String toString() {
  return 'ThemeSelection(followSystem: $followSystem, ui: $ui, light: $light, dark: $dark, syntax: $syntax, preview: $preview)';
}


}

/// @nodoc
abstract mixin class $ThemeSelectionCopyWith<$Res>  {
  factory $ThemeSelectionCopyWith(ThemeSelection value, $Res Function(ThemeSelection) _then) = _$ThemeSelectionCopyWithImpl;
@useResult
$Res call({
 bool followSystem, String ui, String light, String dark, String? syntax, String? preview
});




}
/// @nodoc
class _$ThemeSelectionCopyWithImpl<$Res>
    implements $ThemeSelectionCopyWith<$Res> {
  _$ThemeSelectionCopyWithImpl(this._self, this._then);

  final ThemeSelection _self;
  final $Res Function(ThemeSelection) _then;

/// Create a copy of ThemeSelection
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? followSystem = null,Object? ui = null,Object? light = null,Object? dark = null,Object? syntax = freezed,Object? preview = freezed,}) {
  return _then(_self.copyWith(
followSystem: null == followSystem ? _self.followSystem : followSystem // ignore: cast_nullable_to_non_nullable
as bool,ui: null == ui ? _self.ui : ui // ignore: cast_nullable_to_non_nullable
as String,light: null == light ? _self.light : light // ignore: cast_nullable_to_non_nullable
as String,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as String,syntax: freezed == syntax ? _self.syntax : syntax // ignore: cast_nullable_to_non_nullable
as String?,preview: freezed == preview ? _self.preview : preview // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [ThemeSelection].
extension ThemeSelectionPatterns on ThemeSelection {
/// A variant of `map` that fallback to returning `orElse`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ThemeSelection value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ThemeSelection() when $default != null:
return $default(_that);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// Callbacks receives the raw object, upcasted.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case final Subclass2 value:
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ThemeSelection value)  $default,){
final _that = this;
switch (_that) {
case _ThemeSelection():
return $default(_that);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `map` that fallback to returning `null`.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case final Subclass value:
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ThemeSelection value)?  $default,){
final _that = this;
switch (_that) {
case _ThemeSelection() when $default != null:
return $default(_that);case _:
  return null;

}
}
/// A variant of `when` that fallback to an `orElse` callback.
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return orElse();
/// }
/// ```

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool followSystem,  String ui,  String light,  String dark,  String? syntax,  String? preview)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ThemeSelection() when $default != null:
return $default(_that.followSystem,_that.ui,_that.light,_that.dark,_that.syntax,_that.preview);case _:
  return orElse();

}
}
/// A `switch`-like method, using callbacks.
///
/// As opposed to `map`, this offers destructuring.
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case Subclass2(:final field2):
///     return ...;
/// }
/// ```

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool followSystem,  String ui,  String light,  String dark,  String? syntax,  String? preview)  $default,) {final _that = this;
switch (_that) {
case _ThemeSelection():
return $default(_that.followSystem,_that.ui,_that.light,_that.dark,_that.syntax,_that.preview);case _:
  throw StateError('Unexpected subclass');

}
}
/// A variant of `when` that fallback to returning `null`
///
/// It is equivalent to doing:
/// ```dart
/// switch (sealedClass) {
///   case Subclass(:final field):
///     return ...;
///   case _:
///     return null;
/// }
/// ```

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool followSystem,  String ui,  String light,  String dark,  String? syntax,  String? preview)?  $default,) {final _that = this;
switch (_that) {
case _ThemeSelection() when $default != null:
return $default(_that.followSystem,_that.ui,_that.light,_that.dark,_that.syntax,_that.preview);case _:
  return null;

}
}

}

/// @nodoc
@JsonSerializable()

class _ThemeSelection extends ThemeSelection {
  const _ThemeSelection({this.followSystem = true, this.ui = 'kanso-dark', this.light = 'kanso-light', this.dark = 'kanso-dark', this.syntax, this.preview}): super._();
  factory _ThemeSelection.fromJson(Map<String, dynamic> json) => _$ThemeSelectionFromJson(json);

/// Whether the system's light or dark mode picks the theme.
@override@JsonKey() final  bool followSystem;
/// The theme shown while the system is not followed.
@override@JsonKey() final  String ui;
/// The theme shown in light mode while the system is followed.
@override@JsonKey() final  String light;
/// The theme shown in dark mode while the system is followed.
@override@JsonKey() final  String dark;
/// The theme that colours the Markdown source and the code in it; null
/// for the interface's own.
@override final  String? syntax;
/// The theme that colours the rendered note; null for the interface's
/// own.
@override final  String? preview;

/// Create a copy of ThemeSelection
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ThemeSelectionCopyWith<_ThemeSelection> get copyWith => __$ThemeSelectionCopyWithImpl<_ThemeSelection>(this, _$identity);

@override
Map<String, dynamic> toJson() {
  return _$ThemeSelectionToJson(this, );
}

@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ThemeSelection&&(identical(other.followSystem, followSystem) || other.followSystem == followSystem)&&(identical(other.ui, ui) || other.ui == ui)&&(identical(other.light, light) || other.light == light)&&(identical(other.dark, dark) || other.dark == dark)&&(identical(other.syntax, syntax) || other.syntax == syntax)&&(identical(other.preview, preview) || other.preview == preview));
}

@JsonKey(includeFromJson: false, includeToJson: false)
@override
int get hashCode => Object.hash(runtimeType,followSystem,ui,light,dark,syntax,preview);

@override
String toString() {
  return 'ThemeSelection(followSystem: $followSystem, ui: $ui, light: $light, dark: $dark, syntax: $syntax, preview: $preview)';
}


}

/// @nodoc
abstract mixin class _$ThemeSelectionCopyWith<$Res> implements $ThemeSelectionCopyWith<$Res> {
  factory _$ThemeSelectionCopyWith(_ThemeSelection value, $Res Function(_ThemeSelection) _then) = __$ThemeSelectionCopyWithImpl;
@override @useResult
$Res call({
 bool followSystem, String ui, String light, String dark, String? syntax, String? preview
});




}
/// @nodoc
class __$ThemeSelectionCopyWithImpl<$Res>
    implements _$ThemeSelectionCopyWith<$Res> {
  __$ThemeSelectionCopyWithImpl(this._self, this._then);

  final _ThemeSelection _self;
  final $Res Function(_ThemeSelection) _then;

/// Create a copy of ThemeSelection
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? followSystem = null,Object? ui = null,Object? light = null,Object? dark = null,Object? syntax = freezed,Object? preview = freezed,}) {
  return _then(_ThemeSelection(
followSystem: null == followSystem ? _self.followSystem : followSystem // ignore: cast_nullable_to_non_nullable
as bool,ui: null == ui ? _self.ui : ui // ignore: cast_nullable_to_non_nullable
as String,light: null == light ? _self.light : light // ignore: cast_nullable_to_non_nullable
as String,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as String,syntax: freezed == syntax ? _self.syntax : syntax // ignore: cast_nullable_to_non_nullable
as String?,preview: freezed == preview ? _self.preview : preview // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on

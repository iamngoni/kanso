// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_window_launch.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteWindowLaunch {

 String get noteId;/// The name of the `AppEnvironment` the main window runs in.
 String get environment;/// The library folder of the main window; empty for the app's own.
 String get libraryFolder;
/// Create a copy of NoteWindowLaunch
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteWindowLaunchCopyWith<NoteWindowLaunch> get copyWith => _$NoteWindowLaunchCopyWithImpl<NoteWindowLaunch>(this as NoteWindowLaunch, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteWindowLaunch&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.environment, environment) || other.environment == environment)&&(identical(other.libraryFolder, libraryFolder) || other.libraryFolder == libraryFolder));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,environment,libraryFolder);

@override
String toString() {
  return 'NoteWindowLaunch(noteId: $noteId, environment: $environment, libraryFolder: $libraryFolder)';
}


}

/// @nodoc
abstract mixin class $NoteWindowLaunchCopyWith<$Res>  {
  factory $NoteWindowLaunchCopyWith(NoteWindowLaunch value, $Res Function(NoteWindowLaunch) _then) = _$NoteWindowLaunchCopyWithImpl;
@useResult
$Res call({
 String noteId, String environment, String libraryFolder
});




}
/// @nodoc
class _$NoteWindowLaunchCopyWithImpl<$Res>
    implements $NoteWindowLaunchCopyWith<$Res> {
  _$NoteWindowLaunchCopyWithImpl(this._self, this._then);

  final NoteWindowLaunch _self;
  final $Res Function(NoteWindowLaunch) _then;

/// Create a copy of NoteWindowLaunch
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? environment = null,Object? libraryFolder = null,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,environment: null == environment ? _self.environment : environment // ignore: cast_nullable_to_non_nullable
as String,libraryFolder: null == libraryFolder ? _self.libraryFolder : libraryFolder // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [NoteWindowLaunch].
extension NoteWindowLaunchPatterns on NoteWindowLaunch {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteWindowLaunch value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteWindowLaunch() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteWindowLaunch value)  $default,){
final _that = this;
switch (_that) {
case _NoteWindowLaunch():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteWindowLaunch value)?  $default,){
final _that = this;
switch (_that) {
case _NoteWindowLaunch() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  String environment,  String libraryFolder)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteWindowLaunch() when $default != null:
return $default(_that.noteId,_that.environment,_that.libraryFolder);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  String environment,  String libraryFolder)  $default,) {final _that = this;
switch (_that) {
case _NoteWindowLaunch():
return $default(_that.noteId,_that.environment,_that.libraryFolder);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  String environment,  String libraryFolder)?  $default,) {final _that = this;
switch (_that) {
case _NoteWindowLaunch() when $default != null:
return $default(_that.noteId,_that.environment,_that.libraryFolder);case _:
  return null;

}
}

}

/// @nodoc


class _NoteWindowLaunch extends NoteWindowLaunch {
  const _NoteWindowLaunch({required this.noteId, required this.environment, this.libraryFolder = ''}): super._();
  

@override final  String noteId;
/// The name of the `AppEnvironment` the main window runs in.
@override final  String environment;
/// The library folder of the main window; empty for the app's own.
@override@JsonKey() final  String libraryFolder;

/// Create a copy of NoteWindowLaunch
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteWindowLaunchCopyWith<_NoteWindowLaunch> get copyWith => __$NoteWindowLaunchCopyWithImpl<_NoteWindowLaunch>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteWindowLaunch&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.environment, environment) || other.environment == environment)&&(identical(other.libraryFolder, libraryFolder) || other.libraryFolder == libraryFolder));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,environment,libraryFolder);

@override
String toString() {
  return 'NoteWindowLaunch(noteId: $noteId, environment: $environment, libraryFolder: $libraryFolder)';
}


}

/// @nodoc
abstract mixin class _$NoteWindowLaunchCopyWith<$Res> implements $NoteWindowLaunchCopyWith<$Res> {
  factory _$NoteWindowLaunchCopyWith(_NoteWindowLaunch value, $Res Function(_NoteWindowLaunch) _then) = __$NoteWindowLaunchCopyWithImpl;
@override @useResult
$Res call({
 String noteId, String environment, String libraryFolder
});




}
/// @nodoc
class __$NoteWindowLaunchCopyWithImpl<$Res>
    implements _$NoteWindowLaunchCopyWith<$Res> {
  __$NoteWindowLaunchCopyWithImpl(this._self, this._then);

  final _NoteWindowLaunch _self;
  final $Res Function(_NoteWindowLaunch) _then;

/// Create a copy of NoteWindowLaunch
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? environment = null,Object? libraryFolder = null,}) {
  return _then(_NoteWindowLaunch(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,environment: null == environment ? _self.environment : environment // ignore: cast_nullable_to_non_nullable
as String,libraryFolder: null == libraryFolder ? _self.libraryFolder : libraryFolder // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on

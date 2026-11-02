// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ask_time_note.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AskTimeNote {

/// "Written October 2026".
 String get label;/// Said when too few notes were written in the range; empty otherwise.
 String get relaxedLine;
/// Create a copy of AskTimeNote
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskTimeNoteCopyWith<AskTimeNote> get copyWith => _$AskTimeNoteCopyWithImpl<AskTimeNote>(this as AskTimeNote, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskTimeNote&&(identical(other.label, label) || other.label == label)&&(identical(other.relaxedLine, relaxedLine) || other.relaxedLine == relaxedLine));
}


@override
int get hashCode => Object.hash(runtimeType,label,relaxedLine);

@override
String toString() {
  return 'AskTimeNote(label: $label, relaxedLine: $relaxedLine)';
}


}

/// @nodoc
abstract mixin class $AskTimeNoteCopyWith<$Res>  {
  factory $AskTimeNoteCopyWith(AskTimeNote value, $Res Function(AskTimeNote) _then) = _$AskTimeNoteCopyWithImpl;
@useResult
$Res call({
 String label, String relaxedLine
});




}
/// @nodoc
class _$AskTimeNoteCopyWithImpl<$Res>
    implements $AskTimeNoteCopyWith<$Res> {
  _$AskTimeNoteCopyWithImpl(this._self, this._then);

  final AskTimeNote _self;
  final $Res Function(AskTimeNote) _then;

/// Create a copy of AskTimeNote
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? label = null,Object? relaxedLine = null,}) {
  return _then(_self.copyWith(
label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,relaxedLine: null == relaxedLine ? _self.relaxedLine : relaxedLine // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [AskTimeNote].
extension AskTimeNotePatterns on AskTimeNote {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AskTimeNote value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AskTimeNote() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AskTimeNote value)  $default,){
final _that = this;
switch (_that) {
case _AskTimeNote():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AskTimeNote value)?  $default,){
final _that = this;
switch (_that) {
case _AskTimeNote() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String label,  String relaxedLine)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AskTimeNote() when $default != null:
return $default(_that.label,_that.relaxedLine);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String label,  String relaxedLine)  $default,) {final _that = this;
switch (_that) {
case _AskTimeNote():
return $default(_that.label,_that.relaxedLine);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String label,  String relaxedLine)?  $default,) {final _that = this;
switch (_that) {
case _AskTimeNote() when $default != null:
return $default(_that.label,_that.relaxedLine);case _:
  return null;

}
}

}

/// @nodoc


class _AskTimeNote implements AskTimeNote {
  const _AskTimeNote({required this.label, this.relaxedLine = ''});
  

/// "Written October 2026".
@override final  String label;
/// Said when too few notes were written in the range; empty otherwise.
@override@JsonKey() final  String relaxedLine;

/// Create a copy of AskTimeNote
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AskTimeNoteCopyWith<_AskTimeNote> get copyWith => __$AskTimeNoteCopyWithImpl<_AskTimeNote>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AskTimeNote&&(identical(other.label, label) || other.label == label)&&(identical(other.relaxedLine, relaxedLine) || other.relaxedLine == relaxedLine));
}


@override
int get hashCode => Object.hash(runtimeType,label,relaxedLine);

@override
String toString() {
  return 'AskTimeNote(label: $label, relaxedLine: $relaxedLine)';
}


}

/// @nodoc
abstract mixin class _$AskTimeNoteCopyWith<$Res> implements $AskTimeNoteCopyWith<$Res> {
  factory _$AskTimeNoteCopyWith(_AskTimeNote value, $Res Function(_AskTimeNote) _then) = __$AskTimeNoteCopyWithImpl;
@override @useResult
$Res call({
 String label, String relaxedLine
});




}
/// @nodoc
class __$AskTimeNoteCopyWithImpl<$Res>
    implements _$AskTimeNoteCopyWith<$Res> {
  __$AskTimeNoteCopyWithImpl(this._self, this._then);

  final _AskTimeNote _self;
  final $Res Function(_AskTimeNote) _then;

/// Create a copy of AskTimeNote
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? label = null,Object? relaxedLine = null,}) {
  return _then(_AskTimeNote(
label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,relaxedLine: null == relaxedLine ? _self.relaxedLine : relaxedLine // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on

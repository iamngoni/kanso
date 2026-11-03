// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ask_lines.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AskLines {

 int get start; int get end;
/// Create a copy of AskLines
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskLinesCopyWith<AskLines> get copyWith => _$AskLinesCopyWithImpl<AskLines>(this as AskLines, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskLines&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end));
}


@override
int get hashCode => Object.hash(runtimeType,start,end);

@override
String toString() {
  return 'AskLines(start: $start, end: $end)';
}


}

/// @nodoc
abstract mixin class $AskLinesCopyWith<$Res>  {
  factory $AskLinesCopyWith(AskLines value, $Res Function(AskLines) _then) = _$AskLinesCopyWithImpl;
@useResult
$Res call({
 int start, int end
});




}
/// @nodoc
class _$AskLinesCopyWithImpl<$Res>
    implements $AskLinesCopyWith<$Res> {
  _$AskLinesCopyWithImpl(this._self, this._then);

  final AskLines _self;
  final $Res Function(AskLines) _then;

/// Create a copy of AskLines
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? start = null,Object? end = null,}) {
  return _then(_self.copyWith(
start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [AskLines].
extension AskLinesPatterns on AskLines {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AskLines value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AskLines() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AskLines value)  $default,){
final _that = this;
switch (_that) {
case _AskLines():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AskLines value)?  $default,){
final _that = this;
switch (_that) {
case _AskLines() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int start,  int end)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AskLines() when $default != null:
return $default(_that.start,_that.end);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int start,  int end)  $default,) {final _that = this;
switch (_that) {
case _AskLines():
return $default(_that.start,_that.end);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int start,  int end)?  $default,) {final _that = this;
switch (_that) {
case _AskLines() when $default != null:
return $default(_that.start,_that.end);case _:
  return null;

}
}

}

/// @nodoc


class _AskLines implements AskLines {
  const _AskLines({required this.start, required this.end});
  

@override final  int start;
@override final  int end;

/// Create a copy of AskLines
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AskLinesCopyWith<_AskLines> get copyWith => __$AskLinesCopyWithImpl<_AskLines>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AskLines&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end));
}


@override
int get hashCode => Object.hash(runtimeType,start,end);

@override
String toString() {
  return 'AskLines(start: $start, end: $end)';
}


}

/// @nodoc
abstract mixin class _$AskLinesCopyWith<$Res> implements $AskLinesCopyWith<$Res> {
  factory _$AskLinesCopyWith(_AskLines value, $Res Function(_AskLines) _then) = __$AskLinesCopyWithImpl;
@override @useResult
$Res call({
 int start, int end
});




}
/// @nodoc
class __$AskLinesCopyWithImpl<$Res>
    implements _$AskLinesCopyWith<$Res> {
  __$AskLinesCopyWithImpl(this._self, this._then);

  final _AskLines _self;
  final $Res Function(_AskLines) _then;

/// Create a copy of AskLines
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? start = null,Object? end = null,}) {
  return _then(_AskLines(
start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on

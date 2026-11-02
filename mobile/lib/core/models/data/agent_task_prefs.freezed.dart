// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'agent_task_prefs.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AgentTaskPrefs {

 bool get enabled; bool get newNotes; bool get edits; bool get moves; bool get merges;
/// Create a copy of AgentTaskPrefs
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentTaskPrefsCopyWith<AgentTaskPrefs> get copyWith => _$AgentTaskPrefsCopyWithImpl<AgentTaskPrefs>(this as AgentTaskPrefs, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTaskPrefs&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.newNotes, newNotes) || other.newNotes == newNotes)&&(identical(other.edits, edits) || other.edits == edits)&&(identical(other.moves, moves) || other.moves == moves)&&(identical(other.merges, merges) || other.merges == merges));
}


@override
int get hashCode => Object.hash(runtimeType,enabled,newNotes,edits,moves,merges);

@override
String toString() {
  return 'AgentTaskPrefs(enabled: $enabled, newNotes: $newNotes, edits: $edits, moves: $moves, merges: $merges)';
}


}

/// @nodoc
abstract mixin class $AgentTaskPrefsCopyWith<$Res>  {
  factory $AgentTaskPrefsCopyWith(AgentTaskPrefs value, $Res Function(AgentTaskPrefs) _then) = _$AgentTaskPrefsCopyWithImpl;
@useResult
$Res call({
 bool enabled, bool newNotes, bool edits, bool moves, bool merges
});




}
/// @nodoc
class _$AgentTaskPrefsCopyWithImpl<$Res>
    implements $AgentTaskPrefsCopyWith<$Res> {
  _$AgentTaskPrefsCopyWithImpl(this._self, this._then);

  final AgentTaskPrefs _self;
  final $Res Function(AgentTaskPrefs) _then;

/// Create a copy of AgentTaskPrefs
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? enabled = null,Object? newNotes = null,Object? edits = null,Object? moves = null,Object? merges = null,}) {
  return _then(_self.copyWith(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,newNotes: null == newNotes ? _self.newNotes : newNotes // ignore: cast_nullable_to_non_nullable
as bool,edits: null == edits ? _self.edits : edits // ignore: cast_nullable_to_non_nullable
as bool,moves: null == moves ? _self.moves : moves // ignore: cast_nullable_to_non_nullable
as bool,merges: null == merges ? _self.merges : merges // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [AgentTaskPrefs].
extension AgentTaskPrefsPatterns on AgentTaskPrefs {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AgentTaskPrefs value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AgentTaskPrefs() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AgentTaskPrefs value)  $default,){
final _that = this;
switch (_that) {
case _AgentTaskPrefs():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AgentTaskPrefs value)?  $default,){
final _that = this;
switch (_that) {
case _AgentTaskPrefs() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool enabled,  bool newNotes,  bool edits,  bool moves,  bool merges)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AgentTaskPrefs() when $default != null:
return $default(_that.enabled,_that.newNotes,_that.edits,_that.moves,_that.merges);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool enabled,  bool newNotes,  bool edits,  bool moves,  bool merges)  $default,) {final _that = this;
switch (_that) {
case _AgentTaskPrefs():
return $default(_that.enabled,_that.newNotes,_that.edits,_that.moves,_that.merges);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool enabled,  bool newNotes,  bool edits,  bool moves,  bool merges)?  $default,) {final _that = this;
switch (_that) {
case _AgentTaskPrefs() when $default != null:
return $default(_that.enabled,_that.newNotes,_that.edits,_that.moves,_that.merges);case _:
  return null;

}
}

}

/// @nodoc


class _AgentTaskPrefs implements AgentTaskPrefs {
  const _AgentTaskPrefs({this.enabled = true, this.newNotes = true, this.edits = true, this.moves = true, this.merges = true});
  

@override@JsonKey() final  bool enabled;
@override@JsonKey() final  bool newNotes;
@override@JsonKey() final  bool edits;
@override@JsonKey() final  bool moves;
@override@JsonKey() final  bool merges;

/// Create a copy of AgentTaskPrefs
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AgentTaskPrefsCopyWith<_AgentTaskPrefs> get copyWith => __$AgentTaskPrefsCopyWithImpl<_AgentTaskPrefs>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AgentTaskPrefs&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.newNotes, newNotes) || other.newNotes == newNotes)&&(identical(other.edits, edits) || other.edits == edits)&&(identical(other.moves, moves) || other.moves == moves)&&(identical(other.merges, merges) || other.merges == merges));
}


@override
int get hashCode => Object.hash(runtimeType,enabled,newNotes,edits,moves,merges);

@override
String toString() {
  return 'AgentTaskPrefs(enabled: $enabled, newNotes: $newNotes, edits: $edits, moves: $moves, merges: $merges)';
}


}

/// @nodoc
abstract mixin class _$AgentTaskPrefsCopyWith<$Res> implements $AgentTaskPrefsCopyWith<$Res> {
  factory _$AgentTaskPrefsCopyWith(_AgentTaskPrefs value, $Res Function(_AgentTaskPrefs) _then) = __$AgentTaskPrefsCopyWithImpl;
@override @useResult
$Res call({
 bool enabled, bool newNotes, bool edits, bool moves, bool merges
});




}
/// @nodoc
class __$AgentTaskPrefsCopyWithImpl<$Res>
    implements _$AgentTaskPrefsCopyWith<$Res> {
  __$AgentTaskPrefsCopyWithImpl(this._self, this._then);

  final _AgentTaskPrefs _self;
  final $Res Function(_AgentTaskPrefs) _then;

/// Create a copy of AgentTaskPrefs
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? enabled = null,Object? newNotes = null,Object? edits = null,Object? moves = null,Object? merges = null,}) {
  return _then(_AgentTaskPrefs(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,newNotes: null == newNotes ? _self.newNotes : newNotes // ignore: cast_nullable_to_non_nullable
as bool,edits: null == edits ? _self.edits : edits // ignore: cast_nullable_to_non_nullable
as bool,moves: null == moves ? _self.moves : moves // ignore: cast_nullable_to_non_nullable
as bool,merges: null == merges ? _self.merges : merges // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on

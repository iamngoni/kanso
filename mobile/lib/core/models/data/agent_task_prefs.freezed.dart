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

 bool get enabled; bool get newNotes; bool get edits; bool get moves; bool get merges;/// The most minutes one task may run: 5, 20 or 60.
 int get timeLimitMinutes;/// The most notes one task may read: 50, 200 or 1000.
 int get maxNotes;/// Ask before a task that may read every notebook starts.
 bool get askBeforeReadingAll;
/// Create a copy of AgentTaskPrefs
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentTaskPrefsCopyWith<AgentTaskPrefs> get copyWith => _$AgentTaskPrefsCopyWithImpl<AgentTaskPrefs>(this as AgentTaskPrefs, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTaskPrefs&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.newNotes, newNotes) || other.newNotes == newNotes)&&(identical(other.edits, edits) || other.edits == edits)&&(identical(other.moves, moves) || other.moves == moves)&&(identical(other.merges, merges) || other.merges == merges)&&(identical(other.timeLimitMinutes, timeLimitMinutes) || other.timeLimitMinutes == timeLimitMinutes)&&(identical(other.maxNotes, maxNotes) || other.maxNotes == maxNotes)&&(identical(other.askBeforeReadingAll, askBeforeReadingAll) || other.askBeforeReadingAll == askBeforeReadingAll));
}


@override
int get hashCode => Object.hash(runtimeType,enabled,newNotes,edits,moves,merges,timeLimitMinutes,maxNotes,askBeforeReadingAll);

@override
String toString() {
  return 'AgentTaskPrefs(enabled: $enabled, newNotes: $newNotes, edits: $edits, moves: $moves, merges: $merges, timeLimitMinutes: $timeLimitMinutes, maxNotes: $maxNotes, askBeforeReadingAll: $askBeforeReadingAll)';
}


}

/// @nodoc
abstract mixin class $AgentTaskPrefsCopyWith<$Res>  {
  factory $AgentTaskPrefsCopyWith(AgentTaskPrefs value, $Res Function(AgentTaskPrefs) _then) = _$AgentTaskPrefsCopyWithImpl;
@useResult
$Res call({
 bool enabled, bool newNotes, bool edits, bool moves, bool merges, int timeLimitMinutes, int maxNotes, bool askBeforeReadingAll
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
@pragma('vm:prefer-inline') @override $Res call({Object? enabled = null,Object? newNotes = null,Object? edits = null,Object? moves = null,Object? merges = null,Object? timeLimitMinutes = null,Object? maxNotes = null,Object? askBeforeReadingAll = null,}) {
  return _then(_self.copyWith(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,newNotes: null == newNotes ? _self.newNotes : newNotes // ignore: cast_nullable_to_non_nullable
as bool,edits: null == edits ? _self.edits : edits // ignore: cast_nullable_to_non_nullable
as bool,moves: null == moves ? _self.moves : moves // ignore: cast_nullable_to_non_nullable
as bool,merges: null == merges ? _self.merges : merges // ignore: cast_nullable_to_non_nullable
as bool,timeLimitMinutes: null == timeLimitMinutes ? _self.timeLimitMinutes : timeLimitMinutes // ignore: cast_nullable_to_non_nullable
as int,maxNotes: null == maxNotes ? _self.maxNotes : maxNotes // ignore: cast_nullable_to_non_nullable
as int,askBeforeReadingAll: null == askBeforeReadingAll ? _self.askBeforeReadingAll : askBeforeReadingAll // ignore: cast_nullable_to_non_nullable
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool enabled,  bool newNotes,  bool edits,  bool moves,  bool merges,  int timeLimitMinutes,  int maxNotes,  bool askBeforeReadingAll)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AgentTaskPrefs() when $default != null:
return $default(_that.enabled,_that.newNotes,_that.edits,_that.moves,_that.merges,_that.timeLimitMinutes,_that.maxNotes,_that.askBeforeReadingAll);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool enabled,  bool newNotes,  bool edits,  bool moves,  bool merges,  int timeLimitMinutes,  int maxNotes,  bool askBeforeReadingAll)  $default,) {final _that = this;
switch (_that) {
case _AgentTaskPrefs():
return $default(_that.enabled,_that.newNotes,_that.edits,_that.moves,_that.merges,_that.timeLimitMinutes,_that.maxNotes,_that.askBeforeReadingAll);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool enabled,  bool newNotes,  bool edits,  bool moves,  bool merges,  int timeLimitMinutes,  int maxNotes,  bool askBeforeReadingAll)?  $default,) {final _that = this;
switch (_that) {
case _AgentTaskPrefs() when $default != null:
return $default(_that.enabled,_that.newNotes,_that.edits,_that.moves,_that.merges,_that.timeLimitMinutes,_that.maxNotes,_that.askBeforeReadingAll);case _:
  return null;

}
}

}

/// @nodoc


class _AgentTaskPrefs implements AgentTaskPrefs {
  const _AgentTaskPrefs({this.enabled = true, this.newNotes = true, this.edits = true, this.moves = true, this.merges = true, this.timeLimitMinutes = 20, this.maxNotes = 200, this.askBeforeReadingAll = true});
  

@override@JsonKey() final  bool enabled;
@override@JsonKey() final  bool newNotes;
@override@JsonKey() final  bool edits;
@override@JsonKey() final  bool moves;
@override@JsonKey() final  bool merges;
/// The most minutes one task may run: 5, 20 or 60.
@override@JsonKey() final  int timeLimitMinutes;
/// The most notes one task may read: 50, 200 or 1000.
@override@JsonKey() final  int maxNotes;
/// Ask before a task that may read every notebook starts.
@override@JsonKey() final  bool askBeforeReadingAll;

/// Create a copy of AgentTaskPrefs
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AgentTaskPrefsCopyWith<_AgentTaskPrefs> get copyWith => __$AgentTaskPrefsCopyWithImpl<_AgentTaskPrefs>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AgentTaskPrefs&&(identical(other.enabled, enabled) || other.enabled == enabled)&&(identical(other.newNotes, newNotes) || other.newNotes == newNotes)&&(identical(other.edits, edits) || other.edits == edits)&&(identical(other.moves, moves) || other.moves == moves)&&(identical(other.merges, merges) || other.merges == merges)&&(identical(other.timeLimitMinutes, timeLimitMinutes) || other.timeLimitMinutes == timeLimitMinutes)&&(identical(other.maxNotes, maxNotes) || other.maxNotes == maxNotes)&&(identical(other.askBeforeReadingAll, askBeforeReadingAll) || other.askBeforeReadingAll == askBeforeReadingAll));
}


@override
int get hashCode => Object.hash(runtimeType,enabled,newNotes,edits,moves,merges,timeLimitMinutes,maxNotes,askBeforeReadingAll);

@override
String toString() {
  return 'AgentTaskPrefs(enabled: $enabled, newNotes: $newNotes, edits: $edits, moves: $moves, merges: $merges, timeLimitMinutes: $timeLimitMinutes, maxNotes: $maxNotes, askBeforeReadingAll: $askBeforeReadingAll)';
}


}

/// @nodoc
abstract mixin class _$AgentTaskPrefsCopyWith<$Res> implements $AgentTaskPrefsCopyWith<$Res> {
  factory _$AgentTaskPrefsCopyWith(_AgentTaskPrefs value, $Res Function(_AgentTaskPrefs) _then) = __$AgentTaskPrefsCopyWithImpl;
@override @useResult
$Res call({
 bool enabled, bool newNotes, bool edits, bool moves, bool merges, int timeLimitMinutes, int maxNotes, bool askBeforeReadingAll
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
@override @pragma('vm:prefer-inline') $Res call({Object? enabled = null,Object? newNotes = null,Object? edits = null,Object? moves = null,Object? merges = null,Object? timeLimitMinutes = null,Object? maxNotes = null,Object? askBeforeReadingAll = null,}) {
  return _then(_AgentTaskPrefs(
enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,newNotes: null == newNotes ? _self.newNotes : newNotes // ignore: cast_nullable_to_non_nullable
as bool,edits: null == edits ? _self.edits : edits // ignore: cast_nullable_to_non_nullable
as bool,moves: null == moves ? _self.moves : moves // ignore: cast_nullable_to_non_nullable
as bool,merges: null == merges ? _self.merges : merges // ignore: cast_nullable_to_non_nullable
as bool,timeLimitMinutes: null == timeLimitMinutes ? _self.timeLimitMinutes : timeLimitMinutes // ignore: cast_nullable_to_non_nullable
as int,maxNotes: null == maxNotes ? _self.maxNotes : maxNotes // ignore: cast_nullable_to_non_nullable
as int,askBeforeReadingAll: null == askBeforeReadingAll ? _self.askBeforeReadingAll : askBeforeReadingAll // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on

// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'agent_task.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AgentTask {

 int get id; String get goal; AgentTaskPhase get phase;/// The question, the error or the number of changes, by [phase].
 String get detail;/// The live line: "Reading Q3 planning".
 String get line; String get lastMessage; int get notesRead; int get changes; int get elapsedSeconds;
/// Create a copy of AgentTask
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentTaskCopyWith<AgentTask> get copyWith => _$AgentTaskCopyWithImpl<AgentTask>(this as AgentTask, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTask&&(identical(other.id, id) || other.id == id)&&(identical(other.goal, goal) || other.goal == goal)&&(identical(other.phase, phase) || other.phase == phase)&&(identical(other.detail, detail) || other.detail == detail)&&(identical(other.line, line) || other.line == line)&&(identical(other.lastMessage, lastMessage) || other.lastMessage == lastMessage)&&(identical(other.notesRead, notesRead) || other.notesRead == notesRead)&&(identical(other.changes, changes) || other.changes == changes)&&(identical(other.elapsedSeconds, elapsedSeconds) || other.elapsedSeconds == elapsedSeconds));
}


@override
int get hashCode => Object.hash(runtimeType,id,goal,phase,detail,line,lastMessage,notesRead,changes,elapsedSeconds);

@override
String toString() {
  return 'AgentTask(id: $id, goal: $goal, phase: $phase, detail: $detail, line: $line, lastMessage: $lastMessage, notesRead: $notesRead, changes: $changes, elapsedSeconds: $elapsedSeconds)';
}


}

/// @nodoc
abstract mixin class $AgentTaskCopyWith<$Res>  {
  factory $AgentTaskCopyWith(AgentTask value, $Res Function(AgentTask) _then) = _$AgentTaskCopyWithImpl;
@useResult
$Res call({
 int id, String goal, AgentTaskPhase phase, String detail, String line, String lastMessage, int notesRead, int changes, int elapsedSeconds
});




}
/// @nodoc
class _$AgentTaskCopyWithImpl<$Res>
    implements $AgentTaskCopyWith<$Res> {
  _$AgentTaskCopyWithImpl(this._self, this._then);

  final AgentTask _self;
  final $Res Function(AgentTask) _then;

/// Create a copy of AgentTask
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? goal = null,Object? phase = null,Object? detail = null,Object? line = null,Object? lastMessage = null,Object? notesRead = null,Object? changes = null,Object? elapsedSeconds = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as int,goal: null == goal ? _self.goal : goal // ignore: cast_nullable_to_non_nullable
as String,phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as AgentTaskPhase,detail: null == detail ? _self.detail : detail // ignore: cast_nullable_to_non_nullable
as String,line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as String,lastMessage: null == lastMessage ? _self.lastMessage : lastMessage // ignore: cast_nullable_to_non_nullable
as String,notesRead: null == notesRead ? _self.notesRead : notesRead // ignore: cast_nullable_to_non_nullable
as int,changes: null == changes ? _self.changes : changes // ignore: cast_nullable_to_non_nullable
as int,elapsedSeconds: null == elapsedSeconds ? _self.elapsedSeconds : elapsedSeconds // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [AgentTask].
extension AgentTaskPatterns on AgentTask {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AgentTask value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AgentTask() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AgentTask value)  $default,){
final _that = this;
switch (_that) {
case _AgentTask():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AgentTask value)?  $default,){
final _that = this;
switch (_that) {
case _AgentTask() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int id,  String goal,  AgentTaskPhase phase,  String detail,  String line,  String lastMessage,  int notesRead,  int changes,  int elapsedSeconds)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AgentTask() when $default != null:
return $default(_that.id,_that.goal,_that.phase,_that.detail,_that.line,_that.lastMessage,_that.notesRead,_that.changes,_that.elapsedSeconds);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int id,  String goal,  AgentTaskPhase phase,  String detail,  String line,  String lastMessage,  int notesRead,  int changes,  int elapsedSeconds)  $default,) {final _that = this;
switch (_that) {
case _AgentTask():
return $default(_that.id,_that.goal,_that.phase,_that.detail,_that.line,_that.lastMessage,_that.notesRead,_that.changes,_that.elapsedSeconds);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int id,  String goal,  AgentTaskPhase phase,  String detail,  String line,  String lastMessage,  int notesRead,  int changes,  int elapsedSeconds)?  $default,) {final _that = this;
switch (_that) {
case _AgentTask() when $default != null:
return $default(_that.id,_that.goal,_that.phase,_that.detail,_that.line,_that.lastMessage,_that.notesRead,_that.changes,_that.elapsedSeconds);case _:
  return null;

}
}

}

/// @nodoc


class _AgentTask extends AgentTask {
  const _AgentTask({required this.id, required this.goal, required this.phase, this.detail = '', this.line = '', this.lastMessage = '', this.notesRead = 0, this.changes = 0, this.elapsedSeconds = 0}): super._();
  

@override final  int id;
@override final  String goal;
@override final  AgentTaskPhase phase;
/// The question, the error or the number of changes, by [phase].
@override@JsonKey() final  String detail;
/// The live line: "Reading Q3 planning".
@override@JsonKey() final  String line;
@override@JsonKey() final  String lastMessage;
@override@JsonKey() final  int notesRead;
@override@JsonKey() final  int changes;
@override@JsonKey() final  int elapsedSeconds;

/// Create a copy of AgentTask
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AgentTaskCopyWith<_AgentTask> get copyWith => __$AgentTaskCopyWithImpl<_AgentTask>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AgentTask&&(identical(other.id, id) || other.id == id)&&(identical(other.goal, goal) || other.goal == goal)&&(identical(other.phase, phase) || other.phase == phase)&&(identical(other.detail, detail) || other.detail == detail)&&(identical(other.line, line) || other.line == line)&&(identical(other.lastMessage, lastMessage) || other.lastMessage == lastMessage)&&(identical(other.notesRead, notesRead) || other.notesRead == notesRead)&&(identical(other.changes, changes) || other.changes == changes)&&(identical(other.elapsedSeconds, elapsedSeconds) || other.elapsedSeconds == elapsedSeconds));
}


@override
int get hashCode => Object.hash(runtimeType,id,goal,phase,detail,line,lastMessage,notesRead,changes,elapsedSeconds);

@override
String toString() {
  return 'AgentTask(id: $id, goal: $goal, phase: $phase, detail: $detail, line: $line, lastMessage: $lastMessage, notesRead: $notesRead, changes: $changes, elapsedSeconds: $elapsedSeconds)';
}


}

/// @nodoc
abstract mixin class _$AgentTaskCopyWith<$Res> implements $AgentTaskCopyWith<$Res> {
  factory _$AgentTaskCopyWith(_AgentTask value, $Res Function(_AgentTask) _then) = __$AgentTaskCopyWithImpl;
@override @useResult
$Res call({
 int id, String goal, AgentTaskPhase phase, String detail, String line, String lastMessage, int notesRead, int changes, int elapsedSeconds
});




}
/// @nodoc
class __$AgentTaskCopyWithImpl<$Res>
    implements _$AgentTaskCopyWith<$Res> {
  __$AgentTaskCopyWithImpl(this._self, this._then);

  final _AgentTask _self;
  final $Res Function(_AgentTask) _then;

/// Create a copy of AgentTask
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? goal = null,Object? phase = null,Object? detail = null,Object? line = null,Object? lastMessage = null,Object? notesRead = null,Object? changes = null,Object? elapsedSeconds = null,}) {
  return _then(_AgentTask(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as int,goal: null == goal ? _self.goal : goal // ignore: cast_nullable_to_non_nullable
as String,phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as AgentTaskPhase,detail: null == detail ? _self.detail : detail // ignore: cast_nullable_to_non_nullable
as String,line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as String,lastMessage: null == lastMessage ? _self.lastMessage : lastMessage // ignore: cast_nullable_to_non_nullable
as String,notesRead: null == notesRead ? _self.notesRead : notesRead // ignore: cast_nullable_to_non_nullable
as int,changes: null == changes ? _self.changes : changes // ignore: cast_nullable_to_non_nullable
as int,elapsedSeconds: null == elapsedSeconds ? _self.elapsedSeconds : elapsedSeconds // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on

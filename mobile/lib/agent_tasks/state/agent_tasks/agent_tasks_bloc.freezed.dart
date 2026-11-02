// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'agent_tasks_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AgentTasksEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTasksEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentTasksEvent()';
}


}

/// @nodoc
class $AgentTasksEventCopyWith<$Res>  {
$AgentTasksEventCopyWith(AgentTasksEvent _, $Res Function(AgentTasksEvent) __);
}


/// Adds pattern-matching-related methods to [AgentTasksEvent].
extension AgentTasksEventPatterns on AgentTasksEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AgentTasksStartedEvent value)?  started,TResult Function( AgentTasksPolledEvent value)?  polled,TResult Function( AgentTasksGoalSentEvent value)?  goalSent,TResult Function( AgentTasksStoppedEvent value)?  stopped,TResult Function( AgentTasksRepliedEvent value)?  replied,TResult Function( AgentTasksDismissedEvent value)?  dismissed,TResult Function( AgentTasksTroubleClearedEvent value)?  troubleCleared,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AgentTasksStartedEvent() when started != null:
return started(_that);case AgentTasksPolledEvent() when polled != null:
return polled(_that);case AgentTasksGoalSentEvent() when goalSent != null:
return goalSent(_that);case AgentTasksStoppedEvent() when stopped != null:
return stopped(_that);case AgentTasksRepliedEvent() when replied != null:
return replied(_that);case AgentTasksDismissedEvent() when dismissed != null:
return dismissed(_that);case AgentTasksTroubleClearedEvent() when troubleCleared != null:
return troubleCleared(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AgentTasksStartedEvent value)  started,required TResult Function( AgentTasksPolledEvent value)  polled,required TResult Function( AgentTasksGoalSentEvent value)  goalSent,required TResult Function( AgentTasksStoppedEvent value)  stopped,required TResult Function( AgentTasksRepliedEvent value)  replied,required TResult Function( AgentTasksDismissedEvent value)  dismissed,required TResult Function( AgentTasksTroubleClearedEvent value)  troubleCleared,}){
final _that = this;
switch (_that) {
case AgentTasksStartedEvent():
return started(_that);case AgentTasksPolledEvent():
return polled(_that);case AgentTasksGoalSentEvent():
return goalSent(_that);case AgentTasksStoppedEvent():
return stopped(_that);case AgentTasksRepliedEvent():
return replied(_that);case AgentTasksDismissedEvent():
return dismissed(_that);case AgentTasksTroubleClearedEvent():
return troubleCleared(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AgentTasksStartedEvent value)?  started,TResult? Function( AgentTasksPolledEvent value)?  polled,TResult? Function( AgentTasksGoalSentEvent value)?  goalSent,TResult? Function( AgentTasksStoppedEvent value)?  stopped,TResult? Function( AgentTasksRepliedEvent value)?  replied,TResult? Function( AgentTasksDismissedEvent value)?  dismissed,TResult? Function( AgentTasksTroubleClearedEvent value)?  troubleCleared,}){
final _that = this;
switch (_that) {
case AgentTasksStartedEvent() when started != null:
return started(_that);case AgentTasksPolledEvent() when polled != null:
return polled(_that);case AgentTasksGoalSentEvent() when goalSent != null:
return goalSent(_that);case AgentTasksStoppedEvent() when stopped != null:
return stopped(_that);case AgentTasksRepliedEvent() when replied != null:
return replied(_that);case AgentTasksDismissedEvent() when dismissed != null:
return dismissed(_that);case AgentTasksTroubleClearedEvent() when troubleCleared != null:
return troubleCleared(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  polled,TResult Function( String goal,  List<String> noteIds,  String newNoteBook,  bool confirmedAll)?  goalSent,TResult Function( int id)?  stopped,TResult Function( int id,  String text)?  replied,TResult Function( int id)?  dismissed,TResult Function()?  troubleCleared,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AgentTasksStartedEvent() when started != null:
return started();case AgentTasksPolledEvent() when polled != null:
return polled();case AgentTasksGoalSentEvent() when goalSent != null:
return goalSent(_that.goal,_that.noteIds,_that.newNoteBook,_that.confirmedAll);case AgentTasksStoppedEvent() when stopped != null:
return stopped(_that.id);case AgentTasksRepliedEvent() when replied != null:
return replied(_that.id,_that.text);case AgentTasksDismissedEvent() when dismissed != null:
return dismissed(_that.id);case AgentTasksTroubleClearedEvent() when troubleCleared != null:
return troubleCleared();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  polled,required TResult Function( String goal,  List<String> noteIds,  String newNoteBook,  bool confirmedAll)  goalSent,required TResult Function( int id)  stopped,required TResult Function( int id,  String text)  replied,required TResult Function( int id)  dismissed,required TResult Function()  troubleCleared,}) {final _that = this;
switch (_that) {
case AgentTasksStartedEvent():
return started();case AgentTasksPolledEvent():
return polled();case AgentTasksGoalSentEvent():
return goalSent(_that.goal,_that.noteIds,_that.newNoteBook,_that.confirmedAll);case AgentTasksStoppedEvent():
return stopped(_that.id);case AgentTasksRepliedEvent():
return replied(_that.id,_that.text);case AgentTasksDismissedEvent():
return dismissed(_that.id);case AgentTasksTroubleClearedEvent():
return troubleCleared();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  polled,TResult? Function( String goal,  List<String> noteIds,  String newNoteBook,  bool confirmedAll)?  goalSent,TResult? Function( int id)?  stopped,TResult? Function( int id,  String text)?  replied,TResult? Function( int id)?  dismissed,TResult? Function()?  troubleCleared,}) {final _that = this;
switch (_that) {
case AgentTasksStartedEvent() when started != null:
return started();case AgentTasksPolledEvent() when polled != null:
return polled();case AgentTasksGoalSentEvent() when goalSent != null:
return goalSent(_that.goal,_that.noteIds,_that.newNoteBook,_that.confirmedAll);case AgentTasksStoppedEvent() when stopped != null:
return stopped(_that.id);case AgentTasksRepliedEvent() when replied != null:
return replied(_that.id,_that.text);case AgentTasksDismissedEvent() when dismissed != null:
return dismissed(_that.id);case AgentTasksTroubleClearedEvent() when troubleCleared != null:
return troubleCleared();case _:
  return null;

}
}

}

/// @nodoc


class AgentTasksStartedEvent implements AgentTasksEvent {
  const AgentTasksStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTasksStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentTasksEvent.started()';
}


}




/// @nodoc


class AgentTasksPolledEvent implements AgentTasksEvent {
  const AgentTasksPolledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTasksPolledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentTasksEvent.polled()';
}


}




/// @nodoc


class AgentTasksGoalSentEvent implements AgentTasksEvent {
  const AgentTasksGoalSentEvent(this.goal, {final  List<String> noteIds = const <String>[], this.newNoteBook = '', this.confirmedAll = false}): _noteIds = noteIds;
  

 final  String goal;
 final  List<String> _noteIds;
@JsonKey() List<String> get noteIds {
  if (_noteIds is EqualUnmodifiableListView) return _noteIds;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_noteIds);
}

/// Where new notes go; the Inbox when empty.
@JsonKey() final  String newNoteBook;
/// The person agreed to the task reading every notebook.
@JsonKey() final  bool confirmedAll;

/// Create a copy of AgentTasksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentTasksGoalSentEventCopyWith<AgentTasksGoalSentEvent> get copyWith => _$AgentTasksGoalSentEventCopyWithImpl<AgentTasksGoalSentEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTasksGoalSentEvent&&(identical(other.goal, goal) || other.goal == goal)&&const DeepCollectionEquality().equals(other._noteIds, _noteIds)&&(identical(other.newNoteBook, newNoteBook) || other.newNoteBook == newNoteBook)&&(identical(other.confirmedAll, confirmedAll) || other.confirmedAll == confirmedAll));
}


@override
int get hashCode => Object.hash(runtimeType,goal,const DeepCollectionEquality().hash(_noteIds),newNoteBook,confirmedAll);

@override
String toString() {
  return 'AgentTasksEvent.goalSent(goal: $goal, noteIds: $noteIds, newNoteBook: $newNoteBook, confirmedAll: $confirmedAll)';
}


}

/// @nodoc
abstract mixin class $AgentTasksGoalSentEventCopyWith<$Res> implements $AgentTasksEventCopyWith<$Res> {
  factory $AgentTasksGoalSentEventCopyWith(AgentTasksGoalSentEvent value, $Res Function(AgentTasksGoalSentEvent) _then) = _$AgentTasksGoalSentEventCopyWithImpl;
@useResult
$Res call({
 String goal, List<String> noteIds, String newNoteBook, bool confirmedAll
});




}
/// @nodoc
class _$AgentTasksGoalSentEventCopyWithImpl<$Res>
    implements $AgentTasksGoalSentEventCopyWith<$Res> {
  _$AgentTasksGoalSentEventCopyWithImpl(this._self, this._then);

  final AgentTasksGoalSentEvent _self;
  final $Res Function(AgentTasksGoalSentEvent) _then;

/// Create a copy of AgentTasksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? goal = null,Object? noteIds = null,Object? newNoteBook = null,Object? confirmedAll = null,}) {
  return _then(AgentTasksGoalSentEvent(
null == goal ? _self.goal : goal // ignore: cast_nullable_to_non_nullable
as String,noteIds: null == noteIds ? _self._noteIds : noteIds // ignore: cast_nullable_to_non_nullable
as List<String>,newNoteBook: null == newNoteBook ? _self.newNoteBook : newNoteBook // ignore: cast_nullable_to_non_nullable
as String,confirmedAll: null == confirmedAll ? _self.confirmedAll : confirmedAll // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class AgentTasksStoppedEvent implements AgentTasksEvent {
  const AgentTasksStoppedEvent(this.id);
  

 final  int id;

/// Create a copy of AgentTasksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentTasksStoppedEventCopyWith<AgentTasksStoppedEvent> get copyWith => _$AgentTasksStoppedEventCopyWithImpl<AgentTasksStoppedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTasksStoppedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'AgentTasksEvent.stopped(id: $id)';
}


}

/// @nodoc
abstract mixin class $AgentTasksStoppedEventCopyWith<$Res> implements $AgentTasksEventCopyWith<$Res> {
  factory $AgentTasksStoppedEventCopyWith(AgentTasksStoppedEvent value, $Res Function(AgentTasksStoppedEvent) _then) = _$AgentTasksStoppedEventCopyWithImpl;
@useResult
$Res call({
 int id
});




}
/// @nodoc
class _$AgentTasksStoppedEventCopyWithImpl<$Res>
    implements $AgentTasksStoppedEventCopyWith<$Res> {
  _$AgentTasksStoppedEventCopyWithImpl(this._self, this._then);

  final AgentTasksStoppedEvent _self;
  final $Res Function(AgentTasksStoppedEvent) _then;

/// Create a copy of AgentTasksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(AgentTasksStoppedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class AgentTasksRepliedEvent implements AgentTasksEvent {
  const AgentTasksRepliedEvent(this.id, this.text);
  

 final  int id;
 final  String text;

/// Create a copy of AgentTasksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentTasksRepliedEventCopyWith<AgentTasksRepliedEvent> get copyWith => _$AgentTasksRepliedEventCopyWithImpl<AgentTasksRepliedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTasksRepliedEvent&&(identical(other.id, id) || other.id == id)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,id,text);

@override
String toString() {
  return 'AgentTasksEvent.replied(id: $id, text: $text)';
}


}

/// @nodoc
abstract mixin class $AgentTasksRepliedEventCopyWith<$Res> implements $AgentTasksEventCopyWith<$Res> {
  factory $AgentTasksRepliedEventCopyWith(AgentTasksRepliedEvent value, $Res Function(AgentTasksRepliedEvent) _then) = _$AgentTasksRepliedEventCopyWithImpl;
@useResult
$Res call({
 int id, String text
});




}
/// @nodoc
class _$AgentTasksRepliedEventCopyWithImpl<$Res>
    implements $AgentTasksRepliedEventCopyWith<$Res> {
  _$AgentTasksRepliedEventCopyWithImpl(this._self, this._then);

  final AgentTasksRepliedEvent _self;
  final $Res Function(AgentTasksRepliedEvent) _then;

/// Create a copy of AgentTasksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? text = null,}) {
  return _then(AgentTasksRepliedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as int,null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AgentTasksDismissedEvent implements AgentTasksEvent {
  const AgentTasksDismissedEvent(this.id);
  

 final  int id;

/// Create a copy of AgentTasksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentTasksDismissedEventCopyWith<AgentTasksDismissedEvent> get copyWith => _$AgentTasksDismissedEventCopyWithImpl<AgentTasksDismissedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTasksDismissedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'AgentTasksEvent.dismissed(id: $id)';
}


}

/// @nodoc
abstract mixin class $AgentTasksDismissedEventCopyWith<$Res> implements $AgentTasksEventCopyWith<$Res> {
  factory $AgentTasksDismissedEventCopyWith(AgentTasksDismissedEvent value, $Res Function(AgentTasksDismissedEvent) _then) = _$AgentTasksDismissedEventCopyWithImpl;
@useResult
$Res call({
 int id
});




}
/// @nodoc
class _$AgentTasksDismissedEventCopyWithImpl<$Res>
    implements $AgentTasksDismissedEventCopyWith<$Res> {
  _$AgentTasksDismissedEventCopyWithImpl(this._self, this._then);

  final AgentTasksDismissedEvent _self;
  final $Res Function(AgentTasksDismissedEvent) _then;

/// Create a copy of AgentTasksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(AgentTasksDismissedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class AgentTasksTroubleClearedEvent implements AgentTasksEvent {
  const AgentTasksTroubleClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTasksTroubleClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AgentTasksEvent.troubleCleared()';
}


}




/// @nodoc
mixin _$AgentTasksState {

 LoadStatus get status; AgentTaskPrefs get prefs; List<AgentTask> get tasks;/// A start is on its way.
 bool get starting;/// Counts the tasks started from this page, so it can clear its field.
 int get started;/// Why the last start didn't happen, as the core words it.
 String? get trouble;
/// Create a copy of AgentTasksState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentTasksStateCopyWith<AgentTasksState> get copyWith => _$AgentTasksStateCopyWithImpl<AgentTasksState>(this as AgentTasksState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentTasksState&&(identical(other.status, status) || other.status == status)&&(identical(other.prefs, prefs) || other.prefs == prefs)&&const DeepCollectionEquality().equals(other.tasks, tasks)&&(identical(other.starting, starting) || other.starting == starting)&&(identical(other.started, started) || other.started == started)&&(identical(other.trouble, trouble) || other.trouble == trouble));
}


@override
int get hashCode => Object.hash(runtimeType,status,prefs,const DeepCollectionEquality().hash(tasks),starting,started,trouble);

@override
String toString() {
  return 'AgentTasksState(status: $status, prefs: $prefs, tasks: $tasks, starting: $starting, started: $started, trouble: $trouble)';
}


}

/// @nodoc
abstract mixin class $AgentTasksStateCopyWith<$Res>  {
  factory $AgentTasksStateCopyWith(AgentTasksState value, $Res Function(AgentTasksState) _then) = _$AgentTasksStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, AgentTaskPrefs prefs, List<AgentTask> tasks, bool starting, int started, String? trouble
});


$AgentTaskPrefsCopyWith<$Res> get prefs;

}
/// @nodoc
class _$AgentTasksStateCopyWithImpl<$Res>
    implements $AgentTasksStateCopyWith<$Res> {
  _$AgentTasksStateCopyWithImpl(this._self, this._then);

  final AgentTasksState _self;
  final $Res Function(AgentTasksState) _then;

/// Create a copy of AgentTasksState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? prefs = null,Object? tasks = null,Object? starting = null,Object? started = null,Object? trouble = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,prefs: null == prefs ? _self.prefs : prefs // ignore: cast_nullable_to_non_nullable
as AgentTaskPrefs,tasks: null == tasks ? _self.tasks : tasks // ignore: cast_nullable_to_non_nullable
as List<AgentTask>,starting: null == starting ? _self.starting : starting // ignore: cast_nullable_to_non_nullable
as bool,started: null == started ? _self.started : started // ignore: cast_nullable_to_non_nullable
as int,trouble: freezed == trouble ? _self.trouble : trouble // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}
/// Create a copy of AgentTasksState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AgentTaskPrefsCopyWith<$Res> get prefs {
  
  return $AgentTaskPrefsCopyWith<$Res>(_self.prefs, (value) {
    return _then(_self.copyWith(prefs: value));
  });
}
}


/// Adds pattern-matching-related methods to [AgentTasksState].
extension AgentTasksStatePatterns on AgentTasksState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AgentTasksState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AgentTasksState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AgentTasksState value)  $default,){
final _that = this;
switch (_that) {
case _AgentTasksState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AgentTasksState value)?  $default,){
final _that = this;
switch (_that) {
case _AgentTasksState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  AgentTaskPrefs prefs,  List<AgentTask> tasks,  bool starting,  int started,  String? trouble)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AgentTasksState() when $default != null:
return $default(_that.status,_that.prefs,_that.tasks,_that.starting,_that.started,_that.trouble);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  AgentTaskPrefs prefs,  List<AgentTask> tasks,  bool starting,  int started,  String? trouble)  $default,) {final _that = this;
switch (_that) {
case _AgentTasksState():
return $default(_that.status,_that.prefs,_that.tasks,_that.starting,_that.started,_that.trouble);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  AgentTaskPrefs prefs,  List<AgentTask> tasks,  bool starting,  int started,  String? trouble)?  $default,) {final _that = this;
switch (_that) {
case _AgentTasksState() when $default != null:
return $default(_that.status,_that.prefs,_that.tasks,_that.starting,_that.started,_that.trouble);case _:
  return null;

}
}

}

/// @nodoc


class _AgentTasksState implements AgentTasksState {
  const _AgentTasksState({this.status = LoadStatus.loading, this.prefs = const AgentTaskPrefs(), final  List<AgentTask> tasks = const <AgentTask>[], this.starting = false, this.started = 0, this.trouble}): _tasks = tasks;
  

@override@JsonKey() final  LoadStatus status;
@override@JsonKey() final  AgentTaskPrefs prefs;
 final  List<AgentTask> _tasks;
@override@JsonKey() List<AgentTask> get tasks {
  if (_tasks is EqualUnmodifiableListView) return _tasks;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_tasks);
}

/// A start is on its way.
@override@JsonKey() final  bool starting;
/// Counts the tasks started from this page, so it can clear its field.
@override@JsonKey() final  int started;
/// Why the last start didn't happen, as the core words it.
@override final  String? trouble;

/// Create a copy of AgentTasksState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AgentTasksStateCopyWith<_AgentTasksState> get copyWith => __$AgentTasksStateCopyWithImpl<_AgentTasksState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AgentTasksState&&(identical(other.status, status) || other.status == status)&&(identical(other.prefs, prefs) || other.prefs == prefs)&&const DeepCollectionEquality().equals(other._tasks, _tasks)&&(identical(other.starting, starting) || other.starting == starting)&&(identical(other.started, started) || other.started == started)&&(identical(other.trouble, trouble) || other.trouble == trouble));
}


@override
int get hashCode => Object.hash(runtimeType,status,prefs,const DeepCollectionEquality().hash(_tasks),starting,started,trouble);

@override
String toString() {
  return 'AgentTasksState(status: $status, prefs: $prefs, tasks: $tasks, starting: $starting, started: $started, trouble: $trouble)';
}


}

/// @nodoc
abstract mixin class _$AgentTasksStateCopyWith<$Res> implements $AgentTasksStateCopyWith<$Res> {
  factory _$AgentTasksStateCopyWith(_AgentTasksState value, $Res Function(_AgentTasksState) _then) = __$AgentTasksStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, AgentTaskPrefs prefs, List<AgentTask> tasks, bool starting, int started, String? trouble
});


@override $AgentTaskPrefsCopyWith<$Res> get prefs;

}
/// @nodoc
class __$AgentTasksStateCopyWithImpl<$Res>
    implements _$AgentTasksStateCopyWith<$Res> {
  __$AgentTasksStateCopyWithImpl(this._self, this._then);

  final _AgentTasksState _self;
  final $Res Function(_AgentTasksState) _then;

/// Create a copy of AgentTasksState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? prefs = null,Object? tasks = null,Object? starting = null,Object? started = null,Object? trouble = freezed,}) {
  return _then(_AgentTasksState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,prefs: null == prefs ? _self.prefs : prefs // ignore: cast_nullable_to_non_nullable
as AgentTaskPrefs,tasks: null == tasks ? _self._tasks : tasks // ignore: cast_nullable_to_non_nullable
as List<AgentTask>,starting: null == starting ? _self.starting : starting // ignore: cast_nullable_to_non_nullable
as bool,started: null == started ? _self.started : started // ignore: cast_nullable_to_non_nullable
as int,trouble: freezed == trouble ? _self.trouble : trouble // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of AgentTasksState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AgentTaskPrefsCopyWith<$Res> get prefs {
  
  return $AgentTaskPrefsCopyWith<$Res>(_self.prefs, (value) {
    return _then(_self.copyWith(prefs: value));
  });
}
}

// dart format on

// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'tasks_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$TasksEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TasksEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TasksEvent()';
}


}

/// @nodoc
class $TasksEventCopyWith<$Res>  {
$TasksEventCopyWith(TasksEvent _, $Res Function(TasksEvent) __);
}


/// Adds pattern-matching-related methods to [TasksEvent].
extension TasksEventPatterns on TasksEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( TasksStartedEvent value)?  started,TResult Function( TasksNotebookChosenEvent value)?  notebookChosen,TResult Function( TasksShowDoneChangedEvent value)?  showDoneChanged,TResult Function( TasksChangesReceivedEvent value)?  changesReceived,TResult Function( TasksRetriedEvent value)?  retried,TResult Function( TasksTickedEvent value)?  ticked,TResult Function( TasksNoticeDismissedEvent value)?  noticeDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case TasksStartedEvent() when started != null:
return started(_that);case TasksNotebookChosenEvent() when notebookChosen != null:
return notebookChosen(_that);case TasksShowDoneChangedEvent() when showDoneChanged != null:
return showDoneChanged(_that);case TasksChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case TasksRetriedEvent() when retried != null:
return retried(_that);case TasksTickedEvent() when ticked != null:
return ticked(_that);case TasksNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( TasksStartedEvent value)  started,required TResult Function( TasksNotebookChosenEvent value)  notebookChosen,required TResult Function( TasksShowDoneChangedEvent value)  showDoneChanged,required TResult Function( TasksChangesReceivedEvent value)  changesReceived,required TResult Function( TasksRetriedEvent value)  retried,required TResult Function( TasksTickedEvent value)  ticked,required TResult Function( TasksNoticeDismissedEvent value)  noticeDismissed,}){
final _that = this;
switch (_that) {
case TasksStartedEvent():
return started(_that);case TasksNotebookChosenEvent():
return notebookChosen(_that);case TasksShowDoneChangedEvent():
return showDoneChanged(_that);case TasksChangesReceivedEvent():
return changesReceived(_that);case TasksRetriedEvent():
return retried(_that);case TasksTickedEvent():
return ticked(_that);case TasksNoticeDismissedEvent():
return noticeDismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( TasksStartedEvent value)?  started,TResult? Function( TasksNotebookChosenEvent value)?  notebookChosen,TResult? Function( TasksShowDoneChangedEvent value)?  showDoneChanged,TResult? Function( TasksChangesReceivedEvent value)?  changesReceived,TResult? Function( TasksRetriedEvent value)?  retried,TResult? Function( TasksTickedEvent value)?  ticked,TResult? Function( TasksNoticeDismissedEvent value)?  noticeDismissed,}){
final _that = this;
switch (_that) {
case TasksStartedEvent() when started != null:
return started(_that);case TasksNotebookChosenEvent() when notebookChosen != null:
return notebookChosen(_that);case TasksShowDoneChangedEvent() when showDoneChanged != null:
return showDoneChanged(_that);case TasksChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case TasksRetriedEvent() when retried != null:
return retried(_that);case TasksTickedEvent() when ticked != null:
return ticked(_that);case TasksNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( String? notebookId)?  notebookChosen,TResult Function( bool showDone)?  showDoneChanged,TResult Function( List<String> ids)?  changesReceived,TResult Function()?  retried,TResult Function( TaskItem task)?  ticked,TResult Function()?  noticeDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case TasksStartedEvent() when started != null:
return started();case TasksNotebookChosenEvent() when notebookChosen != null:
return notebookChosen(_that.notebookId);case TasksShowDoneChangedEvent() when showDoneChanged != null:
return showDoneChanged(_that.showDone);case TasksChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case TasksRetriedEvent() when retried != null:
return retried();case TasksTickedEvent() when ticked != null:
return ticked(_that.task);case TasksNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( String? notebookId)  notebookChosen,required TResult Function( bool showDone)  showDoneChanged,required TResult Function( List<String> ids)  changesReceived,required TResult Function()  retried,required TResult Function( TaskItem task)  ticked,required TResult Function()  noticeDismissed,}) {final _that = this;
switch (_that) {
case TasksStartedEvent():
return started();case TasksNotebookChosenEvent():
return notebookChosen(_that.notebookId);case TasksShowDoneChangedEvent():
return showDoneChanged(_that.showDone);case TasksChangesReceivedEvent():
return changesReceived(_that.ids);case TasksRetriedEvent():
return retried();case TasksTickedEvent():
return ticked(_that.task);case TasksNoticeDismissedEvent():
return noticeDismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( String? notebookId)?  notebookChosen,TResult? Function( bool showDone)?  showDoneChanged,TResult? Function( List<String> ids)?  changesReceived,TResult? Function()?  retried,TResult? Function( TaskItem task)?  ticked,TResult? Function()?  noticeDismissed,}) {final _that = this;
switch (_that) {
case TasksStartedEvent() when started != null:
return started();case TasksNotebookChosenEvent() when notebookChosen != null:
return notebookChosen(_that.notebookId);case TasksShowDoneChangedEvent() when showDoneChanged != null:
return showDoneChanged(_that.showDone);case TasksChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case TasksRetriedEvent() when retried != null:
return retried();case TasksTickedEvent() when ticked != null:
return ticked(_that.task);case TasksNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case _:
  return null;

}
}

}

/// @nodoc


class TasksStartedEvent implements TasksEvent {
  const TasksStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TasksStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TasksEvent.started()';
}


}




/// @nodoc


class TasksNotebookChosenEvent implements TasksEvent {
  const TasksNotebookChosenEvent(this.notebookId);
  

 final  String? notebookId;

/// Create a copy of TasksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TasksNotebookChosenEventCopyWith<TasksNotebookChosenEvent> get copyWith => _$TasksNotebookChosenEventCopyWithImpl<TasksNotebookChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TasksNotebookChosenEvent&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'TasksEvent.notebookChosen(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $TasksNotebookChosenEventCopyWith<$Res> implements $TasksEventCopyWith<$Res> {
  factory $TasksNotebookChosenEventCopyWith(TasksNotebookChosenEvent value, $Res Function(TasksNotebookChosenEvent) _then) = _$TasksNotebookChosenEventCopyWithImpl;
@useResult
$Res call({
 String? notebookId
});




}
/// @nodoc
class _$TasksNotebookChosenEventCopyWithImpl<$Res>
    implements $TasksNotebookChosenEventCopyWith<$Res> {
  _$TasksNotebookChosenEventCopyWithImpl(this._self, this._then);

  final TasksNotebookChosenEvent _self;
  final $Res Function(TasksNotebookChosenEvent) _then;

/// Create a copy of TasksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notebookId = freezed,}) {
  return _then(TasksNotebookChosenEvent(
freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class TasksShowDoneChangedEvent implements TasksEvent {
  const TasksShowDoneChangedEvent({required this.showDone});
  

 final  bool showDone;

/// Create a copy of TasksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TasksShowDoneChangedEventCopyWith<TasksShowDoneChangedEvent> get copyWith => _$TasksShowDoneChangedEventCopyWithImpl<TasksShowDoneChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TasksShowDoneChangedEvent&&(identical(other.showDone, showDone) || other.showDone == showDone));
}


@override
int get hashCode => Object.hash(runtimeType,showDone);

@override
String toString() {
  return 'TasksEvent.showDoneChanged(showDone: $showDone)';
}


}

/// @nodoc
abstract mixin class $TasksShowDoneChangedEventCopyWith<$Res> implements $TasksEventCopyWith<$Res> {
  factory $TasksShowDoneChangedEventCopyWith(TasksShowDoneChangedEvent value, $Res Function(TasksShowDoneChangedEvent) _then) = _$TasksShowDoneChangedEventCopyWithImpl;
@useResult
$Res call({
 bool showDone
});




}
/// @nodoc
class _$TasksShowDoneChangedEventCopyWithImpl<$Res>
    implements $TasksShowDoneChangedEventCopyWith<$Res> {
  _$TasksShowDoneChangedEventCopyWithImpl(this._self, this._then);

  final TasksShowDoneChangedEvent _self;
  final $Res Function(TasksShowDoneChangedEvent) _then;

/// Create a copy of TasksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? showDone = null,}) {
  return _then(TasksShowDoneChangedEvent(
showDone: null == showDone ? _self.showDone : showDone // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class TasksChangesReceivedEvent implements TasksEvent {
  const TasksChangesReceivedEvent(final  List<String> ids): _ids = ids;
  

 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of TasksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TasksChangesReceivedEventCopyWith<TasksChangesReceivedEvent> get copyWith => _$TasksChangesReceivedEventCopyWithImpl<TasksChangesReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TasksChangesReceivedEvent&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'TasksEvent.changesReceived(ids: $ids)';
}


}

/// @nodoc
abstract mixin class $TasksChangesReceivedEventCopyWith<$Res> implements $TasksEventCopyWith<$Res> {
  factory $TasksChangesReceivedEventCopyWith(TasksChangesReceivedEvent value, $Res Function(TasksChangesReceivedEvent) _then) = _$TasksChangesReceivedEventCopyWithImpl;
@useResult
$Res call({
 List<String> ids
});




}
/// @nodoc
class _$TasksChangesReceivedEventCopyWithImpl<$Res>
    implements $TasksChangesReceivedEventCopyWith<$Res> {
  _$TasksChangesReceivedEventCopyWithImpl(this._self, this._then);

  final TasksChangesReceivedEvent _self;
  final $Res Function(TasksChangesReceivedEvent) _then;

/// Create a copy of TasksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ids = null,}) {
  return _then(TasksChangesReceivedEvent(
null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class TasksRetriedEvent implements TasksEvent {
  const TasksRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TasksRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TasksEvent.retried()';
}


}




/// @nodoc


class TasksTickedEvent implements TasksEvent {
  const TasksTickedEvent(this.task);
  

 final  TaskItem task;

/// Create a copy of TasksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TasksTickedEventCopyWith<TasksTickedEvent> get copyWith => _$TasksTickedEventCopyWithImpl<TasksTickedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TasksTickedEvent&&(identical(other.task, task) || other.task == task));
}


@override
int get hashCode => Object.hash(runtimeType,task);

@override
String toString() {
  return 'TasksEvent.ticked(task: $task)';
}


}

/// @nodoc
abstract mixin class $TasksTickedEventCopyWith<$Res> implements $TasksEventCopyWith<$Res> {
  factory $TasksTickedEventCopyWith(TasksTickedEvent value, $Res Function(TasksTickedEvent) _then) = _$TasksTickedEventCopyWithImpl;
@useResult
$Res call({
 TaskItem task
});




}
/// @nodoc
class _$TasksTickedEventCopyWithImpl<$Res>
    implements $TasksTickedEventCopyWith<$Res> {
  _$TasksTickedEventCopyWithImpl(this._self, this._then);

  final TasksTickedEvent _self;
  final $Res Function(TasksTickedEvent) _then;

/// Create a copy of TasksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? task = null,}) {
  return _then(TasksTickedEvent(
null == task ? _self.task : task // ignore: cast_nullable_to_non_nullable
as TaskItem,
  ));
}


}

/// @nodoc


class TasksNoticeDismissedEvent implements TasksEvent {
  const TasksNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TasksNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'TasksEvent.noticeDismissed()';
}


}




/// @nodoc
mixin _$TasksState {

 TasksStatus get status;/// The notebook whose tasks show; null for every notebook.
 String? get notebookId;/// Whether done tasks show too, in Completed.
 bool get showDone;/// The keys of the tasks ticked since the place opened: they stay where
/// they were, struck through, until it is opened again.
 List<String> get kept;/// The tasks, in groups; only the groups that have any.
 TaskBoard get board;/// Why reading failed, while [status] is failed.
 ApplicationError? get error;/// What went wrong with the last tick, until it is dismissed.
 TasksNotice? get notice;
/// Create a copy of TasksState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TasksStateCopyWith<TasksState> get copyWith => _$TasksStateCopyWithImpl<TasksState>(this as TasksState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TasksState&&(identical(other.status, status) || other.status == status)&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId)&&(identical(other.showDone, showDone) || other.showDone == showDone)&&const DeepCollectionEquality().equals(other.kept, kept)&&(identical(other.board, board) || other.board == board)&&(identical(other.error, error) || other.error == error)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,status,notebookId,showDone,const DeepCollectionEquality().hash(kept),board,error,notice);

@override
String toString() {
  return 'TasksState(status: $status, notebookId: $notebookId, showDone: $showDone, kept: $kept, board: $board, error: $error, notice: $notice)';
}


}

/// @nodoc
abstract mixin class $TasksStateCopyWith<$Res>  {
  factory $TasksStateCopyWith(TasksState value, $Res Function(TasksState) _then) = _$TasksStateCopyWithImpl;
@useResult
$Res call({
 TasksStatus status, String? notebookId, bool showDone, List<String> kept, TaskBoard board, ApplicationError? error, TasksNotice? notice
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$TasksStateCopyWithImpl<$Res>
    implements $TasksStateCopyWith<$Res> {
  _$TasksStateCopyWithImpl(this._self, this._then);

  final TasksState _self;
  final $Res Function(TasksState) _then;

/// Create a copy of TasksState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? notebookId = freezed,Object? showDone = null,Object? kept = null,Object? board = null,Object? error = freezed,Object? notice = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as TasksStatus,notebookId: freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,showDone: null == showDone ? _self.showDone : showDone // ignore: cast_nullable_to_non_nullable
as bool,kept: null == kept ? _self.kept : kept // ignore: cast_nullable_to_non_nullable
as List<String>,board: null == board ? _self.board : board // ignore: cast_nullable_to_non_nullable
as TaskBoard,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as TasksNotice?,
  ));
}
/// Create a copy of TasksState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}


/// Adds pattern-matching-related methods to [TasksState].
extension TasksStatePatterns on TasksState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TasksState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TasksState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TasksState value)  $default,){
final _that = this;
switch (_that) {
case _TasksState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TasksState value)?  $default,){
final _that = this;
switch (_that) {
case _TasksState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( TasksStatus status,  String? notebookId,  bool showDone,  List<String> kept,  TaskBoard board,  ApplicationError? error,  TasksNotice? notice)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TasksState() when $default != null:
return $default(_that.status,_that.notebookId,_that.showDone,_that.kept,_that.board,_that.error,_that.notice);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( TasksStatus status,  String? notebookId,  bool showDone,  List<String> kept,  TaskBoard board,  ApplicationError? error,  TasksNotice? notice)  $default,) {final _that = this;
switch (_that) {
case _TasksState():
return $default(_that.status,_that.notebookId,_that.showDone,_that.kept,_that.board,_that.error,_that.notice);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( TasksStatus status,  String? notebookId,  bool showDone,  List<String> kept,  TaskBoard board,  ApplicationError? error,  TasksNotice? notice)?  $default,) {final _that = this;
switch (_that) {
case _TasksState() when $default != null:
return $default(_that.status,_that.notebookId,_that.showDone,_that.kept,_that.board,_that.error,_that.notice);case _:
  return null;

}
}

}

/// @nodoc


class _TasksState extends TasksState {
  const _TasksState({this.status = TasksStatus.loading, this.notebookId, this.showDone = false, final  List<String> kept = const <String>[], this.board = const TaskBoard(groups: <TaskSection>[]), this.error, this.notice}): _kept = kept,super._();
  

@override@JsonKey() final  TasksStatus status;
/// The notebook whose tasks show; null for every notebook.
@override final  String? notebookId;
/// Whether done tasks show too, in Completed.
@override@JsonKey() final  bool showDone;
/// The keys of the tasks ticked since the place opened: they stay where
/// they were, struck through, until it is opened again.
 final  List<String> _kept;
/// The keys of the tasks ticked since the place opened: they stay where
/// they were, struck through, until it is opened again.
@override@JsonKey() List<String> get kept {
  if (_kept is EqualUnmodifiableListView) return _kept;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_kept);
}

/// The tasks, in groups; only the groups that have any.
@override@JsonKey() final  TaskBoard board;
/// Why reading failed, while [status] is failed.
@override final  ApplicationError? error;
/// What went wrong with the last tick, until it is dismissed.
@override final  TasksNotice? notice;

/// Create a copy of TasksState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TasksStateCopyWith<_TasksState> get copyWith => __$TasksStateCopyWithImpl<_TasksState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TasksState&&(identical(other.status, status) || other.status == status)&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId)&&(identical(other.showDone, showDone) || other.showDone == showDone)&&const DeepCollectionEquality().equals(other._kept, _kept)&&(identical(other.board, board) || other.board == board)&&(identical(other.error, error) || other.error == error)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,status,notebookId,showDone,const DeepCollectionEquality().hash(_kept),board,error,notice);

@override
String toString() {
  return 'TasksState(status: $status, notebookId: $notebookId, showDone: $showDone, kept: $kept, board: $board, error: $error, notice: $notice)';
}


}

/// @nodoc
abstract mixin class _$TasksStateCopyWith<$Res> implements $TasksStateCopyWith<$Res> {
  factory _$TasksStateCopyWith(_TasksState value, $Res Function(_TasksState) _then) = __$TasksStateCopyWithImpl;
@override @useResult
$Res call({
 TasksStatus status, String? notebookId, bool showDone, List<String> kept, TaskBoard board, ApplicationError? error, TasksNotice? notice
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$TasksStateCopyWithImpl<$Res>
    implements _$TasksStateCopyWith<$Res> {
  __$TasksStateCopyWithImpl(this._self, this._then);

  final _TasksState _self;
  final $Res Function(_TasksState) _then;

/// Create a copy of TasksState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? notebookId = freezed,Object? showDone = null,Object? kept = null,Object? board = null,Object? error = freezed,Object? notice = freezed,}) {
  return _then(_TasksState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as TasksStatus,notebookId: freezed == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String?,showDone: null == showDone ? _self.showDone : showDone // ignore: cast_nullable_to_non_nullable
as bool,kept: null == kept ? _self._kept : kept // ignore: cast_nullable_to_non_nullable
as List<String>,board: null == board ? _self.board : board // ignore: cast_nullable_to_non_nullable
as TaskBoard,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as TasksNotice?,
  ));
}

/// Create a copy of TasksState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res>? get error {
    if (_self.error == null) {
    return null;
  }

  return $ApplicationErrorCopyWith<$Res>(_self.error!, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}

// dart format on

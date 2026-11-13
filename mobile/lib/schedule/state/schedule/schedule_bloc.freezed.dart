// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'schedule_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ScheduleEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ScheduleEvent()';
}


}

/// @nodoc
class $ScheduleEventCopyWith<$Res>  {
$ScheduleEventCopyWith(ScheduleEvent _, $Res Function(ScheduleEvent) __);
}


/// Adds pattern-matching-related methods to [ScheduleEvent].
extension ScheduleEventPatterns on ScheduleEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ScheduleStartedEvent value)?  started,TResult Function( ScheduleModeChangedEvent value)?  modeChanged,TResult Function( ScheduleMonthSteppedEvent value)?  monthStepped,TResult Function( ScheduleTodayShownEvent value)?  todayShown,TResult Function( ScheduleDaySelectedEvent value)?  daySelected,TResult Function( ScheduleNotebookToggledEvent value)?  notebookToggled,TResult Function( ScheduleNotebooksClearedEvent value)?  notebooksCleared,TResult Function( ScheduleKeyToggledEvent value)?  keyToggled,TResult Function( ScheduleKeysClearedEvent value)?  keysCleared,TResult Function( ScheduleShowDoneChangedEvent value)?  showDoneChanged,TResult Function( ScheduleChangesReceivedEvent value)?  changesReceived,TResult Function( ScheduleRetriedEvent value)?  retried,TResult Function( ScheduleTickedEvent value)?  ticked,TResult Function( ScheduleMovedEvent value)?  moved,TResult Function( ScheduleNoticeDismissedEvent value)?  noticeDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ScheduleStartedEvent() when started != null:
return started(_that);case ScheduleModeChangedEvent() when modeChanged != null:
return modeChanged(_that);case ScheduleMonthSteppedEvent() when monthStepped != null:
return monthStepped(_that);case ScheduleTodayShownEvent() when todayShown != null:
return todayShown(_that);case ScheduleDaySelectedEvent() when daySelected != null:
return daySelected(_that);case ScheduleNotebookToggledEvent() when notebookToggled != null:
return notebookToggled(_that);case ScheduleNotebooksClearedEvent() when notebooksCleared != null:
return notebooksCleared(_that);case ScheduleKeyToggledEvent() when keyToggled != null:
return keyToggled(_that);case ScheduleKeysClearedEvent() when keysCleared != null:
return keysCleared(_that);case ScheduleShowDoneChangedEvent() when showDoneChanged != null:
return showDoneChanged(_that);case ScheduleChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case ScheduleRetriedEvent() when retried != null:
return retried(_that);case ScheduleTickedEvent() when ticked != null:
return ticked(_that);case ScheduleMovedEvent() when moved != null:
return moved(_that);case ScheduleNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ScheduleStartedEvent value)  started,required TResult Function( ScheduleModeChangedEvent value)  modeChanged,required TResult Function( ScheduleMonthSteppedEvent value)  monthStepped,required TResult Function( ScheduleTodayShownEvent value)  todayShown,required TResult Function( ScheduleDaySelectedEvent value)  daySelected,required TResult Function( ScheduleNotebookToggledEvent value)  notebookToggled,required TResult Function( ScheduleNotebooksClearedEvent value)  notebooksCleared,required TResult Function( ScheduleKeyToggledEvent value)  keyToggled,required TResult Function( ScheduleKeysClearedEvent value)  keysCleared,required TResult Function( ScheduleShowDoneChangedEvent value)  showDoneChanged,required TResult Function( ScheduleChangesReceivedEvent value)  changesReceived,required TResult Function( ScheduleRetriedEvent value)  retried,required TResult Function( ScheduleTickedEvent value)  ticked,required TResult Function( ScheduleMovedEvent value)  moved,required TResult Function( ScheduleNoticeDismissedEvent value)  noticeDismissed,}){
final _that = this;
switch (_that) {
case ScheduleStartedEvent():
return started(_that);case ScheduleModeChangedEvent():
return modeChanged(_that);case ScheduleMonthSteppedEvent():
return monthStepped(_that);case ScheduleTodayShownEvent():
return todayShown(_that);case ScheduleDaySelectedEvent():
return daySelected(_that);case ScheduleNotebookToggledEvent():
return notebookToggled(_that);case ScheduleNotebooksClearedEvent():
return notebooksCleared(_that);case ScheduleKeyToggledEvent():
return keyToggled(_that);case ScheduleKeysClearedEvent():
return keysCleared(_that);case ScheduleShowDoneChangedEvent():
return showDoneChanged(_that);case ScheduleChangesReceivedEvent():
return changesReceived(_that);case ScheduleRetriedEvent():
return retried(_that);case ScheduleTickedEvent():
return ticked(_that);case ScheduleMovedEvent():
return moved(_that);case ScheduleNoticeDismissedEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ScheduleStartedEvent value)?  started,TResult? Function( ScheduleModeChangedEvent value)?  modeChanged,TResult? Function( ScheduleMonthSteppedEvent value)?  monthStepped,TResult? Function( ScheduleTodayShownEvent value)?  todayShown,TResult? Function( ScheduleDaySelectedEvent value)?  daySelected,TResult? Function( ScheduleNotebookToggledEvent value)?  notebookToggled,TResult? Function( ScheduleNotebooksClearedEvent value)?  notebooksCleared,TResult? Function( ScheduleKeyToggledEvent value)?  keyToggled,TResult? Function( ScheduleKeysClearedEvent value)?  keysCleared,TResult? Function( ScheduleShowDoneChangedEvent value)?  showDoneChanged,TResult? Function( ScheduleChangesReceivedEvent value)?  changesReceived,TResult? Function( ScheduleRetriedEvent value)?  retried,TResult? Function( ScheduleTickedEvent value)?  ticked,TResult? Function( ScheduleMovedEvent value)?  moved,TResult? Function( ScheduleNoticeDismissedEvent value)?  noticeDismissed,}){
final _that = this;
switch (_that) {
case ScheduleStartedEvent() when started != null:
return started(_that);case ScheduleModeChangedEvent() when modeChanged != null:
return modeChanged(_that);case ScheduleMonthSteppedEvent() when monthStepped != null:
return monthStepped(_that);case ScheduleTodayShownEvent() when todayShown != null:
return todayShown(_that);case ScheduleDaySelectedEvent() when daySelected != null:
return daySelected(_that);case ScheduleNotebookToggledEvent() when notebookToggled != null:
return notebookToggled(_that);case ScheduleNotebooksClearedEvent() when notebooksCleared != null:
return notebooksCleared(_that);case ScheduleKeyToggledEvent() when keyToggled != null:
return keyToggled(_that);case ScheduleKeysClearedEvent() when keysCleared != null:
return keysCleared(_that);case ScheduleShowDoneChangedEvent() when showDoneChanged != null:
return showDoneChanged(_that);case ScheduleChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case ScheduleRetriedEvent() when retried != null:
return retried(_that);case ScheduleTickedEvent() when ticked != null:
return ticked(_that);case ScheduleMovedEvent() when moved != null:
return moved(_that);case ScheduleNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( DayDate today,  int weekStart)?  started,TResult Function( ScheduleMode mode)?  modeChanged,TResult Function( int delta)?  monthStepped,TResult Function( DayDate today)?  todayShown,TResult Function( DayDate day)?  daySelected,TResult Function( String notebookId)?  notebookToggled,TResult Function()?  notebooksCleared,TResult Function( String key)?  keyToggled,TResult Function()?  keysCleared,TResult Function( bool showDone)?  showDoneChanged,TResult Function( List<String> ids)?  changesReceived,TResult Function()?  retried,TResult Function( ScheduleRow row)?  ticked,TResult Function( ScheduleRow row,  DayDate day)?  moved,TResult Function()?  noticeDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ScheduleStartedEvent() when started != null:
return started(_that.today,_that.weekStart);case ScheduleModeChangedEvent() when modeChanged != null:
return modeChanged(_that.mode);case ScheduleMonthSteppedEvent() when monthStepped != null:
return monthStepped(_that.delta);case ScheduleTodayShownEvent() when todayShown != null:
return todayShown(_that.today);case ScheduleDaySelectedEvent() when daySelected != null:
return daySelected(_that.day);case ScheduleNotebookToggledEvent() when notebookToggled != null:
return notebookToggled(_that.notebookId);case ScheduleNotebooksClearedEvent() when notebooksCleared != null:
return notebooksCleared();case ScheduleKeyToggledEvent() when keyToggled != null:
return keyToggled(_that.key);case ScheduleKeysClearedEvent() when keysCleared != null:
return keysCleared();case ScheduleShowDoneChangedEvent() when showDoneChanged != null:
return showDoneChanged(_that.showDone);case ScheduleChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case ScheduleRetriedEvent() when retried != null:
return retried();case ScheduleTickedEvent() when ticked != null:
return ticked(_that.row);case ScheduleMovedEvent() when moved != null:
return moved(_that.row,_that.day);case ScheduleNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( DayDate today,  int weekStart)  started,required TResult Function( ScheduleMode mode)  modeChanged,required TResult Function( int delta)  monthStepped,required TResult Function( DayDate today)  todayShown,required TResult Function( DayDate day)  daySelected,required TResult Function( String notebookId)  notebookToggled,required TResult Function()  notebooksCleared,required TResult Function( String key)  keyToggled,required TResult Function()  keysCleared,required TResult Function( bool showDone)  showDoneChanged,required TResult Function( List<String> ids)  changesReceived,required TResult Function()  retried,required TResult Function( ScheduleRow row)  ticked,required TResult Function( ScheduleRow row,  DayDate day)  moved,required TResult Function()  noticeDismissed,}) {final _that = this;
switch (_that) {
case ScheduleStartedEvent():
return started(_that.today,_that.weekStart);case ScheduleModeChangedEvent():
return modeChanged(_that.mode);case ScheduleMonthSteppedEvent():
return monthStepped(_that.delta);case ScheduleTodayShownEvent():
return todayShown(_that.today);case ScheduleDaySelectedEvent():
return daySelected(_that.day);case ScheduleNotebookToggledEvent():
return notebookToggled(_that.notebookId);case ScheduleNotebooksClearedEvent():
return notebooksCleared();case ScheduleKeyToggledEvent():
return keyToggled(_that.key);case ScheduleKeysClearedEvent():
return keysCleared();case ScheduleShowDoneChangedEvent():
return showDoneChanged(_that.showDone);case ScheduleChangesReceivedEvent():
return changesReceived(_that.ids);case ScheduleRetriedEvent():
return retried();case ScheduleTickedEvent():
return ticked(_that.row);case ScheduleMovedEvent():
return moved(_that.row,_that.day);case ScheduleNoticeDismissedEvent():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( DayDate today,  int weekStart)?  started,TResult? Function( ScheduleMode mode)?  modeChanged,TResult? Function( int delta)?  monthStepped,TResult? Function( DayDate today)?  todayShown,TResult? Function( DayDate day)?  daySelected,TResult? Function( String notebookId)?  notebookToggled,TResult? Function()?  notebooksCleared,TResult? Function( String key)?  keyToggled,TResult? Function()?  keysCleared,TResult? Function( bool showDone)?  showDoneChanged,TResult? Function( List<String> ids)?  changesReceived,TResult? Function()?  retried,TResult? Function( ScheduleRow row)?  ticked,TResult? Function( ScheduleRow row,  DayDate day)?  moved,TResult? Function()?  noticeDismissed,}) {final _that = this;
switch (_that) {
case ScheduleStartedEvent() when started != null:
return started(_that.today,_that.weekStart);case ScheduleModeChangedEvent() when modeChanged != null:
return modeChanged(_that.mode);case ScheduleMonthSteppedEvent() when monthStepped != null:
return monthStepped(_that.delta);case ScheduleTodayShownEvent() when todayShown != null:
return todayShown(_that.today);case ScheduleDaySelectedEvent() when daySelected != null:
return daySelected(_that.day);case ScheduleNotebookToggledEvent() when notebookToggled != null:
return notebookToggled(_that.notebookId);case ScheduleNotebooksClearedEvent() when notebooksCleared != null:
return notebooksCleared();case ScheduleKeyToggledEvent() when keyToggled != null:
return keyToggled(_that.key);case ScheduleKeysClearedEvent() when keysCleared != null:
return keysCleared();case ScheduleShowDoneChangedEvent() when showDoneChanged != null:
return showDoneChanged(_that.showDone);case ScheduleChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case ScheduleRetriedEvent() when retried != null:
return retried();case ScheduleTickedEvent() when ticked != null:
return ticked(_that.row);case ScheduleMovedEvent() when moved != null:
return moved(_that.row,_that.day);case ScheduleNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case _:
  return null;

}
}

}

/// @nodoc


class ScheduleStartedEvent implements ScheduleEvent {
  const ScheduleStartedEvent({required this.today, required this.weekStart});
  

 final  DayDate today;
 final  int weekStart;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleStartedEventCopyWith<ScheduleStartedEvent> get copyWith => _$ScheduleStartedEventCopyWithImpl<ScheduleStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleStartedEvent&&(identical(other.today, today) || other.today == today)&&(identical(other.weekStart, weekStart) || other.weekStart == weekStart));
}


@override
int get hashCode => Object.hash(runtimeType,today,weekStart);

@override
String toString() {
  return 'ScheduleEvent.started(today: $today, weekStart: $weekStart)';
}


}

/// @nodoc
abstract mixin class $ScheduleStartedEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleStartedEventCopyWith(ScheduleStartedEvent value, $Res Function(ScheduleStartedEvent) _then) = _$ScheduleStartedEventCopyWithImpl;
@useResult
$Res call({
 DayDate today, int weekStart
});




}
/// @nodoc
class _$ScheduleStartedEventCopyWithImpl<$Res>
    implements $ScheduleStartedEventCopyWith<$Res> {
  _$ScheduleStartedEventCopyWithImpl(this._self, this._then);

  final ScheduleStartedEvent _self;
  final $Res Function(ScheduleStartedEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? today = null,Object? weekStart = null,}) {
  return _then(ScheduleStartedEvent(
today: null == today ? _self.today : today // ignore: cast_nullable_to_non_nullable
as DayDate,weekStart: null == weekStart ? _self.weekStart : weekStart // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class ScheduleModeChangedEvent implements ScheduleEvent {
  const ScheduleModeChangedEvent(this.mode);
  

 final  ScheduleMode mode;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleModeChangedEventCopyWith<ScheduleModeChangedEvent> get copyWith => _$ScheduleModeChangedEventCopyWithImpl<ScheduleModeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleModeChangedEvent&&(identical(other.mode, mode) || other.mode == mode));
}


@override
int get hashCode => Object.hash(runtimeType,mode);

@override
String toString() {
  return 'ScheduleEvent.modeChanged(mode: $mode)';
}


}

/// @nodoc
abstract mixin class $ScheduleModeChangedEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleModeChangedEventCopyWith(ScheduleModeChangedEvent value, $Res Function(ScheduleModeChangedEvent) _then) = _$ScheduleModeChangedEventCopyWithImpl;
@useResult
$Res call({
 ScheduleMode mode
});




}
/// @nodoc
class _$ScheduleModeChangedEventCopyWithImpl<$Res>
    implements $ScheduleModeChangedEventCopyWith<$Res> {
  _$ScheduleModeChangedEventCopyWithImpl(this._self, this._then);

  final ScheduleModeChangedEvent _self;
  final $Res Function(ScheduleModeChangedEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? mode = null,}) {
  return _then(ScheduleModeChangedEvent(
null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as ScheduleMode,
  ));
}


}

/// @nodoc


class ScheduleMonthSteppedEvent implements ScheduleEvent {
  const ScheduleMonthSteppedEvent(this.delta);
  

 final  int delta;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleMonthSteppedEventCopyWith<ScheduleMonthSteppedEvent> get copyWith => _$ScheduleMonthSteppedEventCopyWithImpl<ScheduleMonthSteppedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleMonthSteppedEvent&&(identical(other.delta, delta) || other.delta == delta));
}


@override
int get hashCode => Object.hash(runtimeType,delta);

@override
String toString() {
  return 'ScheduleEvent.monthStepped(delta: $delta)';
}


}

/// @nodoc
abstract mixin class $ScheduleMonthSteppedEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleMonthSteppedEventCopyWith(ScheduleMonthSteppedEvent value, $Res Function(ScheduleMonthSteppedEvent) _then) = _$ScheduleMonthSteppedEventCopyWithImpl;
@useResult
$Res call({
 int delta
});




}
/// @nodoc
class _$ScheduleMonthSteppedEventCopyWithImpl<$Res>
    implements $ScheduleMonthSteppedEventCopyWith<$Res> {
  _$ScheduleMonthSteppedEventCopyWithImpl(this._self, this._then);

  final ScheduleMonthSteppedEvent _self;
  final $Res Function(ScheduleMonthSteppedEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? delta = null,}) {
  return _then(ScheduleMonthSteppedEvent(
null == delta ? _self.delta : delta // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class ScheduleTodayShownEvent implements ScheduleEvent {
  const ScheduleTodayShownEvent(this.today);
  

 final  DayDate today;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleTodayShownEventCopyWith<ScheduleTodayShownEvent> get copyWith => _$ScheduleTodayShownEventCopyWithImpl<ScheduleTodayShownEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleTodayShownEvent&&(identical(other.today, today) || other.today == today));
}


@override
int get hashCode => Object.hash(runtimeType,today);

@override
String toString() {
  return 'ScheduleEvent.todayShown(today: $today)';
}


}

/// @nodoc
abstract mixin class $ScheduleTodayShownEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleTodayShownEventCopyWith(ScheduleTodayShownEvent value, $Res Function(ScheduleTodayShownEvent) _then) = _$ScheduleTodayShownEventCopyWithImpl;
@useResult
$Res call({
 DayDate today
});




}
/// @nodoc
class _$ScheduleTodayShownEventCopyWithImpl<$Res>
    implements $ScheduleTodayShownEventCopyWith<$Res> {
  _$ScheduleTodayShownEventCopyWithImpl(this._self, this._then);

  final ScheduleTodayShownEvent _self;
  final $Res Function(ScheduleTodayShownEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? today = null,}) {
  return _then(ScheduleTodayShownEvent(
null == today ? _self.today : today // ignore: cast_nullable_to_non_nullable
as DayDate,
  ));
}


}

/// @nodoc


class ScheduleDaySelectedEvent implements ScheduleEvent {
  const ScheduleDaySelectedEvent(this.day);
  

 final  DayDate day;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleDaySelectedEventCopyWith<ScheduleDaySelectedEvent> get copyWith => _$ScheduleDaySelectedEventCopyWithImpl<ScheduleDaySelectedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleDaySelectedEvent&&(identical(other.day, day) || other.day == day));
}


@override
int get hashCode => Object.hash(runtimeType,day);

@override
String toString() {
  return 'ScheduleEvent.daySelected(day: $day)';
}


}

/// @nodoc
abstract mixin class $ScheduleDaySelectedEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleDaySelectedEventCopyWith(ScheduleDaySelectedEvent value, $Res Function(ScheduleDaySelectedEvent) _then) = _$ScheduleDaySelectedEventCopyWithImpl;
@useResult
$Res call({
 DayDate day
});




}
/// @nodoc
class _$ScheduleDaySelectedEventCopyWithImpl<$Res>
    implements $ScheduleDaySelectedEventCopyWith<$Res> {
  _$ScheduleDaySelectedEventCopyWithImpl(this._self, this._then);

  final ScheduleDaySelectedEvent _self;
  final $Res Function(ScheduleDaySelectedEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? day = null,}) {
  return _then(ScheduleDaySelectedEvent(
null == day ? _self.day : day // ignore: cast_nullable_to_non_nullable
as DayDate,
  ));
}


}

/// @nodoc


class ScheduleNotebookToggledEvent implements ScheduleEvent {
  const ScheduleNotebookToggledEvent(this.notebookId);
  

 final  String notebookId;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleNotebookToggledEventCopyWith<ScheduleNotebookToggledEvent> get copyWith => _$ScheduleNotebookToggledEventCopyWithImpl<ScheduleNotebookToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleNotebookToggledEvent&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'ScheduleEvent.notebookToggled(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $ScheduleNotebookToggledEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleNotebookToggledEventCopyWith(ScheduleNotebookToggledEvent value, $Res Function(ScheduleNotebookToggledEvent) _then) = _$ScheduleNotebookToggledEventCopyWithImpl;
@useResult
$Res call({
 String notebookId
});




}
/// @nodoc
class _$ScheduleNotebookToggledEventCopyWithImpl<$Res>
    implements $ScheduleNotebookToggledEventCopyWith<$Res> {
  _$ScheduleNotebookToggledEventCopyWithImpl(this._self, this._then);

  final ScheduleNotebookToggledEvent _self;
  final $Res Function(ScheduleNotebookToggledEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notebookId = null,}) {
  return _then(ScheduleNotebookToggledEvent(
null == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ScheduleNotebooksClearedEvent implements ScheduleEvent {
  const ScheduleNotebooksClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleNotebooksClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ScheduleEvent.notebooksCleared()';
}


}




/// @nodoc


class ScheduleKeyToggledEvent implements ScheduleEvent {
  const ScheduleKeyToggledEvent(this.key);
  

 final  String key;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleKeyToggledEventCopyWith<ScheduleKeyToggledEvent> get copyWith => _$ScheduleKeyToggledEventCopyWithImpl<ScheduleKeyToggledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleKeyToggledEvent&&(identical(other.key, key) || other.key == key));
}


@override
int get hashCode => Object.hash(runtimeType,key);

@override
String toString() {
  return 'ScheduleEvent.keyToggled(key: $key)';
}


}

/// @nodoc
abstract mixin class $ScheduleKeyToggledEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleKeyToggledEventCopyWith(ScheduleKeyToggledEvent value, $Res Function(ScheduleKeyToggledEvent) _then) = _$ScheduleKeyToggledEventCopyWithImpl;
@useResult
$Res call({
 String key
});




}
/// @nodoc
class _$ScheduleKeyToggledEventCopyWithImpl<$Res>
    implements $ScheduleKeyToggledEventCopyWith<$Res> {
  _$ScheduleKeyToggledEventCopyWithImpl(this._self, this._then);

  final ScheduleKeyToggledEvent _self;
  final $Res Function(ScheduleKeyToggledEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = null,}) {
  return _then(ScheduleKeyToggledEvent(
null == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ScheduleKeysClearedEvent implements ScheduleEvent {
  const ScheduleKeysClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleKeysClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ScheduleEvent.keysCleared()';
}


}




/// @nodoc


class ScheduleShowDoneChangedEvent implements ScheduleEvent {
  const ScheduleShowDoneChangedEvent({required this.showDone});
  

 final  bool showDone;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleShowDoneChangedEventCopyWith<ScheduleShowDoneChangedEvent> get copyWith => _$ScheduleShowDoneChangedEventCopyWithImpl<ScheduleShowDoneChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleShowDoneChangedEvent&&(identical(other.showDone, showDone) || other.showDone == showDone));
}


@override
int get hashCode => Object.hash(runtimeType,showDone);

@override
String toString() {
  return 'ScheduleEvent.showDoneChanged(showDone: $showDone)';
}


}

/// @nodoc
abstract mixin class $ScheduleShowDoneChangedEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleShowDoneChangedEventCopyWith(ScheduleShowDoneChangedEvent value, $Res Function(ScheduleShowDoneChangedEvent) _then) = _$ScheduleShowDoneChangedEventCopyWithImpl;
@useResult
$Res call({
 bool showDone
});




}
/// @nodoc
class _$ScheduleShowDoneChangedEventCopyWithImpl<$Res>
    implements $ScheduleShowDoneChangedEventCopyWith<$Res> {
  _$ScheduleShowDoneChangedEventCopyWithImpl(this._self, this._then);

  final ScheduleShowDoneChangedEvent _self;
  final $Res Function(ScheduleShowDoneChangedEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? showDone = null,}) {
  return _then(ScheduleShowDoneChangedEvent(
showDone: null == showDone ? _self.showDone : showDone // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class ScheduleChangesReceivedEvent implements ScheduleEvent {
  const ScheduleChangesReceivedEvent(final  List<String> ids): _ids = ids;
  

 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleChangesReceivedEventCopyWith<ScheduleChangesReceivedEvent> get copyWith => _$ScheduleChangesReceivedEventCopyWithImpl<ScheduleChangesReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleChangesReceivedEvent&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'ScheduleEvent.changesReceived(ids: $ids)';
}


}

/// @nodoc
abstract mixin class $ScheduleChangesReceivedEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleChangesReceivedEventCopyWith(ScheduleChangesReceivedEvent value, $Res Function(ScheduleChangesReceivedEvent) _then) = _$ScheduleChangesReceivedEventCopyWithImpl;
@useResult
$Res call({
 List<String> ids
});




}
/// @nodoc
class _$ScheduleChangesReceivedEventCopyWithImpl<$Res>
    implements $ScheduleChangesReceivedEventCopyWith<$Res> {
  _$ScheduleChangesReceivedEventCopyWithImpl(this._self, this._then);

  final ScheduleChangesReceivedEvent _self;
  final $Res Function(ScheduleChangesReceivedEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ids = null,}) {
  return _then(ScheduleChangesReceivedEvent(
null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class ScheduleRetriedEvent implements ScheduleEvent {
  const ScheduleRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ScheduleEvent.retried()';
}


}




/// @nodoc


class ScheduleTickedEvent implements ScheduleEvent {
  const ScheduleTickedEvent(this.row);
  

 final  ScheduleRow row;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleTickedEventCopyWith<ScheduleTickedEvent> get copyWith => _$ScheduleTickedEventCopyWithImpl<ScheduleTickedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleTickedEvent&&const DeepCollectionEquality().equals(other.row, row));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(row));

@override
String toString() {
  return 'ScheduleEvent.ticked(row: $row)';
}


}

/// @nodoc
abstract mixin class $ScheduleTickedEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleTickedEventCopyWith(ScheduleTickedEvent value, $Res Function(ScheduleTickedEvent) _then) = _$ScheduleTickedEventCopyWithImpl;
@useResult
$Res call({
 ScheduleRow row
});




}
/// @nodoc
class _$ScheduleTickedEventCopyWithImpl<$Res>
    implements $ScheduleTickedEventCopyWith<$Res> {
  _$ScheduleTickedEventCopyWithImpl(this._self, this._then);

  final ScheduleTickedEvent _self;
  final $Res Function(ScheduleTickedEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? row = freezed,}) {
  return _then(ScheduleTickedEvent(
freezed == row ? _self.row : row // ignore: cast_nullable_to_non_nullable
as ScheduleRow,
  ));
}


}

/// @nodoc


class ScheduleMovedEvent implements ScheduleEvent {
  const ScheduleMovedEvent(this.row, this.day);
  

 final  ScheduleRow row;
 final  DayDate day;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleMovedEventCopyWith<ScheduleMovedEvent> get copyWith => _$ScheduleMovedEventCopyWithImpl<ScheduleMovedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleMovedEvent&&const DeepCollectionEquality().equals(other.row, row)&&(identical(other.day, day) || other.day == day));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(row),day);

@override
String toString() {
  return 'ScheduleEvent.moved(row: $row, day: $day)';
}


}

/// @nodoc
abstract mixin class $ScheduleMovedEventCopyWith<$Res> implements $ScheduleEventCopyWith<$Res> {
  factory $ScheduleMovedEventCopyWith(ScheduleMovedEvent value, $Res Function(ScheduleMovedEvent) _then) = _$ScheduleMovedEventCopyWithImpl;
@useResult
$Res call({
 ScheduleRow row, DayDate day
});




}
/// @nodoc
class _$ScheduleMovedEventCopyWithImpl<$Res>
    implements $ScheduleMovedEventCopyWith<$Res> {
  _$ScheduleMovedEventCopyWithImpl(this._self, this._then);

  final ScheduleMovedEvent _self;
  final $Res Function(ScheduleMovedEvent) _then;

/// Create a copy of ScheduleEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? row = freezed,Object? day = null,}) {
  return _then(ScheduleMovedEvent(
freezed == row ? _self.row : row // ignore: cast_nullable_to_non_nullable
as ScheduleRow,null == day ? _self.day : day // ignore: cast_nullable_to_non_nullable
as DayDate,
  ));
}


}

/// @nodoc


class ScheduleNoticeDismissedEvent implements ScheduleEvent {
  const ScheduleNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ScheduleEvent.noticeDismissed()';
}


}




/// @nodoc
mixin _$ScheduleState {

 ScheduleStatus get status; ScheduleMode get mode;/// A day of the month shown.
 DayDate get focused;/// The day whose items the month view lists.
 DayDate get selected;/// Today, as the page told it.
 DayDate get today;/// The weekday weeks start on: 0 for Monday through 6 for Sunday.
 int get weekStart;/// Only these notebooks' tasks and notes; every notebook when empty.
 List<String> get books;/// Only these date properties place a note; every one when empty.
 List<String> get keys;/// Whether done tasks show, as the desktop's Show Completed does.
 bool get showDone;/// The date properties the notebooks in view define.
 List<String> get dateKeys;/// What falls on the days of the month grid.
 List<ScheduleRow> get rows;/// Why reading failed, while [status] is failed.
 ApplicationError? get error;/// What went wrong with the last tick or move, until it is dismissed.
 ScheduleNotice? get notice;
/// Create a copy of ScheduleState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ScheduleStateCopyWith<ScheduleState> get copyWith => _$ScheduleStateCopyWithImpl<ScheduleState>(this as ScheduleState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ScheduleState&&(identical(other.status, status) || other.status == status)&&(identical(other.mode, mode) || other.mode == mode)&&(identical(other.focused, focused) || other.focused == focused)&&(identical(other.selected, selected) || other.selected == selected)&&(identical(other.today, today) || other.today == today)&&(identical(other.weekStart, weekStart) || other.weekStart == weekStart)&&const DeepCollectionEquality().equals(other.books, books)&&const DeepCollectionEquality().equals(other.keys, keys)&&(identical(other.showDone, showDone) || other.showDone == showDone)&&const DeepCollectionEquality().equals(other.dateKeys, dateKeys)&&const DeepCollectionEquality().equals(other.rows, rows)&&(identical(other.error, error) || other.error == error)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,status,mode,focused,selected,today,weekStart,const DeepCollectionEquality().hash(books),const DeepCollectionEquality().hash(keys),showDone,const DeepCollectionEquality().hash(dateKeys),const DeepCollectionEquality().hash(rows),error,notice);

@override
String toString() {
  return 'ScheduleState(status: $status, mode: $mode, focused: $focused, selected: $selected, today: $today, weekStart: $weekStart, books: $books, keys: $keys, showDone: $showDone, dateKeys: $dateKeys, rows: $rows, error: $error, notice: $notice)';
}


}

/// @nodoc
abstract mixin class $ScheduleStateCopyWith<$Res>  {
  factory $ScheduleStateCopyWith(ScheduleState value, $Res Function(ScheduleState) _then) = _$ScheduleStateCopyWithImpl;
@useResult
$Res call({
 ScheduleStatus status, ScheduleMode mode, DayDate focused, DayDate selected, DayDate today, int weekStart, List<String> books, List<String> keys, bool showDone, List<String> dateKeys, List<ScheduleRow> rows, ApplicationError? error, ScheduleNotice? notice
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$ScheduleStateCopyWithImpl<$Res>
    implements $ScheduleStateCopyWith<$Res> {
  _$ScheduleStateCopyWithImpl(this._self, this._then);

  final ScheduleState _self;
  final $Res Function(ScheduleState) _then;

/// Create a copy of ScheduleState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? mode = null,Object? focused = null,Object? selected = null,Object? today = null,Object? weekStart = null,Object? books = null,Object? keys = null,Object? showDone = null,Object? dateKeys = null,Object? rows = null,Object? error = freezed,Object? notice = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as ScheduleStatus,mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as ScheduleMode,focused: null == focused ? _self.focused : focused // ignore: cast_nullable_to_non_nullable
as DayDate,selected: null == selected ? _self.selected : selected // ignore: cast_nullable_to_non_nullable
as DayDate,today: null == today ? _self.today : today // ignore: cast_nullable_to_non_nullable
as DayDate,weekStart: null == weekStart ? _self.weekStart : weekStart // ignore: cast_nullable_to_non_nullable
as int,books: null == books ? _self.books : books // ignore: cast_nullable_to_non_nullable
as List<String>,keys: null == keys ? _self.keys : keys // ignore: cast_nullable_to_non_nullable
as List<String>,showDone: null == showDone ? _self.showDone : showDone // ignore: cast_nullable_to_non_nullable
as bool,dateKeys: null == dateKeys ? _self.dateKeys : dateKeys // ignore: cast_nullable_to_non_nullable
as List<String>,rows: null == rows ? _self.rows : rows // ignore: cast_nullable_to_non_nullable
as List<ScheduleRow>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as ScheduleNotice?,
  ));
}
/// Create a copy of ScheduleState
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


/// Adds pattern-matching-related methods to [ScheduleState].
extension ScheduleStatePatterns on ScheduleState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ScheduleState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ScheduleState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ScheduleState value)  $default,){
final _that = this;
switch (_that) {
case _ScheduleState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ScheduleState value)?  $default,){
final _that = this;
switch (_that) {
case _ScheduleState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( ScheduleStatus status,  ScheduleMode mode,  DayDate focused,  DayDate selected,  DayDate today,  int weekStart,  List<String> books,  List<String> keys,  bool showDone,  List<String> dateKeys,  List<ScheduleRow> rows,  ApplicationError? error,  ScheduleNotice? notice)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ScheduleState() when $default != null:
return $default(_that.status,_that.mode,_that.focused,_that.selected,_that.today,_that.weekStart,_that.books,_that.keys,_that.showDone,_that.dateKeys,_that.rows,_that.error,_that.notice);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( ScheduleStatus status,  ScheduleMode mode,  DayDate focused,  DayDate selected,  DayDate today,  int weekStart,  List<String> books,  List<String> keys,  bool showDone,  List<String> dateKeys,  List<ScheduleRow> rows,  ApplicationError? error,  ScheduleNotice? notice)  $default,) {final _that = this;
switch (_that) {
case _ScheduleState():
return $default(_that.status,_that.mode,_that.focused,_that.selected,_that.today,_that.weekStart,_that.books,_that.keys,_that.showDone,_that.dateKeys,_that.rows,_that.error,_that.notice);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( ScheduleStatus status,  ScheduleMode mode,  DayDate focused,  DayDate selected,  DayDate today,  int weekStart,  List<String> books,  List<String> keys,  bool showDone,  List<String> dateKeys,  List<ScheduleRow> rows,  ApplicationError? error,  ScheduleNotice? notice)?  $default,) {final _that = this;
switch (_that) {
case _ScheduleState() when $default != null:
return $default(_that.status,_that.mode,_that.focused,_that.selected,_that.today,_that.weekStart,_that.books,_that.keys,_that.showDone,_that.dateKeys,_that.rows,_that.error,_that.notice);case _:
  return null;

}
}

}

/// @nodoc


class _ScheduleState extends ScheduleState {
  const _ScheduleState({this.status = ScheduleStatus.loading, this.mode = ScheduleMode.month, this.focused = const DayDate(year: 1970, month: 1, day: 1), this.selected = const DayDate(year: 1970, month: 1, day: 1), this.today = const DayDate(year: 1970, month: 1, day: 1), this.weekStart = 0, final  List<String> books = const <String>[], final  List<String> keys = const <String>[], this.showDone = true, final  List<String> dateKeys = const <String>[], final  List<ScheduleRow> rows = const <ScheduleRow>[], this.error, this.notice}): _books = books,_keys = keys,_dateKeys = dateKeys,_rows = rows,super._();
  

@override@JsonKey() final  ScheduleStatus status;
@override@JsonKey() final  ScheduleMode mode;
/// A day of the month shown.
@override@JsonKey() final  DayDate focused;
/// The day whose items the month view lists.
@override@JsonKey() final  DayDate selected;
/// Today, as the page told it.
@override@JsonKey() final  DayDate today;
/// The weekday weeks start on: 0 for Monday through 6 for Sunday.
@override@JsonKey() final  int weekStart;
/// Only these notebooks' tasks and notes; every notebook when empty.
 final  List<String> _books;
/// Only these notebooks' tasks and notes; every notebook when empty.
@override@JsonKey() List<String> get books {
  if (_books is EqualUnmodifiableListView) return _books;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_books);
}

/// Only these date properties place a note; every one when empty.
 final  List<String> _keys;
/// Only these date properties place a note; every one when empty.
@override@JsonKey() List<String> get keys {
  if (_keys is EqualUnmodifiableListView) return _keys;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_keys);
}

/// Whether done tasks show, as the desktop's Show Completed does.
@override@JsonKey() final  bool showDone;
/// The date properties the notebooks in view define.
 final  List<String> _dateKeys;
/// The date properties the notebooks in view define.
@override@JsonKey() List<String> get dateKeys {
  if (_dateKeys is EqualUnmodifiableListView) return _dateKeys;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_dateKeys);
}

/// What falls on the days of the month grid.
 final  List<ScheduleRow> _rows;
/// What falls on the days of the month grid.
@override@JsonKey() List<ScheduleRow> get rows {
  if (_rows is EqualUnmodifiableListView) return _rows;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_rows);
}

/// Why reading failed, while [status] is failed.
@override final  ApplicationError? error;
/// What went wrong with the last tick or move, until it is dismissed.
@override final  ScheduleNotice? notice;

/// Create a copy of ScheduleState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ScheduleStateCopyWith<_ScheduleState> get copyWith => __$ScheduleStateCopyWithImpl<_ScheduleState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ScheduleState&&(identical(other.status, status) || other.status == status)&&(identical(other.mode, mode) || other.mode == mode)&&(identical(other.focused, focused) || other.focused == focused)&&(identical(other.selected, selected) || other.selected == selected)&&(identical(other.today, today) || other.today == today)&&(identical(other.weekStart, weekStart) || other.weekStart == weekStart)&&const DeepCollectionEquality().equals(other._books, _books)&&const DeepCollectionEquality().equals(other._keys, _keys)&&(identical(other.showDone, showDone) || other.showDone == showDone)&&const DeepCollectionEquality().equals(other._dateKeys, _dateKeys)&&const DeepCollectionEquality().equals(other._rows, _rows)&&(identical(other.error, error) || other.error == error)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,status,mode,focused,selected,today,weekStart,const DeepCollectionEquality().hash(_books),const DeepCollectionEquality().hash(_keys),showDone,const DeepCollectionEquality().hash(_dateKeys),const DeepCollectionEquality().hash(_rows),error,notice);

@override
String toString() {
  return 'ScheduleState(status: $status, mode: $mode, focused: $focused, selected: $selected, today: $today, weekStart: $weekStart, books: $books, keys: $keys, showDone: $showDone, dateKeys: $dateKeys, rows: $rows, error: $error, notice: $notice)';
}


}

/// @nodoc
abstract mixin class _$ScheduleStateCopyWith<$Res> implements $ScheduleStateCopyWith<$Res> {
  factory _$ScheduleStateCopyWith(_ScheduleState value, $Res Function(_ScheduleState) _then) = __$ScheduleStateCopyWithImpl;
@override @useResult
$Res call({
 ScheduleStatus status, ScheduleMode mode, DayDate focused, DayDate selected, DayDate today, int weekStart, List<String> books, List<String> keys, bool showDone, List<String> dateKeys, List<ScheduleRow> rows, ApplicationError? error, ScheduleNotice? notice
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$ScheduleStateCopyWithImpl<$Res>
    implements _$ScheduleStateCopyWith<$Res> {
  __$ScheduleStateCopyWithImpl(this._self, this._then);

  final _ScheduleState _self;
  final $Res Function(_ScheduleState) _then;

/// Create a copy of ScheduleState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? mode = null,Object? focused = null,Object? selected = null,Object? today = null,Object? weekStart = null,Object? books = null,Object? keys = null,Object? showDone = null,Object? dateKeys = null,Object? rows = null,Object? error = freezed,Object? notice = freezed,}) {
  return _then(_ScheduleState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as ScheduleStatus,mode: null == mode ? _self.mode : mode // ignore: cast_nullable_to_non_nullable
as ScheduleMode,focused: null == focused ? _self.focused : focused // ignore: cast_nullable_to_non_nullable
as DayDate,selected: null == selected ? _self.selected : selected // ignore: cast_nullable_to_non_nullable
as DayDate,today: null == today ? _self.today : today // ignore: cast_nullable_to_non_nullable
as DayDate,weekStart: null == weekStart ? _self.weekStart : weekStart // ignore: cast_nullable_to_non_nullable
as int,books: null == books ? _self._books : books // ignore: cast_nullable_to_non_nullable
as List<String>,keys: null == keys ? _self._keys : keys // ignore: cast_nullable_to_non_nullable
as List<String>,showDone: null == showDone ? _self.showDone : showDone // ignore: cast_nullable_to_non_nullable
as bool,dateKeys: null == dateKeys ? _self._dateKeys : dateKeys // ignore: cast_nullable_to_non_nullable
as List<String>,rows: null == rows ? _self._rows : rows // ignore: cast_nullable_to_non_nullable
as List<ScheduleRow>,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as ScheduleNotice?,
  ));
}

/// Create a copy of ScheduleState
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

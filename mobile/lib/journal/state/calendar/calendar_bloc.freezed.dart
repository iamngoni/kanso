// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'calendar_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CalendarEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CalendarEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CalendarEvent()';
}


}

/// @nodoc
class $CalendarEventCopyWith<$Res>  {
$CalendarEventCopyWith(CalendarEvent _, $Res Function(CalendarEvent) __);
}


/// Adds pattern-matching-related methods to [CalendarEvent].
extension CalendarEventPatterns on CalendarEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CalendarStartedEvent value)?  started,TResult Function( CalendarMonthSteppedEvent value)?  monthStepped,TResult Function( CalendarTodayShownEvent value)?  todayShown,TResult Function( CalendarRetriedEvent value)?  retried,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CalendarStartedEvent() when started != null:
return started(_that);case CalendarMonthSteppedEvent() when monthStepped != null:
return monthStepped(_that);case CalendarTodayShownEvent() when todayShown != null:
return todayShown(_that);case CalendarRetriedEvent() when retried != null:
return retried(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CalendarStartedEvent value)  started,required TResult Function( CalendarMonthSteppedEvent value)  monthStepped,required TResult Function( CalendarTodayShownEvent value)  todayShown,required TResult Function( CalendarRetriedEvent value)  retried,}){
final _that = this;
switch (_that) {
case CalendarStartedEvent():
return started(_that);case CalendarMonthSteppedEvent():
return monthStepped(_that);case CalendarTodayShownEvent():
return todayShown(_that);case CalendarRetriedEvent():
return retried(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CalendarStartedEvent value)?  started,TResult? Function( CalendarMonthSteppedEvent value)?  monthStepped,TResult? Function( CalendarTodayShownEvent value)?  todayShown,TResult? Function( CalendarRetriedEvent value)?  retried,}){
final _that = this;
switch (_that) {
case CalendarStartedEvent() when started != null:
return started(_that);case CalendarMonthSteppedEvent() when monthStepped != null:
return monthStepped(_that);case CalendarTodayShownEvent() when todayShown != null:
return todayShown(_that);case CalendarRetriedEvent() when retried != null:
return retried(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String? openNote,  DayDate today,  int weekStart)?  started,TResult Function( int by)?  monthStepped,TResult Function()?  todayShown,TResult Function()?  retried,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CalendarStartedEvent() when started != null:
return started(_that.openNote,_that.today,_that.weekStart);case CalendarMonthSteppedEvent() when monthStepped != null:
return monthStepped(_that.by);case CalendarTodayShownEvent() when todayShown != null:
return todayShown();case CalendarRetriedEvent() when retried != null:
return retried();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String? openNote,  DayDate today,  int weekStart)  started,required TResult Function( int by)  monthStepped,required TResult Function()  todayShown,required TResult Function()  retried,}) {final _that = this;
switch (_that) {
case CalendarStartedEvent():
return started(_that.openNote,_that.today,_that.weekStart);case CalendarMonthSteppedEvent():
return monthStepped(_that.by);case CalendarTodayShownEvent():
return todayShown();case CalendarRetriedEvent():
return retried();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String? openNote,  DayDate today,  int weekStart)?  started,TResult? Function( int by)?  monthStepped,TResult? Function()?  todayShown,TResult? Function()?  retried,}) {final _that = this;
switch (_that) {
case CalendarStartedEvent() when started != null:
return started(_that.openNote,_that.today,_that.weekStart);case CalendarMonthSteppedEvent() when monthStepped != null:
return monthStepped(_that.by);case CalendarTodayShownEvent() when todayShown != null:
return todayShown();case CalendarRetriedEvent() when retried != null:
return retried();case _:
  return null;

}
}

}

/// @nodoc


class CalendarStartedEvent implements CalendarEvent {
  const CalendarStartedEvent({required this.openNote, required this.today, required this.weekStart});
  

 final  String? openNote;
 final  DayDate today;
 final  int weekStart;

/// Create a copy of CalendarEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CalendarStartedEventCopyWith<CalendarStartedEvent> get copyWith => _$CalendarStartedEventCopyWithImpl<CalendarStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CalendarStartedEvent&&(identical(other.openNote, openNote) || other.openNote == openNote)&&(identical(other.today, today) || other.today == today)&&(identical(other.weekStart, weekStart) || other.weekStart == weekStart));
}


@override
int get hashCode => Object.hash(runtimeType,openNote,today,weekStart);

@override
String toString() {
  return 'CalendarEvent.started(openNote: $openNote, today: $today, weekStart: $weekStart)';
}


}

/// @nodoc
abstract mixin class $CalendarStartedEventCopyWith<$Res> implements $CalendarEventCopyWith<$Res> {
  factory $CalendarStartedEventCopyWith(CalendarStartedEvent value, $Res Function(CalendarStartedEvent) _then) = _$CalendarStartedEventCopyWithImpl;
@useResult
$Res call({
 String? openNote, DayDate today, int weekStart
});




}
/// @nodoc
class _$CalendarStartedEventCopyWithImpl<$Res>
    implements $CalendarStartedEventCopyWith<$Res> {
  _$CalendarStartedEventCopyWithImpl(this._self, this._then);

  final CalendarStartedEvent _self;
  final $Res Function(CalendarStartedEvent) _then;

/// Create a copy of CalendarEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? openNote = freezed,Object? today = null,Object? weekStart = null,}) {
  return _then(CalendarStartedEvent(
openNote: freezed == openNote ? _self.openNote : openNote // ignore: cast_nullable_to_non_nullable
as String?,today: null == today ? _self.today : today // ignore: cast_nullable_to_non_nullable
as DayDate,weekStart: null == weekStart ? _self.weekStart : weekStart // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class CalendarMonthSteppedEvent implements CalendarEvent {
  const CalendarMonthSteppedEvent(this.by);
  

 final  int by;

/// Create a copy of CalendarEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CalendarMonthSteppedEventCopyWith<CalendarMonthSteppedEvent> get copyWith => _$CalendarMonthSteppedEventCopyWithImpl<CalendarMonthSteppedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CalendarMonthSteppedEvent&&(identical(other.by, by) || other.by == by));
}


@override
int get hashCode => Object.hash(runtimeType,by);

@override
String toString() {
  return 'CalendarEvent.monthStepped(by: $by)';
}


}

/// @nodoc
abstract mixin class $CalendarMonthSteppedEventCopyWith<$Res> implements $CalendarEventCopyWith<$Res> {
  factory $CalendarMonthSteppedEventCopyWith(CalendarMonthSteppedEvent value, $Res Function(CalendarMonthSteppedEvent) _then) = _$CalendarMonthSteppedEventCopyWithImpl;
@useResult
$Res call({
 int by
});




}
/// @nodoc
class _$CalendarMonthSteppedEventCopyWithImpl<$Res>
    implements $CalendarMonthSteppedEventCopyWith<$Res> {
  _$CalendarMonthSteppedEventCopyWithImpl(this._self, this._then);

  final CalendarMonthSteppedEvent _self;
  final $Res Function(CalendarMonthSteppedEvent) _then;

/// Create a copy of CalendarEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? by = null,}) {
  return _then(CalendarMonthSteppedEvent(
null == by ? _self.by : by // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class CalendarTodayShownEvent implements CalendarEvent {
  const CalendarTodayShownEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CalendarTodayShownEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CalendarEvent.todayShown()';
}


}




/// @nodoc


class CalendarRetriedEvent implements CalendarEvent {
  const CalendarRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CalendarRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CalendarEvent.retried()';
}


}




/// @nodoc
mixin _$CalendarState {

 LoadStatus get status;/// The month shown.
 int get year; int get month;/// Today, which the grid marks.
 DayDate get today;/// The first day of the week: 0 is Monday.
 int get weekStart;/// The month as read; null until it is. It keeps showing the last month
/// read while the next is.
 MonthGrid? get grid; ApplicationError? get error;
/// Create a copy of CalendarState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CalendarStateCopyWith<CalendarState> get copyWith => _$CalendarStateCopyWithImpl<CalendarState>(this as CalendarState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CalendarState&&(identical(other.status, status) || other.status == status)&&(identical(other.year, year) || other.year == year)&&(identical(other.month, month) || other.month == month)&&(identical(other.today, today) || other.today == today)&&(identical(other.weekStart, weekStart) || other.weekStart == weekStart)&&(identical(other.grid, grid) || other.grid == grid)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,year,month,today,weekStart,grid,error);

@override
String toString() {
  return 'CalendarState(status: $status, year: $year, month: $month, today: $today, weekStart: $weekStart, grid: $grid, error: $error)';
}


}

/// @nodoc
abstract mixin class $CalendarStateCopyWith<$Res>  {
  factory $CalendarStateCopyWith(CalendarState value, $Res Function(CalendarState) _then) = _$CalendarStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, int year, int month, DayDate today, int weekStart, MonthGrid? grid, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$CalendarStateCopyWithImpl<$Res>
    implements $CalendarStateCopyWith<$Res> {
  _$CalendarStateCopyWithImpl(this._self, this._then);

  final CalendarState _self;
  final $Res Function(CalendarState) _then;

/// Create a copy of CalendarState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? year = null,Object? month = null,Object? today = null,Object? weekStart = null,Object? grid = freezed,Object? error = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,year: null == year ? _self.year : year // ignore: cast_nullable_to_non_nullable
as int,month: null == month ? _self.month : month // ignore: cast_nullable_to_non_nullable
as int,today: null == today ? _self.today : today // ignore: cast_nullable_to_non_nullable
as DayDate,weekStart: null == weekStart ? _self.weekStart : weekStart // ignore: cast_nullable_to_non_nullable
as int,grid: freezed == grid ? _self.grid : grid // ignore: cast_nullable_to_non_nullable
as MonthGrid?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of CalendarState
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


/// Adds pattern-matching-related methods to [CalendarState].
extension CalendarStatePatterns on CalendarState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CalendarState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CalendarState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CalendarState value)  $default,){
final _that = this;
switch (_that) {
case _CalendarState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CalendarState value)?  $default,){
final _that = this;
switch (_that) {
case _CalendarState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  int year,  int month,  DayDate today,  int weekStart,  MonthGrid? grid,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CalendarState() when $default != null:
return $default(_that.status,_that.year,_that.month,_that.today,_that.weekStart,_that.grid,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  int year,  int month,  DayDate today,  int weekStart,  MonthGrid? grid,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _CalendarState():
return $default(_that.status,_that.year,_that.month,_that.today,_that.weekStart,_that.grid,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  int year,  int month,  DayDate today,  int weekStart,  MonthGrid? grid,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _CalendarState() when $default != null:
return $default(_that.status,_that.year,_that.month,_that.today,_that.weekStart,_that.grid,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _CalendarState implements CalendarState {
  const _CalendarState({this.status = LoadStatus.loading, this.year = 2000, this.month = 1, this.today = const DayDate(year: 2000, month: 1, day: 1), this.weekStart = 0, this.grid, this.error});
  

@override@JsonKey() final  LoadStatus status;
/// The month shown.
@override@JsonKey() final  int year;
@override@JsonKey() final  int month;
/// Today, which the grid marks.
@override@JsonKey() final  DayDate today;
/// The first day of the week: 0 is Monday.
@override@JsonKey() final  int weekStart;
/// The month as read; null until it is. It keeps showing the last month
/// read while the next is.
@override final  MonthGrid? grid;
@override final  ApplicationError? error;

/// Create a copy of CalendarState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CalendarStateCopyWith<_CalendarState> get copyWith => __$CalendarStateCopyWithImpl<_CalendarState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CalendarState&&(identical(other.status, status) || other.status == status)&&(identical(other.year, year) || other.year == year)&&(identical(other.month, month) || other.month == month)&&(identical(other.today, today) || other.today == today)&&(identical(other.weekStart, weekStart) || other.weekStart == weekStart)&&(identical(other.grid, grid) || other.grid == grid)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,status,year,month,today,weekStart,grid,error);

@override
String toString() {
  return 'CalendarState(status: $status, year: $year, month: $month, today: $today, weekStart: $weekStart, grid: $grid, error: $error)';
}


}

/// @nodoc
abstract mixin class _$CalendarStateCopyWith<$Res> implements $CalendarStateCopyWith<$Res> {
  factory _$CalendarStateCopyWith(_CalendarState value, $Res Function(_CalendarState) _then) = __$CalendarStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, int year, int month, DayDate today, int weekStart, MonthGrid? grid, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$CalendarStateCopyWithImpl<$Res>
    implements _$CalendarStateCopyWith<$Res> {
  __$CalendarStateCopyWithImpl(this._self, this._then);

  final _CalendarState _self;
  final $Res Function(_CalendarState) _then;

/// Create a copy of CalendarState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? year = null,Object? month = null,Object? today = null,Object? weekStart = null,Object? grid = freezed,Object? error = freezed,}) {
  return _then(_CalendarState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,year: null == year ? _self.year : year // ignore: cast_nullable_to_non_nullable
as int,month: null == month ? _self.month : month // ignore: cast_nullable_to_non_nullable
as int,today: null == today ? _self.today : today // ignore: cast_nullable_to_non_nullable
as DayDate,weekStart: null == weekStart ? _self.weekStart : weekStart // ignore: cast_nullable_to_non_nullable
as int,grid: freezed == grid ? _self.grid : grid // ignore: cast_nullable_to_non_nullable
as MonthGrid?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of CalendarState
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

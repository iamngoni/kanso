// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'system_calendar.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SystemCalendar {

 String get id; String get name; String get color; bool get enabled;
/// Create a copy of SystemCalendar
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SystemCalendarCopyWith<SystemCalendar> get copyWith => _$SystemCalendarCopyWithImpl<SystemCalendar>(this as SystemCalendar, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SystemCalendar&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.color, color) || other.color == color)&&(identical(other.enabled, enabled) || other.enabled == enabled));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,color,enabled);

@override
String toString() {
  return 'SystemCalendar(id: $id, name: $name, color: $color, enabled: $enabled)';
}


}

/// @nodoc
abstract mixin class $SystemCalendarCopyWith<$Res>  {
  factory $SystemCalendarCopyWith(SystemCalendar value, $Res Function(SystemCalendar) _then) = _$SystemCalendarCopyWithImpl;
@useResult
$Res call({
 String id, String name, String color, bool enabled
});




}
/// @nodoc
class _$SystemCalendarCopyWithImpl<$Res>
    implements $SystemCalendarCopyWith<$Res> {
  _$SystemCalendarCopyWithImpl(this._self, this._then);

  final SystemCalendar _self;
  final $Res Function(SystemCalendar) _then;

/// Create a copy of SystemCalendar
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? name = null,Object? color = null,Object? enabled = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,color: null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as String,enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [SystemCalendar].
extension SystemCalendarPatterns on SystemCalendar {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SystemCalendar value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SystemCalendar() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SystemCalendar value)  $default,){
final _that = this;
switch (_that) {
case _SystemCalendar():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SystemCalendar value)?  $default,){
final _that = this;
switch (_that) {
case _SystemCalendar() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String name,  String color,  bool enabled)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SystemCalendar() when $default != null:
return $default(_that.id,_that.name,_that.color,_that.enabled);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String name,  String color,  bool enabled)  $default,) {final _that = this;
switch (_that) {
case _SystemCalendar():
return $default(_that.id,_that.name,_that.color,_that.enabled);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String name,  String color,  bool enabled)?  $default,) {final _that = this;
switch (_that) {
case _SystemCalendar() when $default != null:
return $default(_that.id,_that.name,_that.color,_that.enabled);case _:
  return null;

}
}

}

/// @nodoc


class _SystemCalendar implements SystemCalendar {
  const _SystemCalendar({required this.id, required this.name, required this.color, required this.enabled});
  

@override final  String id;
@override final  String name;
@override final  String color;
@override final  bool enabled;

/// Create a copy of SystemCalendar
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SystemCalendarCopyWith<_SystemCalendar> get copyWith => __$SystemCalendarCopyWithImpl<_SystemCalendar>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SystemCalendar&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.color, color) || other.color == color)&&(identical(other.enabled, enabled) || other.enabled == enabled));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,color,enabled);

@override
String toString() {
  return 'SystemCalendar(id: $id, name: $name, color: $color, enabled: $enabled)';
}


}

/// @nodoc
abstract mixin class _$SystemCalendarCopyWith<$Res> implements $SystemCalendarCopyWith<$Res> {
  factory _$SystemCalendarCopyWith(_SystemCalendar value, $Res Function(_SystemCalendar) _then) = __$SystemCalendarCopyWithImpl;
@override @useResult
$Res call({
 String id, String name, String color, bool enabled
});




}
/// @nodoc
class __$SystemCalendarCopyWithImpl<$Res>
    implements _$SystemCalendarCopyWith<$Res> {
  __$SystemCalendarCopyWithImpl(this._self, this._then);

  final _SystemCalendar _self;
  final $Res Function(_SystemCalendar) _then;

/// Create a copy of SystemCalendar
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? name = null,Object? color = null,Object? enabled = null,}) {
  return _then(_SystemCalendar(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,color: null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as String,enabled: null == enabled ? _self.enabled : enabled // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc
mixin _$SystemCalendarEvent {

 String get uid; int get start; int get end; String get title; String get location; String get calendarId; String get calendar; String get color; String get conferenceUrl; List<SystemCalendarAttendee> get attendees; bool get allDay; bool get free; bool get declined;
/// Create a copy of SystemCalendarEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SystemCalendarEventCopyWith<SystemCalendarEvent> get copyWith => _$SystemCalendarEventCopyWithImpl<SystemCalendarEvent>(this as SystemCalendarEvent, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SystemCalendarEvent&&(identical(other.uid, uid) || other.uid == uid)&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end)&&(identical(other.title, title) || other.title == title)&&(identical(other.location, location) || other.location == location)&&(identical(other.calendarId, calendarId) || other.calendarId == calendarId)&&(identical(other.calendar, calendar) || other.calendar == calendar)&&(identical(other.color, color) || other.color == color)&&(identical(other.conferenceUrl, conferenceUrl) || other.conferenceUrl == conferenceUrl)&&const DeepCollectionEquality().equals(other.attendees, attendees)&&(identical(other.allDay, allDay) || other.allDay == allDay)&&(identical(other.free, free) || other.free == free)&&(identical(other.declined, declined) || other.declined == declined));
}


@override
int get hashCode => Object.hash(runtimeType,uid,start,end,title,location,calendarId,calendar,color,conferenceUrl,const DeepCollectionEquality().hash(attendees),allDay,free,declined);

@override
String toString() {
  return 'SystemCalendarEvent(uid: $uid, start: $start, end: $end, title: $title, location: $location, calendarId: $calendarId, calendar: $calendar, color: $color, conferenceUrl: $conferenceUrl, attendees: $attendees, allDay: $allDay, free: $free, declined: $declined)';
}


}

/// @nodoc
abstract mixin class $SystemCalendarEventCopyWith<$Res>  {
  factory $SystemCalendarEventCopyWith(SystemCalendarEvent value, $Res Function(SystemCalendarEvent) _then) = _$SystemCalendarEventCopyWithImpl;
@useResult
$Res call({
 String uid, int start, int end, String title, String location, String calendarId, String calendar, String color, String conferenceUrl, List<SystemCalendarAttendee> attendees, bool allDay, bool free, bool declined
});




}
/// @nodoc
class _$SystemCalendarEventCopyWithImpl<$Res>
    implements $SystemCalendarEventCopyWith<$Res> {
  _$SystemCalendarEventCopyWithImpl(this._self, this._then);

  final SystemCalendarEvent _self;
  final $Res Function(SystemCalendarEvent) _then;

/// Create a copy of SystemCalendarEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? uid = null,Object? start = null,Object? end = null,Object? title = null,Object? location = null,Object? calendarId = null,Object? calendar = null,Object? color = null,Object? conferenceUrl = null,Object? attendees = null,Object? allDay = null,Object? free = null,Object? declined = null,}) {
  return _then(_self.copyWith(
uid: null == uid ? _self.uid : uid // ignore: cast_nullable_to_non_nullable
as String,start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,location: null == location ? _self.location : location // ignore: cast_nullable_to_non_nullable
as String,calendarId: null == calendarId ? _self.calendarId : calendarId // ignore: cast_nullable_to_non_nullable
as String,calendar: null == calendar ? _self.calendar : calendar // ignore: cast_nullable_to_non_nullable
as String,color: null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as String,conferenceUrl: null == conferenceUrl ? _self.conferenceUrl : conferenceUrl // ignore: cast_nullable_to_non_nullable
as String,attendees: null == attendees ? _self.attendees : attendees // ignore: cast_nullable_to_non_nullable
as List<SystemCalendarAttendee>,allDay: null == allDay ? _self.allDay : allDay // ignore: cast_nullable_to_non_nullable
as bool,free: null == free ? _self.free : free // ignore: cast_nullable_to_non_nullable
as bool,declined: null == declined ? _self.declined : declined // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [SystemCalendarEvent].
extension SystemCalendarEventPatterns on SystemCalendarEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SystemCalendarEvent value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SystemCalendarEvent() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SystemCalendarEvent value)  $default,){
final _that = this;
switch (_that) {
case _SystemCalendarEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SystemCalendarEvent value)?  $default,){
final _that = this;
switch (_that) {
case _SystemCalendarEvent() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String uid,  int start,  int end,  String title,  String location,  String calendarId,  String calendar,  String color,  String conferenceUrl,  List<SystemCalendarAttendee> attendees,  bool allDay,  bool free,  bool declined)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SystemCalendarEvent() when $default != null:
return $default(_that.uid,_that.start,_that.end,_that.title,_that.location,_that.calendarId,_that.calendar,_that.color,_that.conferenceUrl,_that.attendees,_that.allDay,_that.free,_that.declined);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String uid,  int start,  int end,  String title,  String location,  String calendarId,  String calendar,  String color,  String conferenceUrl,  List<SystemCalendarAttendee> attendees,  bool allDay,  bool free,  bool declined)  $default,) {final _that = this;
switch (_that) {
case _SystemCalendarEvent():
return $default(_that.uid,_that.start,_that.end,_that.title,_that.location,_that.calendarId,_that.calendar,_that.color,_that.conferenceUrl,_that.attendees,_that.allDay,_that.free,_that.declined);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String uid,  int start,  int end,  String title,  String location,  String calendarId,  String calendar,  String color,  String conferenceUrl,  List<SystemCalendarAttendee> attendees,  bool allDay,  bool free,  bool declined)?  $default,) {final _that = this;
switch (_that) {
case _SystemCalendarEvent() when $default != null:
return $default(_that.uid,_that.start,_that.end,_that.title,_that.location,_that.calendarId,_that.calendar,_that.color,_that.conferenceUrl,_that.attendees,_that.allDay,_that.free,_that.declined);case _:
  return null;

}
}

}

/// @nodoc


class _SystemCalendarEvent implements SystemCalendarEvent {
  const _SystemCalendarEvent({required this.uid, required this.start, required this.end, required this.title, required this.location, required this.calendarId, required this.calendar, required this.color, required this.conferenceUrl, required final  List<SystemCalendarAttendee> attendees, required this.allDay, required this.free, required this.declined}): _attendees = attendees;
  

@override final  String uid;
@override final  int start;
@override final  int end;
@override final  String title;
@override final  String location;
@override final  String calendarId;
@override final  String calendar;
@override final  String color;
@override final  String conferenceUrl;
 final  List<SystemCalendarAttendee> _attendees;
@override List<SystemCalendarAttendee> get attendees {
  if (_attendees is EqualUnmodifiableListView) return _attendees;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_attendees);
}

@override final  bool allDay;
@override final  bool free;
@override final  bool declined;

/// Create a copy of SystemCalendarEvent
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SystemCalendarEventCopyWith<_SystemCalendarEvent> get copyWith => __$SystemCalendarEventCopyWithImpl<_SystemCalendarEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SystemCalendarEvent&&(identical(other.uid, uid) || other.uid == uid)&&(identical(other.start, start) || other.start == start)&&(identical(other.end, end) || other.end == end)&&(identical(other.title, title) || other.title == title)&&(identical(other.location, location) || other.location == location)&&(identical(other.calendarId, calendarId) || other.calendarId == calendarId)&&(identical(other.calendar, calendar) || other.calendar == calendar)&&(identical(other.color, color) || other.color == color)&&(identical(other.conferenceUrl, conferenceUrl) || other.conferenceUrl == conferenceUrl)&&const DeepCollectionEquality().equals(other._attendees, _attendees)&&(identical(other.allDay, allDay) || other.allDay == allDay)&&(identical(other.free, free) || other.free == free)&&(identical(other.declined, declined) || other.declined == declined));
}


@override
int get hashCode => Object.hash(runtimeType,uid,start,end,title,location,calendarId,calendar,color,conferenceUrl,const DeepCollectionEquality().hash(_attendees),allDay,free,declined);

@override
String toString() {
  return 'SystemCalendarEvent(uid: $uid, start: $start, end: $end, title: $title, location: $location, calendarId: $calendarId, calendar: $calendar, color: $color, conferenceUrl: $conferenceUrl, attendees: $attendees, allDay: $allDay, free: $free, declined: $declined)';
}


}

/// @nodoc
abstract mixin class _$SystemCalendarEventCopyWith<$Res> implements $SystemCalendarEventCopyWith<$Res> {
  factory _$SystemCalendarEventCopyWith(_SystemCalendarEvent value, $Res Function(_SystemCalendarEvent) _then) = __$SystemCalendarEventCopyWithImpl;
@override @useResult
$Res call({
 String uid, int start, int end, String title, String location, String calendarId, String calendar, String color, String conferenceUrl, List<SystemCalendarAttendee> attendees, bool allDay, bool free, bool declined
});




}
/// @nodoc
class __$SystemCalendarEventCopyWithImpl<$Res>
    implements _$SystemCalendarEventCopyWith<$Res> {
  __$SystemCalendarEventCopyWithImpl(this._self, this._then);

  final _SystemCalendarEvent _self;
  final $Res Function(_SystemCalendarEvent) _then;

/// Create a copy of SystemCalendarEvent
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? uid = null,Object? start = null,Object? end = null,Object? title = null,Object? location = null,Object? calendarId = null,Object? calendar = null,Object? color = null,Object? conferenceUrl = null,Object? attendees = null,Object? allDay = null,Object? free = null,Object? declined = null,}) {
  return _then(_SystemCalendarEvent(
uid: null == uid ? _self.uid : uid // ignore: cast_nullable_to_non_nullable
as String,start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as int,end: null == end ? _self.end : end // ignore: cast_nullable_to_non_nullable
as int,title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,location: null == location ? _self.location : location // ignore: cast_nullable_to_non_nullable
as String,calendarId: null == calendarId ? _self.calendarId : calendarId // ignore: cast_nullable_to_non_nullable
as String,calendar: null == calendar ? _self.calendar : calendar // ignore: cast_nullable_to_non_nullable
as String,color: null == color ? _self.color : color // ignore: cast_nullable_to_non_nullable
as String,conferenceUrl: null == conferenceUrl ? _self.conferenceUrl : conferenceUrl // ignore: cast_nullable_to_non_nullable
as String,attendees: null == attendees ? _self._attendees : attendees // ignore: cast_nullable_to_non_nullable
as List<SystemCalendarAttendee>,allDay: null == allDay ? _self.allDay : allDay // ignore: cast_nullable_to_non_nullable
as bool,free: null == free ? _self.free : free // ignore: cast_nullable_to_non_nullable
as bool,declined: null == declined ? _self.declined : declined // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc
mixin _$SystemCalendarAttendee {

 String get name; String get email;
/// Create a copy of SystemCalendarAttendee
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SystemCalendarAttendeeCopyWith<SystemCalendarAttendee> get copyWith => _$SystemCalendarAttendeeCopyWithImpl<SystemCalendarAttendee>(this as SystemCalendarAttendee, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SystemCalendarAttendee&&(identical(other.name, name) || other.name == name)&&(identical(other.email, email) || other.email == email));
}


@override
int get hashCode => Object.hash(runtimeType,name,email);

@override
String toString() {
  return 'SystemCalendarAttendee(name: $name, email: $email)';
}


}

/// @nodoc
abstract mixin class $SystemCalendarAttendeeCopyWith<$Res>  {
  factory $SystemCalendarAttendeeCopyWith(SystemCalendarAttendee value, $Res Function(SystemCalendarAttendee) _then) = _$SystemCalendarAttendeeCopyWithImpl;
@useResult
$Res call({
 String name, String email
});




}
/// @nodoc
class _$SystemCalendarAttendeeCopyWithImpl<$Res>
    implements $SystemCalendarAttendeeCopyWith<$Res> {
  _$SystemCalendarAttendeeCopyWithImpl(this._self, this._then);

  final SystemCalendarAttendee _self;
  final $Res Function(SystemCalendarAttendee) _then;

/// Create a copy of SystemCalendarAttendee
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? name = null,Object? email = null,}) {
  return _then(_self.copyWith(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,email: null == email ? _self.email : email // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [SystemCalendarAttendee].
extension SystemCalendarAttendeePatterns on SystemCalendarAttendee {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SystemCalendarAttendee value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SystemCalendarAttendee() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SystemCalendarAttendee value)  $default,){
final _that = this;
switch (_that) {
case _SystemCalendarAttendee():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SystemCalendarAttendee value)?  $default,){
final _that = this;
switch (_that) {
case _SystemCalendarAttendee() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String name,  String email)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SystemCalendarAttendee() when $default != null:
return $default(_that.name,_that.email);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String name,  String email)  $default,) {final _that = this;
switch (_that) {
case _SystemCalendarAttendee():
return $default(_that.name,_that.email);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String name,  String email)?  $default,) {final _that = this;
switch (_that) {
case _SystemCalendarAttendee() when $default != null:
return $default(_that.name,_that.email);case _:
  return null;

}
}

}

/// @nodoc


class _SystemCalendarAttendee implements SystemCalendarAttendee {
  const _SystemCalendarAttendee({required this.name, required this.email});
  

@override final  String name;
@override final  String email;

/// Create a copy of SystemCalendarAttendee
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SystemCalendarAttendeeCopyWith<_SystemCalendarAttendee> get copyWith => __$SystemCalendarAttendeeCopyWithImpl<_SystemCalendarAttendee>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SystemCalendarAttendee&&(identical(other.name, name) || other.name == name)&&(identical(other.email, email) || other.email == email));
}


@override
int get hashCode => Object.hash(runtimeType,name,email);

@override
String toString() {
  return 'SystemCalendarAttendee(name: $name, email: $email)';
}


}

/// @nodoc
abstract mixin class _$SystemCalendarAttendeeCopyWith<$Res> implements $SystemCalendarAttendeeCopyWith<$Res> {
  factory _$SystemCalendarAttendeeCopyWith(_SystemCalendarAttendee value, $Res Function(_SystemCalendarAttendee) _then) = __$SystemCalendarAttendeeCopyWithImpl;
@override @useResult
$Res call({
 String name, String email
});




}
/// @nodoc
class __$SystemCalendarAttendeeCopyWithImpl<$Res>
    implements _$SystemCalendarAttendeeCopyWith<$Res> {
  __$SystemCalendarAttendeeCopyWithImpl(this._self, this._then);

  final _SystemCalendarAttendee _self;
  final $Res Function(_SystemCalendarAttendee) _then;

/// Create a copy of SystemCalendarAttendee
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? name = null,Object? email = null,}) {
  return _then(_SystemCalendarAttendee(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,email: null == email ? _self.email : email // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on

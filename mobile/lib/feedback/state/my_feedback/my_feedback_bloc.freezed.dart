// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'my_feedback_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$MyFeedbackEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MyFeedbackEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'MyFeedbackEvent()';
}


}

/// @nodoc
class $MyFeedbackEventCopyWith<$Res>  {
$MyFeedbackEventCopyWith(MyFeedbackEvent _, $Res Function(MyFeedbackEvent) __);
}


/// Adds pattern-matching-related methods to [MyFeedbackEvent].
extension MyFeedbackEventPatterns on MyFeedbackEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( MyFeedbackStartedEvent value)?  started,TResult Function( MyFeedbackRefreshRequestedEvent value)?  refreshRequested,TResult Function( MyFeedbackDeleteRequestedEvent value)?  deleteRequested,TResult Function( MyFeedbackSeenEvent value)?  seen,TResult Function( MyFeedbackResumedEvent value)?  resumed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case MyFeedbackStartedEvent() when started != null:
return started(_that);case MyFeedbackRefreshRequestedEvent() when refreshRequested != null:
return refreshRequested(_that);case MyFeedbackDeleteRequestedEvent() when deleteRequested != null:
return deleteRequested(_that);case MyFeedbackSeenEvent() when seen != null:
return seen(_that);case MyFeedbackResumedEvent() when resumed != null:
return resumed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( MyFeedbackStartedEvent value)  started,required TResult Function( MyFeedbackRefreshRequestedEvent value)  refreshRequested,required TResult Function( MyFeedbackDeleteRequestedEvent value)  deleteRequested,required TResult Function( MyFeedbackSeenEvent value)  seen,required TResult Function( MyFeedbackResumedEvent value)  resumed,}){
final _that = this;
switch (_that) {
case MyFeedbackStartedEvent():
return started(_that);case MyFeedbackRefreshRequestedEvent():
return refreshRequested(_that);case MyFeedbackDeleteRequestedEvent():
return deleteRequested(_that);case MyFeedbackSeenEvent():
return seen(_that);case MyFeedbackResumedEvent():
return resumed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( MyFeedbackStartedEvent value)?  started,TResult? Function( MyFeedbackRefreshRequestedEvent value)?  refreshRequested,TResult? Function( MyFeedbackDeleteRequestedEvent value)?  deleteRequested,TResult? Function( MyFeedbackSeenEvent value)?  seen,TResult? Function( MyFeedbackResumedEvent value)?  resumed,}){
final _that = this;
switch (_that) {
case MyFeedbackStartedEvent() when started != null:
return started(_that);case MyFeedbackRefreshRequestedEvent() when refreshRequested != null:
return refreshRequested(_that);case MyFeedbackDeleteRequestedEvent() when deleteRequested != null:
return deleteRequested(_that);case MyFeedbackSeenEvent() when seen != null:
return seen(_that);case MyFeedbackResumedEvent() when resumed != null:
return resumed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( bool launch)?  started,TResult Function()?  refreshRequested,TResult Function( String id)?  deleteRequested,TResult Function()?  seen,TResult Function()?  resumed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case MyFeedbackStartedEvent() when started != null:
return started(_that.launch);case MyFeedbackRefreshRequestedEvent() when refreshRequested != null:
return refreshRequested();case MyFeedbackDeleteRequestedEvent() when deleteRequested != null:
return deleteRequested(_that.id);case MyFeedbackSeenEvent() when seen != null:
return seen();case MyFeedbackResumedEvent() when resumed != null:
return resumed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( bool launch)  started,required TResult Function()  refreshRequested,required TResult Function( String id)  deleteRequested,required TResult Function()  seen,required TResult Function()  resumed,}) {final _that = this;
switch (_that) {
case MyFeedbackStartedEvent():
return started(_that.launch);case MyFeedbackRefreshRequestedEvent():
return refreshRequested();case MyFeedbackDeleteRequestedEvent():
return deleteRequested(_that.id);case MyFeedbackSeenEvent():
return seen();case MyFeedbackResumedEvent():
return resumed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( bool launch)?  started,TResult? Function()?  refreshRequested,TResult? Function( String id)?  deleteRequested,TResult? Function()?  seen,TResult? Function()?  resumed,}) {final _that = this;
switch (_that) {
case MyFeedbackStartedEvent() when started != null:
return started(_that.launch);case MyFeedbackRefreshRequestedEvent() when refreshRequested != null:
return refreshRequested();case MyFeedbackDeleteRequestedEvent() when deleteRequested != null:
return deleteRequested(_that.id);case MyFeedbackSeenEvent() when seen != null:
return seen();case MyFeedbackResumedEvent() when resumed != null:
return resumed();case _:
  return null;

}
}

}

/// @nodoc


class MyFeedbackStartedEvent implements MyFeedbackEvent {
  const MyFeedbackStartedEvent({this.launch = false});
  

@JsonKey() final  bool launch;

/// Create a copy of MyFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MyFeedbackStartedEventCopyWith<MyFeedbackStartedEvent> get copyWith => _$MyFeedbackStartedEventCopyWithImpl<MyFeedbackStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MyFeedbackStartedEvent&&(identical(other.launch, launch) || other.launch == launch));
}


@override
int get hashCode => Object.hash(runtimeType,launch);

@override
String toString() {
  return 'MyFeedbackEvent.started(launch: $launch)';
}


}

/// @nodoc
abstract mixin class $MyFeedbackStartedEventCopyWith<$Res> implements $MyFeedbackEventCopyWith<$Res> {
  factory $MyFeedbackStartedEventCopyWith(MyFeedbackStartedEvent value, $Res Function(MyFeedbackStartedEvent) _then) = _$MyFeedbackStartedEventCopyWithImpl;
@useResult
$Res call({
 bool launch
});




}
/// @nodoc
class _$MyFeedbackStartedEventCopyWithImpl<$Res>
    implements $MyFeedbackStartedEventCopyWith<$Res> {
  _$MyFeedbackStartedEventCopyWithImpl(this._self, this._then);

  final MyFeedbackStartedEvent _self;
  final $Res Function(MyFeedbackStartedEvent) _then;

/// Create a copy of MyFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? launch = null,}) {
  return _then(MyFeedbackStartedEvent(
launch: null == launch ? _self.launch : launch // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class MyFeedbackRefreshRequestedEvent implements MyFeedbackEvent {
  const MyFeedbackRefreshRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MyFeedbackRefreshRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'MyFeedbackEvent.refreshRequested()';
}


}




/// @nodoc


class MyFeedbackDeleteRequestedEvent implements MyFeedbackEvent {
  const MyFeedbackDeleteRequestedEvent(this.id);
  

 final  String id;

/// Create a copy of MyFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MyFeedbackDeleteRequestedEventCopyWith<MyFeedbackDeleteRequestedEvent> get copyWith => _$MyFeedbackDeleteRequestedEventCopyWithImpl<MyFeedbackDeleteRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MyFeedbackDeleteRequestedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'MyFeedbackEvent.deleteRequested(id: $id)';
}


}

/// @nodoc
abstract mixin class $MyFeedbackDeleteRequestedEventCopyWith<$Res> implements $MyFeedbackEventCopyWith<$Res> {
  factory $MyFeedbackDeleteRequestedEventCopyWith(MyFeedbackDeleteRequestedEvent value, $Res Function(MyFeedbackDeleteRequestedEvent) _then) = _$MyFeedbackDeleteRequestedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$MyFeedbackDeleteRequestedEventCopyWithImpl<$Res>
    implements $MyFeedbackDeleteRequestedEventCopyWith<$Res> {
  _$MyFeedbackDeleteRequestedEventCopyWithImpl(this._self, this._then);

  final MyFeedbackDeleteRequestedEvent _self;
  final $Res Function(MyFeedbackDeleteRequestedEvent) _then;

/// Create a copy of MyFeedbackEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(MyFeedbackDeleteRequestedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class MyFeedbackSeenEvent implements MyFeedbackEvent {
  const MyFeedbackSeenEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MyFeedbackSeenEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'MyFeedbackEvent.seen()';
}


}




/// @nodoc


class MyFeedbackResumedEvent implements MyFeedbackEvent {
  const MyFeedbackResumedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MyFeedbackResumedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'MyFeedbackEvent.resumed()';
}


}




/// @nodoc
mixin _$MyFeedbackState {

 LoadStatus get status;/// The reports, newest first.
 List<FeedbackEntry> get entries;/// The server is being asked.
 bool get refreshing;
/// Create a copy of MyFeedbackState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$MyFeedbackStateCopyWith<MyFeedbackState> get copyWith => _$MyFeedbackStateCopyWithImpl<MyFeedbackState>(this as MyFeedbackState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is MyFeedbackState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.entries, entries)&&(identical(other.refreshing, refreshing) || other.refreshing == refreshing));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(entries),refreshing);

@override
String toString() {
  return 'MyFeedbackState(status: $status, entries: $entries, refreshing: $refreshing)';
}


}

/// @nodoc
abstract mixin class $MyFeedbackStateCopyWith<$Res>  {
  factory $MyFeedbackStateCopyWith(MyFeedbackState value, $Res Function(MyFeedbackState) _then) = _$MyFeedbackStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<FeedbackEntry> entries, bool refreshing
});




}
/// @nodoc
class _$MyFeedbackStateCopyWithImpl<$Res>
    implements $MyFeedbackStateCopyWith<$Res> {
  _$MyFeedbackStateCopyWithImpl(this._self, this._then);

  final MyFeedbackState _self;
  final $Res Function(MyFeedbackState) _then;

/// Create a copy of MyFeedbackState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? entries = null,Object? refreshing = null,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,entries: null == entries ? _self.entries : entries // ignore: cast_nullable_to_non_nullable
as List<FeedbackEntry>,refreshing: null == refreshing ? _self.refreshing : refreshing // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

}


/// Adds pattern-matching-related methods to [MyFeedbackState].
extension MyFeedbackStatePatterns on MyFeedbackState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _MyFeedbackState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _MyFeedbackState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _MyFeedbackState value)  $default,){
final _that = this;
switch (_that) {
case _MyFeedbackState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _MyFeedbackState value)?  $default,){
final _that = this;
switch (_that) {
case _MyFeedbackState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<FeedbackEntry> entries,  bool refreshing)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _MyFeedbackState() when $default != null:
return $default(_that.status,_that.entries,_that.refreshing);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<FeedbackEntry> entries,  bool refreshing)  $default,) {final _that = this;
switch (_that) {
case _MyFeedbackState():
return $default(_that.status,_that.entries,_that.refreshing);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<FeedbackEntry> entries,  bool refreshing)?  $default,) {final _that = this;
switch (_that) {
case _MyFeedbackState() when $default != null:
return $default(_that.status,_that.entries,_that.refreshing);case _:
  return null;

}
}

}

/// @nodoc


class _MyFeedbackState extends MyFeedbackState {
  const _MyFeedbackState({this.status = LoadStatus.loading, final  List<FeedbackEntry> entries = const <FeedbackEntry>[], this.refreshing = false}): _entries = entries,super._();
  

@override@JsonKey() final  LoadStatus status;
/// The reports, newest first.
 final  List<FeedbackEntry> _entries;
/// The reports, newest first.
@override@JsonKey() List<FeedbackEntry> get entries {
  if (_entries is EqualUnmodifiableListView) return _entries;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_entries);
}

/// The server is being asked.
@override@JsonKey() final  bool refreshing;

/// Create a copy of MyFeedbackState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$MyFeedbackStateCopyWith<_MyFeedbackState> get copyWith => __$MyFeedbackStateCopyWithImpl<_MyFeedbackState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _MyFeedbackState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._entries, _entries)&&(identical(other.refreshing, refreshing) || other.refreshing == refreshing));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_entries),refreshing);

@override
String toString() {
  return 'MyFeedbackState(status: $status, entries: $entries, refreshing: $refreshing)';
}


}

/// @nodoc
abstract mixin class _$MyFeedbackStateCopyWith<$Res> implements $MyFeedbackStateCopyWith<$Res> {
  factory _$MyFeedbackStateCopyWith(_MyFeedbackState value, $Res Function(_MyFeedbackState) _then) = __$MyFeedbackStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<FeedbackEntry> entries, bool refreshing
});




}
/// @nodoc
class __$MyFeedbackStateCopyWithImpl<$Res>
    implements _$MyFeedbackStateCopyWith<$Res> {
  __$MyFeedbackStateCopyWithImpl(this._self, this._then);

  final _MyFeedbackState _self;
  final $Res Function(_MyFeedbackState) _then;

/// Create a copy of MyFeedbackState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? entries = null,Object? refreshing = null,}) {
  return _then(_MyFeedbackState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,entries: null == entries ? _self._entries : entries // ignore: cast_nullable_to_non_nullable
as List<FeedbackEntry>,refreshing: null == refreshing ? _self.refreshing : refreshing // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

// dart format on

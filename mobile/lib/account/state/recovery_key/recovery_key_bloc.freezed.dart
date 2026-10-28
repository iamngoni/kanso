// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'recovery_key_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$RecoveryKeyEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoveryKeyEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecoveryKeyEvent()';
}


}

/// @nodoc
class $RecoveryKeyEventCopyWith<$Res>  {
$RecoveryKeyEventCopyWith(RecoveryKeyEvent _, $Res Function(RecoveryKeyEvent) __);
}


/// Adds pattern-matching-related methods to [RecoveryKeyEvent].
extension RecoveryKeyEventPatterns on RecoveryKeyEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( RecoveryKeyStartedEvent value)?  started,TResult Function( RecoveryKeyConfirmedChangedEvent value)?  confirmedChanged,TResult Function( RecoveryKeyDoneEvent value)?  done,required TResult orElse(),}){
final _that = this;
switch (_that) {
case RecoveryKeyStartedEvent() when started != null:
return started(_that);case RecoveryKeyConfirmedChangedEvent() when confirmedChanged != null:
return confirmedChanged(_that);case RecoveryKeyDoneEvent() when done != null:
return done(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( RecoveryKeyStartedEvent value)  started,required TResult Function( RecoveryKeyConfirmedChangedEvent value)  confirmedChanged,required TResult Function( RecoveryKeyDoneEvent value)  done,}){
final _that = this;
switch (_that) {
case RecoveryKeyStartedEvent():
return started(_that);case RecoveryKeyConfirmedChangedEvent():
return confirmedChanged(_that);case RecoveryKeyDoneEvent():
return done(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( RecoveryKeyStartedEvent value)?  started,TResult? Function( RecoveryKeyConfirmedChangedEvent value)?  confirmedChanged,TResult? Function( RecoveryKeyDoneEvent value)?  done,}){
final _that = this;
switch (_that) {
case RecoveryKeyStartedEvent() when started != null:
return started(_that);case RecoveryKeyConfirmedChangedEvent() when confirmedChanged != null:
return confirmedChanged(_that);case RecoveryKeyDoneEvent() when done != null:
return done(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( RecoveryKeyText? key)?  started,TResult Function( bool confirmed)?  confirmedChanged,TResult Function()?  done,required TResult orElse(),}) {final _that = this;
switch (_that) {
case RecoveryKeyStartedEvent() when started != null:
return started(_that.key);case RecoveryKeyConfirmedChangedEvent() when confirmedChanged != null:
return confirmedChanged(_that.confirmed);case RecoveryKeyDoneEvent() when done != null:
return done();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( RecoveryKeyText? key)  started,required TResult Function( bool confirmed)  confirmedChanged,required TResult Function()  done,}) {final _that = this;
switch (_that) {
case RecoveryKeyStartedEvent():
return started(_that.key);case RecoveryKeyConfirmedChangedEvent():
return confirmedChanged(_that.confirmed);case RecoveryKeyDoneEvent():
return done();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( RecoveryKeyText? key)?  started,TResult? Function( bool confirmed)?  confirmedChanged,TResult? Function()?  done,}) {final _that = this;
switch (_that) {
case RecoveryKeyStartedEvent() when started != null:
return started(_that.key);case RecoveryKeyConfirmedChangedEvent() when confirmedChanged != null:
return confirmedChanged(_that.confirmed);case RecoveryKeyDoneEvent() when done != null:
return done();case _:
  return null;

}
}

}

/// @nodoc


class RecoveryKeyStartedEvent implements RecoveryKeyEvent {
  const RecoveryKeyStartedEvent({this.key});
  

 final  RecoveryKeyText? key;

/// Create a copy of RecoveryKeyEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecoveryKeyStartedEventCopyWith<RecoveryKeyStartedEvent> get copyWith => _$RecoveryKeyStartedEventCopyWithImpl<RecoveryKeyStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoveryKeyStartedEvent&&(identical(other.key, key) || other.key == key));
}


@override
int get hashCode => Object.hash(runtimeType,key);

@override
String toString() {
  return 'RecoveryKeyEvent.started(key: $key)';
}


}

/// @nodoc
abstract mixin class $RecoveryKeyStartedEventCopyWith<$Res> implements $RecoveryKeyEventCopyWith<$Res> {
  factory $RecoveryKeyStartedEventCopyWith(RecoveryKeyStartedEvent value, $Res Function(RecoveryKeyStartedEvent) _then) = _$RecoveryKeyStartedEventCopyWithImpl;
@useResult
$Res call({
 RecoveryKeyText? key
});




}
/// @nodoc
class _$RecoveryKeyStartedEventCopyWithImpl<$Res>
    implements $RecoveryKeyStartedEventCopyWith<$Res> {
  _$RecoveryKeyStartedEventCopyWithImpl(this._self, this._then);

  final RecoveryKeyStartedEvent _self;
  final $Res Function(RecoveryKeyStartedEvent) _then;

/// Create a copy of RecoveryKeyEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? key = freezed,}) {
  return _then(RecoveryKeyStartedEvent(
key: freezed == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as RecoveryKeyText?,
  ));
}


}

/// @nodoc


class RecoveryKeyConfirmedChangedEvent implements RecoveryKeyEvent {
  const RecoveryKeyConfirmedChangedEvent({required this.confirmed});
  

 final  bool confirmed;

/// Create a copy of RecoveryKeyEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecoveryKeyConfirmedChangedEventCopyWith<RecoveryKeyConfirmedChangedEvent> get copyWith => _$RecoveryKeyConfirmedChangedEventCopyWithImpl<RecoveryKeyConfirmedChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoveryKeyConfirmedChangedEvent&&(identical(other.confirmed, confirmed) || other.confirmed == confirmed));
}


@override
int get hashCode => Object.hash(runtimeType,confirmed);

@override
String toString() {
  return 'RecoveryKeyEvent.confirmedChanged(confirmed: $confirmed)';
}


}

/// @nodoc
abstract mixin class $RecoveryKeyConfirmedChangedEventCopyWith<$Res> implements $RecoveryKeyEventCopyWith<$Res> {
  factory $RecoveryKeyConfirmedChangedEventCopyWith(RecoveryKeyConfirmedChangedEvent value, $Res Function(RecoveryKeyConfirmedChangedEvent) _then) = _$RecoveryKeyConfirmedChangedEventCopyWithImpl;
@useResult
$Res call({
 bool confirmed
});




}
/// @nodoc
class _$RecoveryKeyConfirmedChangedEventCopyWithImpl<$Res>
    implements $RecoveryKeyConfirmedChangedEventCopyWith<$Res> {
  _$RecoveryKeyConfirmedChangedEventCopyWithImpl(this._self, this._then);

  final RecoveryKeyConfirmedChangedEvent _self;
  final $Res Function(RecoveryKeyConfirmedChangedEvent) _then;

/// Create a copy of RecoveryKeyEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? confirmed = null,}) {
  return _then(RecoveryKeyConfirmedChangedEvent(
confirmed: null == confirmed ? _self.confirmed : confirmed // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class RecoveryKeyDoneEvent implements RecoveryKeyEvent {
  const RecoveryKeyDoneEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoveryKeyDoneEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'RecoveryKeyEvent.done()';
}


}




/// @nodoc
mixin _$RecoveryKeyState {

 LoadStatus get status;/// The key to write down; null when the library holds none unsaved.
 RecoveryKeyText? get key;/// Whether the person said they saved it.
 bool get confirmed;/// Whether Done is being carried out.
 bool get saving;/// Why the key could not be read or Done failed.
 ApplicationError? get error;/// Whether Done went through: the screen closes.
 bool get done;
/// Create a copy of RecoveryKeyState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$RecoveryKeyStateCopyWith<RecoveryKeyState> get copyWith => _$RecoveryKeyStateCopyWithImpl<RecoveryKeyState>(this as RecoveryKeyState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is RecoveryKeyState&&(identical(other.status, status) || other.status == status)&&(identical(other.key, key) || other.key == key)&&(identical(other.confirmed, confirmed) || other.confirmed == confirmed)&&(identical(other.saving, saving) || other.saving == saving)&&(identical(other.error, error) || other.error == error)&&(identical(other.done, done) || other.done == done));
}


@override
int get hashCode => Object.hash(runtimeType,status,key,confirmed,saving,error,done);

@override
String toString() {
  return 'RecoveryKeyState(status: $status, key: $key, confirmed: $confirmed, saving: $saving, error: $error, done: $done)';
}


}

/// @nodoc
abstract mixin class $RecoveryKeyStateCopyWith<$Res>  {
  factory $RecoveryKeyStateCopyWith(RecoveryKeyState value, $Res Function(RecoveryKeyState) _then) = _$RecoveryKeyStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, RecoveryKeyText? key, bool confirmed, bool saving, ApplicationError? error, bool done
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$RecoveryKeyStateCopyWithImpl<$Res>
    implements $RecoveryKeyStateCopyWith<$Res> {
  _$RecoveryKeyStateCopyWithImpl(this._self, this._then);

  final RecoveryKeyState _self;
  final $Res Function(RecoveryKeyState) _then;

/// Create a copy of RecoveryKeyState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? key = freezed,Object? confirmed = null,Object? saving = null,Object? error = freezed,Object? done = null,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,key: freezed == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as RecoveryKeyText?,confirmed: null == confirmed ? _self.confirmed : confirmed // ignore: cast_nullable_to_non_nullable
as bool,saving: null == saving ? _self.saving : saving // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,done: null == done ? _self.done : done // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}
/// Create a copy of RecoveryKeyState
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


/// Adds pattern-matching-related methods to [RecoveryKeyState].
extension RecoveryKeyStatePatterns on RecoveryKeyState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _RecoveryKeyState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _RecoveryKeyState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _RecoveryKeyState value)  $default,){
final _that = this;
switch (_that) {
case _RecoveryKeyState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _RecoveryKeyState value)?  $default,){
final _that = this;
switch (_that) {
case _RecoveryKeyState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  RecoveryKeyText? key,  bool confirmed,  bool saving,  ApplicationError? error,  bool done)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _RecoveryKeyState() when $default != null:
return $default(_that.status,_that.key,_that.confirmed,_that.saving,_that.error,_that.done);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  RecoveryKeyText? key,  bool confirmed,  bool saving,  ApplicationError? error,  bool done)  $default,) {final _that = this;
switch (_that) {
case _RecoveryKeyState():
return $default(_that.status,_that.key,_that.confirmed,_that.saving,_that.error,_that.done);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  RecoveryKeyText? key,  bool confirmed,  bool saving,  ApplicationError? error,  bool done)?  $default,) {final _that = this;
switch (_that) {
case _RecoveryKeyState() when $default != null:
return $default(_that.status,_that.key,_that.confirmed,_that.saving,_that.error,_that.done);case _:
  return null;

}
}

}

/// @nodoc


class _RecoveryKeyState implements RecoveryKeyState {
  const _RecoveryKeyState({this.status = LoadStatus.loading, this.key, this.confirmed = false, this.saving = false, this.error, this.done = false});
  

@override@JsonKey() final  LoadStatus status;
/// The key to write down; null when the library holds none unsaved.
@override final  RecoveryKeyText? key;
/// Whether the person said they saved it.
@override@JsonKey() final  bool confirmed;
/// Whether Done is being carried out.
@override@JsonKey() final  bool saving;
/// Why the key could not be read or Done failed.
@override final  ApplicationError? error;
/// Whether Done went through: the screen closes.
@override@JsonKey() final  bool done;

/// Create a copy of RecoveryKeyState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$RecoveryKeyStateCopyWith<_RecoveryKeyState> get copyWith => __$RecoveryKeyStateCopyWithImpl<_RecoveryKeyState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _RecoveryKeyState&&(identical(other.status, status) || other.status == status)&&(identical(other.key, key) || other.key == key)&&(identical(other.confirmed, confirmed) || other.confirmed == confirmed)&&(identical(other.saving, saving) || other.saving == saving)&&(identical(other.error, error) || other.error == error)&&(identical(other.done, done) || other.done == done));
}


@override
int get hashCode => Object.hash(runtimeType,status,key,confirmed,saving,error,done);

@override
String toString() {
  return 'RecoveryKeyState(status: $status, key: $key, confirmed: $confirmed, saving: $saving, error: $error, done: $done)';
}


}

/// @nodoc
abstract mixin class _$RecoveryKeyStateCopyWith<$Res> implements $RecoveryKeyStateCopyWith<$Res> {
  factory _$RecoveryKeyStateCopyWith(_RecoveryKeyState value, $Res Function(_RecoveryKeyState) _then) = __$RecoveryKeyStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, RecoveryKeyText? key, bool confirmed, bool saving, ApplicationError? error, bool done
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$RecoveryKeyStateCopyWithImpl<$Res>
    implements _$RecoveryKeyStateCopyWith<$Res> {
  __$RecoveryKeyStateCopyWithImpl(this._self, this._then);

  final _RecoveryKeyState _self;
  final $Res Function(_RecoveryKeyState) _then;

/// Create a copy of RecoveryKeyState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? key = freezed,Object? confirmed = null,Object? saving = null,Object? error = freezed,Object? done = null,}) {
  return _then(_RecoveryKeyState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,key: freezed == key ? _self.key : key // ignore: cast_nullable_to_non_nullable
as RecoveryKeyText?,confirmed: null == confirmed ? _self.confirmed : confirmed // ignore: cast_nullable_to_non_nullable
as bool,saving: null == saving ? _self.saving : saving // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,done: null == done ? _self.done : done // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}

/// Create a copy of RecoveryKeyState
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

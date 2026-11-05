// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'lock_remove_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$LockRemoveEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockRemoveEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockRemoveEvent()';
}


}

/// @nodoc
class $LockRemoveEventCopyWith<$Res>  {
$LockRemoveEventCopyWith(LockRemoveEvent _, $Res Function(LockRemoveEvent) __);
}


/// Adds pattern-matching-related methods to [LockRemoveEvent].
extension LockRemoveEventPatterns on LockRemoveEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LockRemoveConfirmedEvent value)?  confirmed,TResult Function( LockRemoveCancelledEvent value)?  cancelled,TResult Function( LockRemoveProgressedEvent value)?  progressed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent() when confirmed != null:
return confirmed(_that);case LockRemoveCancelledEvent() when cancelled != null:
return cancelled(_that);case LockRemoveProgressedEvent() when progressed != null:
return progressed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LockRemoveConfirmedEvent value)  confirmed,required TResult Function( LockRemoveCancelledEvent value)  cancelled,required TResult Function( LockRemoveProgressedEvent value)  progressed,}){
final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent():
return confirmed(_that);case LockRemoveCancelledEvent():
return cancelled(_that);case LockRemoveProgressedEvent():
return progressed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LockRemoveConfirmedEvent value)?  confirmed,TResult? Function( LockRemoveCancelledEvent value)?  cancelled,TResult? Function( LockRemoveProgressedEvent value)?  progressed,}){
final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent() when confirmed != null:
return confirmed(_that);case LockRemoveCancelledEvent() when cancelled != null:
return cancelled(_that);case LockRemoveProgressedEvent() when progressed != null:
return progressed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( LockTarget target)?  confirmed,TResult Function()?  cancelled,TResult Function( SealProgress progress)?  progressed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent() when confirmed != null:
return confirmed(_that.target);case LockRemoveCancelledEvent() when cancelled != null:
return cancelled();case LockRemoveProgressedEvent() when progressed != null:
return progressed(_that.progress);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( LockTarget target)  confirmed,required TResult Function()  cancelled,required TResult Function( SealProgress progress)  progressed,}) {final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent():
return confirmed(_that.target);case LockRemoveCancelledEvent():
return cancelled();case LockRemoveProgressedEvent():
return progressed(_that.progress);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( LockTarget target)?  confirmed,TResult? Function()?  cancelled,TResult? Function( SealProgress progress)?  progressed,}) {final _that = this;
switch (_that) {
case LockRemoveConfirmedEvent() when confirmed != null:
return confirmed(_that.target);case LockRemoveCancelledEvent() when cancelled != null:
return cancelled();case LockRemoveProgressedEvent() when progressed != null:
return progressed(_that.progress);case _:
  return null;

}
}

}

/// @nodoc


class LockRemoveConfirmedEvent implements LockRemoveEvent {
  const LockRemoveConfirmedEvent(this.target);
  

 final  LockTarget target;

/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockRemoveConfirmedEventCopyWith<LockRemoveConfirmedEvent> get copyWith => _$LockRemoveConfirmedEventCopyWithImpl<LockRemoveConfirmedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockRemoveConfirmedEvent&&(identical(other.target, target) || other.target == target));
}


@override
int get hashCode => Object.hash(runtimeType,target);

@override
String toString() {
  return 'LockRemoveEvent.confirmed(target: $target)';
}


}

/// @nodoc
abstract mixin class $LockRemoveConfirmedEventCopyWith<$Res> implements $LockRemoveEventCopyWith<$Res> {
  factory $LockRemoveConfirmedEventCopyWith(LockRemoveConfirmedEvent value, $Res Function(LockRemoveConfirmedEvent) _then) = _$LockRemoveConfirmedEventCopyWithImpl;
@useResult
$Res call({
 LockTarget target
});


$LockTargetCopyWith<$Res> get target;

}
/// @nodoc
class _$LockRemoveConfirmedEventCopyWithImpl<$Res>
    implements $LockRemoveConfirmedEventCopyWith<$Res> {
  _$LockRemoveConfirmedEventCopyWithImpl(this._self, this._then);

  final LockRemoveConfirmedEvent _self;
  final $Res Function(LockRemoveConfirmedEvent) _then;

/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? target = null,}) {
  return _then(LockRemoveConfirmedEvent(
null == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as LockTarget,
  ));
}

/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$LockTargetCopyWith<$Res> get target {
  
  return $LockTargetCopyWith<$Res>(_self.target, (value) {
    return _then(_self.copyWith(target: value));
  });
}
}

/// @nodoc


class LockRemoveCancelledEvent implements LockRemoveEvent {
  const LockRemoveCancelledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockRemoveCancelledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockRemoveEvent.cancelled()';
}


}




/// @nodoc


class LockRemoveProgressedEvent implements LockRemoveEvent {
  const LockRemoveProgressedEvent(this.progress);
  

 final  SealProgress progress;

/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockRemoveProgressedEventCopyWith<LockRemoveProgressedEvent> get copyWith => _$LockRemoveProgressedEventCopyWithImpl<LockRemoveProgressedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockRemoveProgressedEvent&&(identical(other.progress, progress) || other.progress == progress));
}


@override
int get hashCode => Object.hash(runtimeType,progress);

@override
String toString() {
  return 'LockRemoveEvent.progressed(progress: $progress)';
}


}

/// @nodoc
abstract mixin class $LockRemoveProgressedEventCopyWith<$Res> implements $LockRemoveEventCopyWith<$Res> {
  factory $LockRemoveProgressedEventCopyWith(LockRemoveProgressedEvent value, $Res Function(LockRemoveProgressedEvent) _then) = _$LockRemoveProgressedEventCopyWithImpl;
@useResult
$Res call({
 SealProgress progress
});




}
/// @nodoc
class _$LockRemoveProgressedEventCopyWithImpl<$Res>
    implements $LockRemoveProgressedEventCopyWith<$Res> {
  _$LockRemoveProgressedEventCopyWithImpl(this._self, this._then);

  final LockRemoveProgressedEvent _self;
  final $Res Function(LockRemoveProgressedEvent) _then;

/// Create a copy of LockRemoveEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? progress = null,}) {
  return _then(LockRemoveProgressedEvent(
null == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as SealProgress,
  ));
}


}

/// @nodoc
mixin _$LockRemoveState {

 RemoveStatus get status;/// Why it failed, with [RemoveStatus.failed]. An invalid request carries
/// the library's own sentence, which the sheet shows as it is.
 ApplicationError? get error;/// How far unsealing the files has come, while removing.
 SealProgress? get progress;/// Whether Cancel was pressed: it stops after the file being opened.
 bool get cancelling;/// The files copied for the note because another locked note still shows
/// the sealed ones, once [status] is done.
 LockCopyNotice? get copyNotice;
/// Create a copy of LockRemoveState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockRemoveStateCopyWith<LockRemoveState> get copyWith => _$LockRemoveStateCopyWithImpl<LockRemoveState>(this as LockRemoveState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockRemoveState&&(identical(other.status, status) || other.status == status)&&(identical(other.error, error) || other.error == error)&&(identical(other.progress, progress) || other.progress == progress)&&(identical(other.cancelling, cancelling) || other.cancelling == cancelling)&&(identical(other.copyNotice, copyNotice) || other.copyNotice == copyNotice));
}


@override
int get hashCode => Object.hash(runtimeType,status,error,progress,cancelling,copyNotice);

@override
String toString() {
  return 'LockRemoveState(status: $status, error: $error, progress: $progress, cancelling: $cancelling, copyNotice: $copyNotice)';
}


}

/// @nodoc
abstract mixin class $LockRemoveStateCopyWith<$Res>  {
  factory $LockRemoveStateCopyWith(LockRemoveState value, $Res Function(LockRemoveState) _then) = _$LockRemoveStateCopyWithImpl;
@useResult
$Res call({
 RemoveStatus status, ApplicationError? error, SealProgress? progress, bool cancelling, LockCopyNotice? copyNotice
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$LockRemoveStateCopyWithImpl<$Res>
    implements $LockRemoveStateCopyWith<$Res> {
  _$LockRemoveStateCopyWithImpl(this._self, this._then);

  final LockRemoveState _self;
  final $Res Function(LockRemoveState) _then;

/// Create a copy of LockRemoveState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? error = freezed,Object? progress = freezed,Object? cancelling = null,Object? copyNotice = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as RemoveStatus,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,progress: freezed == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as SealProgress?,cancelling: null == cancelling ? _self.cancelling : cancelling // ignore: cast_nullable_to_non_nullable
as bool,copyNotice: freezed == copyNotice ? _self.copyNotice : copyNotice // ignore: cast_nullable_to_non_nullable
as LockCopyNotice?,
  ));
}
/// Create a copy of LockRemoveState
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


/// Adds pattern-matching-related methods to [LockRemoveState].
extension LockRemoveStatePatterns on LockRemoveState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _LockRemoveState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _LockRemoveState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _LockRemoveState value)  $default,){
final _that = this;
switch (_that) {
case _LockRemoveState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _LockRemoveState value)?  $default,){
final _that = this;
switch (_that) {
case _LockRemoveState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( RemoveStatus status,  ApplicationError? error,  SealProgress? progress,  bool cancelling,  LockCopyNotice? copyNotice)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _LockRemoveState() when $default != null:
return $default(_that.status,_that.error,_that.progress,_that.cancelling,_that.copyNotice);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( RemoveStatus status,  ApplicationError? error,  SealProgress? progress,  bool cancelling,  LockCopyNotice? copyNotice)  $default,) {final _that = this;
switch (_that) {
case _LockRemoveState():
return $default(_that.status,_that.error,_that.progress,_that.cancelling,_that.copyNotice);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( RemoveStatus status,  ApplicationError? error,  SealProgress? progress,  bool cancelling,  LockCopyNotice? copyNotice)?  $default,) {final _that = this;
switch (_that) {
case _LockRemoveState() when $default != null:
return $default(_that.status,_that.error,_that.progress,_that.cancelling,_that.copyNotice);case _:
  return null;

}
}

}

/// @nodoc


class _LockRemoveState implements LockRemoveState {
  const _LockRemoveState({this.status = RemoveStatus.idle, this.error, this.progress, this.cancelling = false, this.copyNotice});
  

@override@JsonKey() final  RemoveStatus status;
/// Why it failed, with [RemoveStatus.failed]. An invalid request carries
/// the library's own sentence, which the sheet shows as it is.
@override final  ApplicationError? error;
/// How far unsealing the files has come, while removing.
@override final  SealProgress? progress;
/// Whether Cancel was pressed: it stops after the file being opened.
@override@JsonKey() final  bool cancelling;
/// The files copied for the note because another locked note still shows
/// the sealed ones, once [status] is done.
@override final  LockCopyNotice? copyNotice;

/// Create a copy of LockRemoveState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$LockRemoveStateCopyWith<_LockRemoveState> get copyWith => __$LockRemoveStateCopyWithImpl<_LockRemoveState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _LockRemoveState&&(identical(other.status, status) || other.status == status)&&(identical(other.error, error) || other.error == error)&&(identical(other.progress, progress) || other.progress == progress)&&(identical(other.cancelling, cancelling) || other.cancelling == cancelling)&&(identical(other.copyNotice, copyNotice) || other.copyNotice == copyNotice));
}


@override
int get hashCode => Object.hash(runtimeType,status,error,progress,cancelling,copyNotice);

@override
String toString() {
  return 'LockRemoveState(status: $status, error: $error, progress: $progress, cancelling: $cancelling, copyNotice: $copyNotice)';
}


}

/// @nodoc
abstract mixin class _$LockRemoveStateCopyWith<$Res> implements $LockRemoveStateCopyWith<$Res> {
  factory _$LockRemoveStateCopyWith(_LockRemoveState value, $Res Function(_LockRemoveState) _then) = __$LockRemoveStateCopyWithImpl;
@override @useResult
$Res call({
 RemoveStatus status, ApplicationError? error, SealProgress? progress, bool cancelling, LockCopyNotice? copyNotice
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$LockRemoveStateCopyWithImpl<$Res>
    implements _$LockRemoveStateCopyWith<$Res> {
  __$LockRemoveStateCopyWithImpl(this._self, this._then);

  final _LockRemoveState _self;
  final $Res Function(_LockRemoveState) _then;

/// Create a copy of LockRemoveState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? error = freezed,Object? progress = freezed,Object? cancelling = null,Object? copyNotice = freezed,}) {
  return _then(_LockRemoveState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as RemoveStatus,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,progress: freezed == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as SealProgress?,cancelling: null == cancelling ? _self.cancelling : cancelling // ignore: cast_nullable_to_non_nullable
as bool,copyNotice: freezed == copyNotice ? _self.copyNotice : copyNotice // ignore: cast_nullable_to_non_nullable
as LockCopyNotice?,
  ));
}

/// Create a copy of LockRemoveState
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

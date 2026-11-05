// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'interrupted_lock_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$InterruptedLockEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InterruptedLockEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'InterruptedLockEvent()';
}


}

/// @nodoc
class $InterruptedLockEventCopyWith<$Res>  {
$InterruptedLockEventCopyWith(InterruptedLockEvent _, $Res Function(InterruptedLockEvent) __);
}


/// Adds pattern-matching-related methods to [InterruptedLockEvent].
extension InterruptedLockEventPatterns on InterruptedLockEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( InterruptedLockCheckedEvent value)?  checked,TResult Function( InterruptedLockFinishedEvent value)?  finished,TResult Function( InterruptedLockUndoneEvent value)?  undone,TResult Function( InterruptedLockPasswordDeclinedEvent value)?  passwordDeclined,TResult Function( InterruptedLockProgressedEvent value)?  progressed,TResult Function( InterruptedLockNoticeShownEvent value)?  noticeShown,required TResult orElse(),}){
final _that = this;
switch (_that) {
case InterruptedLockCheckedEvent() when checked != null:
return checked(_that);case InterruptedLockFinishedEvent() when finished != null:
return finished(_that);case InterruptedLockUndoneEvent() when undone != null:
return undone(_that);case InterruptedLockPasswordDeclinedEvent() when passwordDeclined != null:
return passwordDeclined(_that);case InterruptedLockProgressedEvent() when progressed != null:
return progressed(_that);case InterruptedLockNoticeShownEvent() when noticeShown != null:
return noticeShown(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( InterruptedLockCheckedEvent value)  checked,required TResult Function( InterruptedLockFinishedEvent value)  finished,required TResult Function( InterruptedLockUndoneEvent value)  undone,required TResult Function( InterruptedLockPasswordDeclinedEvent value)  passwordDeclined,required TResult Function( InterruptedLockProgressedEvent value)  progressed,required TResult Function( InterruptedLockNoticeShownEvent value)  noticeShown,}){
final _that = this;
switch (_that) {
case InterruptedLockCheckedEvent():
return checked(_that);case InterruptedLockFinishedEvent():
return finished(_that);case InterruptedLockUndoneEvent():
return undone(_that);case InterruptedLockPasswordDeclinedEvent():
return passwordDeclined(_that);case InterruptedLockProgressedEvent():
return progressed(_that);case InterruptedLockNoticeShownEvent():
return noticeShown(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( InterruptedLockCheckedEvent value)?  checked,TResult? Function( InterruptedLockFinishedEvent value)?  finished,TResult? Function( InterruptedLockUndoneEvent value)?  undone,TResult? Function( InterruptedLockPasswordDeclinedEvent value)?  passwordDeclined,TResult? Function( InterruptedLockProgressedEvent value)?  progressed,TResult? Function( InterruptedLockNoticeShownEvent value)?  noticeShown,}){
final _that = this;
switch (_that) {
case InterruptedLockCheckedEvent() when checked != null:
return checked(_that);case InterruptedLockFinishedEvent() when finished != null:
return finished(_that);case InterruptedLockUndoneEvent() when undone != null:
return undone(_that);case InterruptedLockPasswordDeclinedEvent() when passwordDeclined != null:
return passwordDeclined(_that);case InterruptedLockProgressedEvent() when progressed != null:
return progressed(_that);case InterruptedLockNoticeShownEvent() when noticeShown != null:
return noticeShown(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  checked,TResult Function( String? password)?  finished,TResult Function( String? password)?  undone,TResult Function()?  passwordDeclined,TResult Function( SealProgress progress)?  progressed,TResult Function()?  noticeShown,required TResult orElse(),}) {final _that = this;
switch (_that) {
case InterruptedLockCheckedEvent() when checked != null:
return checked(_that.noteId);case InterruptedLockFinishedEvent() when finished != null:
return finished(_that.password);case InterruptedLockUndoneEvent() when undone != null:
return undone(_that.password);case InterruptedLockPasswordDeclinedEvent() when passwordDeclined != null:
return passwordDeclined();case InterruptedLockProgressedEvent() when progressed != null:
return progressed(_that.progress);case InterruptedLockNoticeShownEvent() when noticeShown != null:
return noticeShown();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  checked,required TResult Function( String? password)  finished,required TResult Function( String? password)  undone,required TResult Function()  passwordDeclined,required TResult Function( SealProgress progress)  progressed,required TResult Function()  noticeShown,}) {final _that = this;
switch (_that) {
case InterruptedLockCheckedEvent():
return checked(_that.noteId);case InterruptedLockFinishedEvent():
return finished(_that.password);case InterruptedLockUndoneEvent():
return undone(_that.password);case InterruptedLockPasswordDeclinedEvent():
return passwordDeclined();case InterruptedLockProgressedEvent():
return progressed(_that.progress);case InterruptedLockNoticeShownEvent():
return noticeShown();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  checked,TResult? Function( String? password)?  finished,TResult? Function( String? password)?  undone,TResult? Function()?  passwordDeclined,TResult? Function( SealProgress progress)?  progressed,TResult? Function()?  noticeShown,}) {final _that = this;
switch (_that) {
case InterruptedLockCheckedEvent() when checked != null:
return checked(_that.noteId);case InterruptedLockFinishedEvent() when finished != null:
return finished(_that.password);case InterruptedLockUndoneEvent() when undone != null:
return undone(_that.password);case InterruptedLockPasswordDeclinedEvent() when passwordDeclined != null:
return passwordDeclined();case InterruptedLockProgressedEvent() when progressed != null:
return progressed(_that.progress);case InterruptedLockNoticeShownEvent() when noticeShown != null:
return noticeShown();case _:
  return null;

}
}

}

/// @nodoc


class InterruptedLockCheckedEvent implements InterruptedLockEvent {
  const InterruptedLockCheckedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of InterruptedLockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InterruptedLockCheckedEventCopyWith<InterruptedLockCheckedEvent> get copyWith => _$InterruptedLockCheckedEventCopyWithImpl<InterruptedLockCheckedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InterruptedLockCheckedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'InterruptedLockEvent.checked(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $InterruptedLockCheckedEventCopyWith<$Res> implements $InterruptedLockEventCopyWith<$Res> {
  factory $InterruptedLockCheckedEventCopyWith(InterruptedLockCheckedEvent value, $Res Function(InterruptedLockCheckedEvent) _then) = _$InterruptedLockCheckedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$InterruptedLockCheckedEventCopyWithImpl<$Res>
    implements $InterruptedLockCheckedEventCopyWith<$Res> {
  _$InterruptedLockCheckedEventCopyWithImpl(this._self, this._then);

  final InterruptedLockCheckedEvent _self;
  final $Res Function(InterruptedLockCheckedEvent) _then;

/// Create a copy of InterruptedLockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(InterruptedLockCheckedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class InterruptedLockFinishedEvent implements InterruptedLockEvent {
  const InterruptedLockFinishedEvent({this.password});
  

 final  String? password;

/// Create a copy of InterruptedLockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InterruptedLockFinishedEventCopyWith<InterruptedLockFinishedEvent> get copyWith => _$InterruptedLockFinishedEventCopyWithImpl<InterruptedLockFinishedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InterruptedLockFinishedEvent&&(identical(other.password, password) || other.password == password));
}


@override
int get hashCode => Object.hash(runtimeType,password);

@override
String toString() {
  return 'InterruptedLockEvent.finished(password: $password)';
}


}

/// @nodoc
abstract mixin class $InterruptedLockFinishedEventCopyWith<$Res> implements $InterruptedLockEventCopyWith<$Res> {
  factory $InterruptedLockFinishedEventCopyWith(InterruptedLockFinishedEvent value, $Res Function(InterruptedLockFinishedEvent) _then) = _$InterruptedLockFinishedEventCopyWithImpl;
@useResult
$Res call({
 String? password
});




}
/// @nodoc
class _$InterruptedLockFinishedEventCopyWithImpl<$Res>
    implements $InterruptedLockFinishedEventCopyWith<$Res> {
  _$InterruptedLockFinishedEventCopyWithImpl(this._self, this._then);

  final InterruptedLockFinishedEvent _self;
  final $Res Function(InterruptedLockFinishedEvent) _then;

/// Create a copy of InterruptedLockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? password = freezed,}) {
  return _then(InterruptedLockFinishedEvent(
password: freezed == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class InterruptedLockUndoneEvent implements InterruptedLockEvent {
  const InterruptedLockUndoneEvent({this.password});
  

 final  String? password;

/// Create a copy of InterruptedLockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InterruptedLockUndoneEventCopyWith<InterruptedLockUndoneEvent> get copyWith => _$InterruptedLockUndoneEventCopyWithImpl<InterruptedLockUndoneEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InterruptedLockUndoneEvent&&(identical(other.password, password) || other.password == password));
}


@override
int get hashCode => Object.hash(runtimeType,password);

@override
String toString() {
  return 'InterruptedLockEvent.undone(password: $password)';
}


}

/// @nodoc
abstract mixin class $InterruptedLockUndoneEventCopyWith<$Res> implements $InterruptedLockEventCopyWith<$Res> {
  factory $InterruptedLockUndoneEventCopyWith(InterruptedLockUndoneEvent value, $Res Function(InterruptedLockUndoneEvent) _then) = _$InterruptedLockUndoneEventCopyWithImpl;
@useResult
$Res call({
 String? password
});




}
/// @nodoc
class _$InterruptedLockUndoneEventCopyWithImpl<$Res>
    implements $InterruptedLockUndoneEventCopyWith<$Res> {
  _$InterruptedLockUndoneEventCopyWithImpl(this._self, this._then);

  final InterruptedLockUndoneEvent _self;
  final $Res Function(InterruptedLockUndoneEvent) _then;

/// Create a copy of InterruptedLockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? password = freezed,}) {
  return _then(InterruptedLockUndoneEvent(
password: freezed == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class InterruptedLockPasswordDeclinedEvent implements InterruptedLockEvent {
  const InterruptedLockPasswordDeclinedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InterruptedLockPasswordDeclinedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'InterruptedLockEvent.passwordDeclined()';
}


}




/// @nodoc


class InterruptedLockProgressedEvent implements InterruptedLockEvent {
  const InterruptedLockProgressedEvent(this.progress);
  

 final  SealProgress progress;

/// Create a copy of InterruptedLockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InterruptedLockProgressedEventCopyWith<InterruptedLockProgressedEvent> get copyWith => _$InterruptedLockProgressedEventCopyWithImpl<InterruptedLockProgressedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InterruptedLockProgressedEvent&&(identical(other.progress, progress) || other.progress == progress));
}


@override
int get hashCode => Object.hash(runtimeType,progress);

@override
String toString() {
  return 'InterruptedLockEvent.progressed(progress: $progress)';
}


}

/// @nodoc
abstract mixin class $InterruptedLockProgressedEventCopyWith<$Res> implements $InterruptedLockEventCopyWith<$Res> {
  factory $InterruptedLockProgressedEventCopyWith(InterruptedLockProgressedEvent value, $Res Function(InterruptedLockProgressedEvent) _then) = _$InterruptedLockProgressedEventCopyWithImpl;
@useResult
$Res call({
 SealProgress progress
});




}
/// @nodoc
class _$InterruptedLockProgressedEventCopyWithImpl<$Res>
    implements $InterruptedLockProgressedEventCopyWith<$Res> {
  _$InterruptedLockProgressedEventCopyWithImpl(this._self, this._then);

  final InterruptedLockProgressedEvent _self;
  final $Res Function(InterruptedLockProgressedEvent) _then;

/// Create a copy of InterruptedLockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? progress = null,}) {
  return _then(InterruptedLockProgressedEvent(
null == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as SealProgress,
  ));
}


}

/// @nodoc


class InterruptedLockNoticeShownEvent implements InterruptedLockEvent {
  const InterruptedLockNoticeShownEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InterruptedLockNoticeShownEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'InterruptedLockEvent.noticeShown()';
}


}




/// @nodoc
mixin _$InterruptedLockState {

 String get noteId; InterruptedStage get stage;/// How many pictures and files the stopped lock covers, and how many are
/// sealed.
 int get files; int get sealed;/// What the person chose, while it is asked for the password.
 InterruptedAction? get action;/// How far finishing has come.
 SealProgress? get progress;/// Whether to say, once, that notes locked before this update seal their
/// pictures and files.
 bool get filesNotice; ApplicationError? get error;
/// Create a copy of InterruptedLockState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InterruptedLockStateCopyWith<InterruptedLockState> get copyWith => _$InterruptedLockStateCopyWithImpl<InterruptedLockState>(this as InterruptedLockState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InterruptedLockState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.stage, stage) || other.stage == stage)&&(identical(other.files, files) || other.files == files)&&(identical(other.sealed, sealed) || other.sealed == sealed)&&(identical(other.action, action) || other.action == action)&&(identical(other.progress, progress) || other.progress == progress)&&(identical(other.filesNotice, filesNotice) || other.filesNotice == filesNotice)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,stage,files,sealed,action,progress,filesNotice,error);

@override
String toString() {
  return 'InterruptedLockState(noteId: $noteId, stage: $stage, files: $files, sealed: $sealed, action: $action, progress: $progress, filesNotice: $filesNotice, error: $error)';
}


}

/// @nodoc
abstract mixin class $InterruptedLockStateCopyWith<$Res>  {
  factory $InterruptedLockStateCopyWith(InterruptedLockState value, $Res Function(InterruptedLockState) _then) = _$InterruptedLockStateCopyWithImpl;
@useResult
$Res call({
 String noteId, InterruptedStage stage, int files, int sealed, InterruptedAction? action, SealProgress? progress, bool filesNotice, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$InterruptedLockStateCopyWithImpl<$Res>
    implements $InterruptedLockStateCopyWith<$Res> {
  _$InterruptedLockStateCopyWithImpl(this._self, this._then);

  final InterruptedLockState _self;
  final $Res Function(InterruptedLockState) _then;

/// Create a copy of InterruptedLockState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? stage = null,Object? files = null,Object? sealed = null,Object? action = freezed,Object? progress = freezed,Object? filesNotice = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,stage: null == stage ? _self.stage : stage // ignore: cast_nullable_to_non_nullable
as InterruptedStage,files: null == files ? _self.files : files // ignore: cast_nullable_to_non_nullable
as int,sealed: null == sealed ? _self.sealed : sealed // ignore: cast_nullable_to_non_nullable
as int,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as InterruptedAction?,progress: freezed == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as SealProgress?,filesNotice: null == filesNotice ? _self.filesNotice : filesNotice // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of InterruptedLockState
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


/// Adds pattern-matching-related methods to [InterruptedLockState].
extension InterruptedLockStatePatterns on InterruptedLockState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _InterruptedLockState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _InterruptedLockState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _InterruptedLockState value)  $default,){
final _that = this;
switch (_that) {
case _InterruptedLockState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _InterruptedLockState value)?  $default,){
final _that = this;
switch (_that) {
case _InterruptedLockState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  InterruptedStage stage,  int files,  int sealed,  InterruptedAction? action,  SealProgress? progress,  bool filesNotice,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _InterruptedLockState() when $default != null:
return $default(_that.noteId,_that.stage,_that.files,_that.sealed,_that.action,_that.progress,_that.filesNotice,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  InterruptedStage stage,  int files,  int sealed,  InterruptedAction? action,  SealProgress? progress,  bool filesNotice,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _InterruptedLockState():
return $default(_that.noteId,_that.stage,_that.files,_that.sealed,_that.action,_that.progress,_that.filesNotice,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  InterruptedStage stage,  int files,  int sealed,  InterruptedAction? action,  SealProgress? progress,  bool filesNotice,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _InterruptedLockState() when $default != null:
return $default(_that.noteId,_that.stage,_that.files,_that.sealed,_that.action,_that.progress,_that.filesNotice,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _InterruptedLockState implements InterruptedLockState {
  const _InterruptedLockState({this.noteId = '', this.stage = InterruptedStage.checking, this.files = 0, this.sealed = 0, this.action, this.progress, this.filesNotice = false, this.error});
  

@override@JsonKey() final  String noteId;
@override@JsonKey() final  InterruptedStage stage;
/// How many pictures and files the stopped lock covers, and how many are
/// sealed.
@override@JsonKey() final  int files;
@override@JsonKey() final  int sealed;
/// What the person chose, while it is asked for the password.
@override final  InterruptedAction? action;
/// How far finishing has come.
@override final  SealProgress? progress;
/// Whether to say, once, that notes locked before this update seal their
/// pictures and files.
@override@JsonKey() final  bool filesNotice;
@override final  ApplicationError? error;

/// Create a copy of InterruptedLockState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$InterruptedLockStateCopyWith<_InterruptedLockState> get copyWith => __$InterruptedLockStateCopyWithImpl<_InterruptedLockState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _InterruptedLockState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.stage, stage) || other.stage == stage)&&(identical(other.files, files) || other.files == files)&&(identical(other.sealed, sealed) || other.sealed == sealed)&&(identical(other.action, action) || other.action == action)&&(identical(other.progress, progress) || other.progress == progress)&&(identical(other.filesNotice, filesNotice) || other.filesNotice == filesNotice)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,stage,files,sealed,action,progress,filesNotice,error);

@override
String toString() {
  return 'InterruptedLockState(noteId: $noteId, stage: $stage, files: $files, sealed: $sealed, action: $action, progress: $progress, filesNotice: $filesNotice, error: $error)';
}


}

/// @nodoc
abstract mixin class _$InterruptedLockStateCopyWith<$Res> implements $InterruptedLockStateCopyWith<$Res> {
  factory _$InterruptedLockStateCopyWith(_InterruptedLockState value, $Res Function(_InterruptedLockState) _then) = __$InterruptedLockStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, InterruptedStage stage, int files, int sealed, InterruptedAction? action, SealProgress? progress, bool filesNotice, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$InterruptedLockStateCopyWithImpl<$Res>
    implements _$InterruptedLockStateCopyWith<$Res> {
  __$InterruptedLockStateCopyWithImpl(this._self, this._then);

  final _InterruptedLockState _self;
  final $Res Function(_InterruptedLockState) _then;

/// Create a copy of InterruptedLockState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? stage = null,Object? files = null,Object? sealed = null,Object? action = freezed,Object? progress = freezed,Object? filesNotice = null,Object? error = freezed,}) {
  return _then(_InterruptedLockState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,stage: null == stage ? _self.stage : stage // ignore: cast_nullable_to_non_nullable
as InterruptedStage,files: null == files ? _self.files : files // ignore: cast_nullable_to_non_nullable
as int,sealed: null == sealed ? _self.sealed : sealed // ignore: cast_nullable_to_non_nullable
as int,action: freezed == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as InterruptedAction?,progress: freezed == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as SealProgress?,filesNotice: null == filesNotice ? _self.filesNotice : filesNotice // ignore: cast_nullable_to_non_nullable
as bool,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of InterruptedLockState
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

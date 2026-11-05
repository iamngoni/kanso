// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'files_migration_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$FilesMigrationEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FilesMigrationEvent()';
}


}

/// @nodoc
class $FilesMigrationEventCopyWith<$Res>  {
$FilesMigrationEventCopyWith(FilesMigrationEvent _, $Res Function(FilesMigrationEvent) __);
}


/// Adds pattern-matching-related methods to [FilesMigrationEvent].
extension FilesMigrationEventPatterns on FilesMigrationEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( FilesMigrationCheckedEvent value)?  checked,TResult Function( FilesMigrationOpenedEvent value)?  opened,TResult Function( FilesMigrationSealRequestedEvent value)?  sealRequested,TResult Function( FilesMigrationSealedEvent value)?  sealed,TResult Function( FilesMigrationUndoRequestedEvent value)?  undoRequested,TResult Function( FilesMigrationUndoneEvent value)?  undone,TResult Function( FilesMigrationPostponedEvent value)?  postponed,TResult Function( FilesMigrationCancelledEvent value)?  cancelled,TResult Function( FilesMigrationProgressedEvent value)?  progressed,TResult Function( FilesMigrationDismissedEvent value)?  dismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case FilesMigrationCheckedEvent() when checked != null:
return checked(_that);case FilesMigrationOpenedEvent() when opened != null:
return opened(_that);case FilesMigrationSealRequestedEvent() when sealRequested != null:
return sealRequested(_that);case FilesMigrationSealedEvent() when sealed != null:
return sealed(_that);case FilesMigrationUndoRequestedEvent() when undoRequested != null:
return undoRequested(_that);case FilesMigrationUndoneEvent() when undone != null:
return undone(_that);case FilesMigrationPostponedEvent() when postponed != null:
return postponed(_that);case FilesMigrationCancelledEvent() when cancelled != null:
return cancelled(_that);case FilesMigrationProgressedEvent() when progressed != null:
return progressed(_that);case FilesMigrationDismissedEvent() when dismissed != null:
return dismissed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( FilesMigrationCheckedEvent value)  checked,required TResult Function( FilesMigrationOpenedEvent value)  opened,required TResult Function( FilesMigrationSealRequestedEvent value)  sealRequested,required TResult Function( FilesMigrationSealedEvent value)  sealed,required TResult Function( FilesMigrationUndoRequestedEvent value)  undoRequested,required TResult Function( FilesMigrationUndoneEvent value)  undone,required TResult Function( FilesMigrationPostponedEvent value)  postponed,required TResult Function( FilesMigrationCancelledEvent value)  cancelled,required TResult Function( FilesMigrationProgressedEvent value)  progressed,required TResult Function( FilesMigrationDismissedEvent value)  dismissed,}){
final _that = this;
switch (_that) {
case FilesMigrationCheckedEvent():
return checked(_that);case FilesMigrationOpenedEvent():
return opened(_that);case FilesMigrationSealRequestedEvent():
return sealRequested(_that);case FilesMigrationSealedEvent():
return sealed(_that);case FilesMigrationUndoRequestedEvent():
return undoRequested(_that);case FilesMigrationUndoneEvent():
return undone(_that);case FilesMigrationPostponedEvent():
return postponed(_that);case FilesMigrationCancelledEvent():
return cancelled(_that);case FilesMigrationProgressedEvent():
return progressed(_that);case FilesMigrationDismissedEvent():
return dismissed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( FilesMigrationCheckedEvent value)?  checked,TResult? Function( FilesMigrationOpenedEvent value)?  opened,TResult? Function( FilesMigrationSealRequestedEvent value)?  sealRequested,TResult? Function( FilesMigrationSealedEvent value)?  sealed,TResult? Function( FilesMigrationUndoRequestedEvent value)?  undoRequested,TResult? Function( FilesMigrationUndoneEvent value)?  undone,TResult? Function( FilesMigrationPostponedEvent value)?  postponed,TResult? Function( FilesMigrationCancelledEvent value)?  cancelled,TResult? Function( FilesMigrationProgressedEvent value)?  progressed,TResult? Function( FilesMigrationDismissedEvent value)?  dismissed,}){
final _that = this;
switch (_that) {
case FilesMigrationCheckedEvent() when checked != null:
return checked(_that);case FilesMigrationOpenedEvent() when opened != null:
return opened(_that);case FilesMigrationSealRequestedEvent() when sealRequested != null:
return sealRequested(_that);case FilesMigrationSealedEvent() when sealed != null:
return sealed(_that);case FilesMigrationUndoRequestedEvent() when undoRequested != null:
return undoRequested(_that);case FilesMigrationUndoneEvent() when undone != null:
return undone(_that);case FilesMigrationPostponedEvent() when postponed != null:
return postponed(_that);case FilesMigrationCancelledEvent() when cancelled != null:
return cancelled(_that);case FilesMigrationProgressedEvent() when progressed != null:
return progressed(_that);case FilesMigrationDismissedEvent() when dismissed != null:
return dismissed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( bool launch)?  checked,TResult Function()?  opened,TResult Function()?  sealRequested,TResult Function( String? password)?  sealed,TResult Function()?  undoRequested,TResult Function( String? password)?  undone,TResult Function()?  postponed,TResult Function()?  cancelled,TResult Function( SealProgress progress)?  progressed,TResult Function()?  dismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case FilesMigrationCheckedEvent() when checked != null:
return checked(_that.launch);case FilesMigrationOpenedEvent() when opened != null:
return opened();case FilesMigrationSealRequestedEvent() when sealRequested != null:
return sealRequested();case FilesMigrationSealedEvent() when sealed != null:
return sealed(_that.password);case FilesMigrationUndoRequestedEvent() when undoRequested != null:
return undoRequested();case FilesMigrationUndoneEvent() when undone != null:
return undone(_that.password);case FilesMigrationPostponedEvent() when postponed != null:
return postponed();case FilesMigrationCancelledEvent() when cancelled != null:
return cancelled();case FilesMigrationProgressedEvent() when progressed != null:
return progressed(_that.progress);case FilesMigrationDismissedEvent() when dismissed != null:
return dismissed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( bool launch)  checked,required TResult Function()  opened,required TResult Function()  sealRequested,required TResult Function( String? password)  sealed,required TResult Function()  undoRequested,required TResult Function( String? password)  undone,required TResult Function()  postponed,required TResult Function()  cancelled,required TResult Function( SealProgress progress)  progressed,required TResult Function()  dismissed,}) {final _that = this;
switch (_that) {
case FilesMigrationCheckedEvent():
return checked(_that.launch);case FilesMigrationOpenedEvent():
return opened();case FilesMigrationSealRequestedEvent():
return sealRequested();case FilesMigrationSealedEvent():
return sealed(_that.password);case FilesMigrationUndoRequestedEvent():
return undoRequested();case FilesMigrationUndoneEvent():
return undone(_that.password);case FilesMigrationPostponedEvent():
return postponed();case FilesMigrationCancelledEvent():
return cancelled();case FilesMigrationProgressedEvent():
return progressed(_that.progress);case FilesMigrationDismissedEvent():
return dismissed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( bool launch)?  checked,TResult? Function()?  opened,TResult? Function()?  sealRequested,TResult? Function( String? password)?  sealed,TResult? Function()?  undoRequested,TResult? Function( String? password)?  undone,TResult? Function()?  postponed,TResult? Function()?  cancelled,TResult? Function( SealProgress progress)?  progressed,TResult? Function()?  dismissed,}) {final _that = this;
switch (_that) {
case FilesMigrationCheckedEvent() when checked != null:
return checked(_that.launch);case FilesMigrationOpenedEvent() when opened != null:
return opened();case FilesMigrationSealRequestedEvent() when sealRequested != null:
return sealRequested();case FilesMigrationSealedEvent() when sealed != null:
return sealed(_that.password);case FilesMigrationUndoRequestedEvent() when undoRequested != null:
return undoRequested();case FilesMigrationUndoneEvent() when undone != null:
return undone(_that.password);case FilesMigrationPostponedEvent() when postponed != null:
return postponed();case FilesMigrationCancelledEvent() when cancelled != null:
return cancelled();case FilesMigrationProgressedEvent() when progressed != null:
return progressed(_that.progress);case FilesMigrationDismissedEvent() when dismissed != null:
return dismissed();case _:
  return null;

}
}

}

/// @nodoc


class FilesMigrationCheckedEvent implements FilesMigrationEvent {
  const FilesMigrationCheckedEvent({this.launch = false});
  

@JsonKey() final  bool launch;

/// Create a copy of FilesMigrationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FilesMigrationCheckedEventCopyWith<FilesMigrationCheckedEvent> get copyWith => _$FilesMigrationCheckedEventCopyWithImpl<FilesMigrationCheckedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationCheckedEvent&&(identical(other.launch, launch) || other.launch == launch));
}


@override
int get hashCode => Object.hash(runtimeType,launch);

@override
String toString() {
  return 'FilesMigrationEvent.checked(launch: $launch)';
}


}

/// @nodoc
abstract mixin class $FilesMigrationCheckedEventCopyWith<$Res> implements $FilesMigrationEventCopyWith<$Res> {
  factory $FilesMigrationCheckedEventCopyWith(FilesMigrationCheckedEvent value, $Res Function(FilesMigrationCheckedEvent) _then) = _$FilesMigrationCheckedEventCopyWithImpl;
@useResult
$Res call({
 bool launch
});




}
/// @nodoc
class _$FilesMigrationCheckedEventCopyWithImpl<$Res>
    implements $FilesMigrationCheckedEventCopyWith<$Res> {
  _$FilesMigrationCheckedEventCopyWithImpl(this._self, this._then);

  final FilesMigrationCheckedEvent _self;
  final $Res Function(FilesMigrationCheckedEvent) _then;

/// Create a copy of FilesMigrationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? launch = null,}) {
  return _then(FilesMigrationCheckedEvent(
launch: null == launch ? _self.launch : launch // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class FilesMigrationOpenedEvent implements FilesMigrationEvent {
  const FilesMigrationOpenedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationOpenedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FilesMigrationEvent.opened()';
}


}




/// @nodoc


class FilesMigrationSealRequestedEvent implements FilesMigrationEvent {
  const FilesMigrationSealRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationSealRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FilesMigrationEvent.sealRequested()';
}


}




/// @nodoc


class FilesMigrationSealedEvent implements FilesMigrationEvent {
  const FilesMigrationSealedEvent({this.password});
  

 final  String? password;

/// Create a copy of FilesMigrationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FilesMigrationSealedEventCopyWith<FilesMigrationSealedEvent> get copyWith => _$FilesMigrationSealedEventCopyWithImpl<FilesMigrationSealedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationSealedEvent&&(identical(other.password, password) || other.password == password));
}


@override
int get hashCode => Object.hash(runtimeType,password);

@override
String toString() {
  return 'FilesMigrationEvent.sealed(password: $password)';
}


}

/// @nodoc
abstract mixin class $FilesMigrationSealedEventCopyWith<$Res> implements $FilesMigrationEventCopyWith<$Res> {
  factory $FilesMigrationSealedEventCopyWith(FilesMigrationSealedEvent value, $Res Function(FilesMigrationSealedEvent) _then) = _$FilesMigrationSealedEventCopyWithImpl;
@useResult
$Res call({
 String? password
});




}
/// @nodoc
class _$FilesMigrationSealedEventCopyWithImpl<$Res>
    implements $FilesMigrationSealedEventCopyWith<$Res> {
  _$FilesMigrationSealedEventCopyWithImpl(this._self, this._then);

  final FilesMigrationSealedEvent _self;
  final $Res Function(FilesMigrationSealedEvent) _then;

/// Create a copy of FilesMigrationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? password = freezed,}) {
  return _then(FilesMigrationSealedEvent(
password: freezed == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class FilesMigrationUndoRequestedEvent implements FilesMigrationEvent {
  const FilesMigrationUndoRequestedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationUndoRequestedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FilesMigrationEvent.undoRequested()';
}


}




/// @nodoc


class FilesMigrationUndoneEvent implements FilesMigrationEvent {
  const FilesMigrationUndoneEvent({this.password});
  

 final  String? password;

/// Create a copy of FilesMigrationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FilesMigrationUndoneEventCopyWith<FilesMigrationUndoneEvent> get copyWith => _$FilesMigrationUndoneEventCopyWithImpl<FilesMigrationUndoneEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationUndoneEvent&&(identical(other.password, password) || other.password == password));
}


@override
int get hashCode => Object.hash(runtimeType,password);

@override
String toString() {
  return 'FilesMigrationEvent.undone(password: $password)';
}


}

/// @nodoc
abstract mixin class $FilesMigrationUndoneEventCopyWith<$Res> implements $FilesMigrationEventCopyWith<$Res> {
  factory $FilesMigrationUndoneEventCopyWith(FilesMigrationUndoneEvent value, $Res Function(FilesMigrationUndoneEvent) _then) = _$FilesMigrationUndoneEventCopyWithImpl;
@useResult
$Res call({
 String? password
});




}
/// @nodoc
class _$FilesMigrationUndoneEventCopyWithImpl<$Res>
    implements $FilesMigrationUndoneEventCopyWith<$Res> {
  _$FilesMigrationUndoneEventCopyWithImpl(this._self, this._then);

  final FilesMigrationUndoneEvent _self;
  final $Res Function(FilesMigrationUndoneEvent) _then;

/// Create a copy of FilesMigrationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? password = freezed,}) {
  return _then(FilesMigrationUndoneEvent(
password: freezed == password ? _self.password : password // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class FilesMigrationPostponedEvent implements FilesMigrationEvent {
  const FilesMigrationPostponedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationPostponedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FilesMigrationEvent.postponed()';
}


}




/// @nodoc


class FilesMigrationCancelledEvent implements FilesMigrationEvent {
  const FilesMigrationCancelledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationCancelledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FilesMigrationEvent.cancelled()';
}


}




/// @nodoc


class FilesMigrationProgressedEvent implements FilesMigrationEvent {
  const FilesMigrationProgressedEvent(this.progress);
  

 final  SealProgress progress;

/// Create a copy of FilesMigrationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FilesMigrationProgressedEventCopyWith<FilesMigrationProgressedEvent> get copyWith => _$FilesMigrationProgressedEventCopyWithImpl<FilesMigrationProgressedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationProgressedEvent&&(identical(other.progress, progress) || other.progress == progress));
}


@override
int get hashCode => Object.hash(runtimeType,progress);

@override
String toString() {
  return 'FilesMigrationEvent.progressed(progress: $progress)';
}


}

/// @nodoc
abstract mixin class $FilesMigrationProgressedEventCopyWith<$Res> implements $FilesMigrationEventCopyWith<$Res> {
  factory $FilesMigrationProgressedEventCopyWith(FilesMigrationProgressedEvent value, $Res Function(FilesMigrationProgressedEvent) _then) = _$FilesMigrationProgressedEventCopyWithImpl;
@useResult
$Res call({
 SealProgress progress
});




}
/// @nodoc
class _$FilesMigrationProgressedEventCopyWithImpl<$Res>
    implements $FilesMigrationProgressedEventCopyWith<$Res> {
  _$FilesMigrationProgressedEventCopyWithImpl(this._self, this._then);

  final FilesMigrationProgressedEvent _self;
  final $Res Function(FilesMigrationProgressedEvent) _then;

/// Create a copy of FilesMigrationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? progress = null,}) {
  return _then(FilesMigrationProgressedEvent(
null == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as SealProgress,
  ));
}


}

/// @nodoc


class FilesMigrationDismissedEvent implements FilesMigrationEvent {
  const FilesMigrationDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'FilesMigrationEvent.dismissed()';
}


}




/// @nodoc
mixin _$FilesMigrationState {

 MigrationStage get stage; MigrationAction get action;/// What waits, as last read.
 FilesMigration? get info;/// Whether the sheet is the first-launch prompt, whose "Later" is
/// remembered.
 bool get launch;/// How far sealing has come.
 SealProgress? get progress;/// Whether Cancel was pressed and the pass is finishing its file.
 bool get cancelling;/// What a finished pass did.
 MigrationOutcome? get outcome;/// How many files an undo made readable again.
 int get undone; ApplicationError? get error;
/// Create a copy of FilesMigrationState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$FilesMigrationStateCopyWith<FilesMigrationState> get copyWith => _$FilesMigrationStateCopyWithImpl<FilesMigrationState>(this as FilesMigrationState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is FilesMigrationState&&(identical(other.stage, stage) || other.stage == stage)&&(identical(other.action, action) || other.action == action)&&(identical(other.info, info) || other.info == info)&&(identical(other.launch, launch) || other.launch == launch)&&(identical(other.progress, progress) || other.progress == progress)&&(identical(other.cancelling, cancelling) || other.cancelling == cancelling)&&(identical(other.outcome, outcome) || other.outcome == outcome)&&(identical(other.undone, undone) || other.undone == undone)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,stage,action,info,launch,progress,cancelling,outcome,undone,error);

@override
String toString() {
  return 'FilesMigrationState(stage: $stage, action: $action, info: $info, launch: $launch, progress: $progress, cancelling: $cancelling, outcome: $outcome, undone: $undone, error: $error)';
}


}

/// @nodoc
abstract mixin class $FilesMigrationStateCopyWith<$Res>  {
  factory $FilesMigrationStateCopyWith(FilesMigrationState value, $Res Function(FilesMigrationState) _then) = _$FilesMigrationStateCopyWithImpl;
@useResult
$Res call({
 MigrationStage stage, MigrationAction action, FilesMigration? info, bool launch, SealProgress? progress, bool cancelling, MigrationOutcome? outcome, int undone, ApplicationError? error
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$FilesMigrationStateCopyWithImpl<$Res>
    implements $FilesMigrationStateCopyWith<$Res> {
  _$FilesMigrationStateCopyWithImpl(this._self, this._then);

  final FilesMigrationState _self;
  final $Res Function(FilesMigrationState) _then;

/// Create a copy of FilesMigrationState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? stage = null,Object? action = null,Object? info = freezed,Object? launch = null,Object? progress = freezed,Object? cancelling = null,Object? outcome = freezed,Object? undone = null,Object? error = freezed,}) {
  return _then(_self.copyWith(
stage: null == stage ? _self.stage : stage // ignore: cast_nullable_to_non_nullable
as MigrationStage,action: null == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as MigrationAction,info: freezed == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as FilesMigration?,launch: null == launch ? _self.launch : launch // ignore: cast_nullable_to_non_nullable
as bool,progress: freezed == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as SealProgress?,cancelling: null == cancelling ? _self.cancelling : cancelling // ignore: cast_nullable_to_non_nullable
as bool,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as MigrationOutcome?,undone: null == undone ? _self.undone : undone // ignore: cast_nullable_to_non_nullable
as int,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}
/// Create a copy of FilesMigrationState
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


/// Adds pattern-matching-related methods to [FilesMigrationState].
extension FilesMigrationStatePatterns on FilesMigrationState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _FilesMigrationState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _FilesMigrationState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _FilesMigrationState value)  $default,){
final _that = this;
switch (_that) {
case _FilesMigrationState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _FilesMigrationState value)?  $default,){
final _that = this;
switch (_that) {
case _FilesMigrationState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( MigrationStage stage,  MigrationAction action,  FilesMigration? info,  bool launch,  SealProgress? progress,  bool cancelling,  MigrationOutcome? outcome,  int undone,  ApplicationError? error)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _FilesMigrationState() when $default != null:
return $default(_that.stage,_that.action,_that.info,_that.launch,_that.progress,_that.cancelling,_that.outcome,_that.undone,_that.error);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( MigrationStage stage,  MigrationAction action,  FilesMigration? info,  bool launch,  SealProgress? progress,  bool cancelling,  MigrationOutcome? outcome,  int undone,  ApplicationError? error)  $default,) {final _that = this;
switch (_that) {
case _FilesMigrationState():
return $default(_that.stage,_that.action,_that.info,_that.launch,_that.progress,_that.cancelling,_that.outcome,_that.undone,_that.error);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( MigrationStage stage,  MigrationAction action,  FilesMigration? info,  bool launch,  SealProgress? progress,  bool cancelling,  MigrationOutcome? outcome,  int undone,  ApplicationError? error)?  $default,) {final _that = this;
switch (_that) {
case _FilesMigrationState() when $default != null:
return $default(_that.stage,_that.action,_that.info,_that.launch,_that.progress,_that.cancelling,_that.outcome,_that.undone,_that.error);case _:
  return null;

}
}

}

/// @nodoc


class _FilesMigrationState implements FilesMigrationState {
  const _FilesMigrationState({this.stage = MigrationStage.checking, this.action = MigrationAction.seal, this.info, this.launch = false, this.progress, this.cancelling = false, this.outcome, this.undone = 0, this.error});
  

@override@JsonKey() final  MigrationStage stage;
@override@JsonKey() final  MigrationAction action;
/// What waits, as last read.
@override final  FilesMigration? info;
/// Whether the sheet is the first-launch prompt, whose "Later" is
/// remembered.
@override@JsonKey() final  bool launch;
/// How far sealing has come.
@override final  SealProgress? progress;
/// Whether Cancel was pressed and the pass is finishing its file.
@override@JsonKey() final  bool cancelling;
/// What a finished pass did.
@override final  MigrationOutcome? outcome;
/// How many files an undo made readable again.
@override@JsonKey() final  int undone;
@override final  ApplicationError? error;

/// Create a copy of FilesMigrationState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$FilesMigrationStateCopyWith<_FilesMigrationState> get copyWith => __$FilesMigrationStateCopyWithImpl<_FilesMigrationState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _FilesMigrationState&&(identical(other.stage, stage) || other.stage == stage)&&(identical(other.action, action) || other.action == action)&&(identical(other.info, info) || other.info == info)&&(identical(other.launch, launch) || other.launch == launch)&&(identical(other.progress, progress) || other.progress == progress)&&(identical(other.cancelling, cancelling) || other.cancelling == cancelling)&&(identical(other.outcome, outcome) || other.outcome == outcome)&&(identical(other.undone, undone) || other.undone == undone)&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,stage,action,info,launch,progress,cancelling,outcome,undone,error);

@override
String toString() {
  return 'FilesMigrationState(stage: $stage, action: $action, info: $info, launch: $launch, progress: $progress, cancelling: $cancelling, outcome: $outcome, undone: $undone, error: $error)';
}


}

/// @nodoc
abstract mixin class _$FilesMigrationStateCopyWith<$Res> implements $FilesMigrationStateCopyWith<$Res> {
  factory _$FilesMigrationStateCopyWith(_FilesMigrationState value, $Res Function(_FilesMigrationState) _then) = __$FilesMigrationStateCopyWithImpl;
@override @useResult
$Res call({
 MigrationStage stage, MigrationAction action, FilesMigration? info, bool launch, SealProgress? progress, bool cancelling, MigrationOutcome? outcome, int undone, ApplicationError? error
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$FilesMigrationStateCopyWithImpl<$Res>
    implements _$FilesMigrationStateCopyWith<$Res> {
  __$FilesMigrationStateCopyWithImpl(this._self, this._then);

  final _FilesMigrationState _self;
  final $Res Function(_FilesMigrationState) _then;

/// Create a copy of FilesMigrationState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? stage = null,Object? action = null,Object? info = freezed,Object? launch = null,Object? progress = freezed,Object? cancelling = null,Object? outcome = freezed,Object? undone = null,Object? error = freezed,}) {
  return _then(_FilesMigrationState(
stage: null == stage ? _self.stage : stage // ignore: cast_nullable_to_non_nullable
as MigrationStage,action: null == action ? _self.action : action // ignore: cast_nullable_to_non_nullable
as MigrationAction,info: freezed == info ? _self.info : info // ignore: cast_nullable_to_non_nullable
as FilesMigration?,launch: null == launch ? _self.launch : launch // ignore: cast_nullable_to_non_nullable
as bool,progress: freezed == progress ? _self.progress : progress // ignore: cast_nullable_to_non_nullable
as SealProgress?,cancelling: null == cancelling ? _self.cancelling : cancelling // ignore: cast_nullable_to_non_nullable
as bool,outcome: freezed == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as MigrationOutcome?,undone: null == undone ? _self.undone : undone // ignore: cast_nullable_to_non_nullable
as int,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,
  ));
}

/// Create a copy of FilesMigrationState
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

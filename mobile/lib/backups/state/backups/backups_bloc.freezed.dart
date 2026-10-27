// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'backups_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$BackupsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BackupsEvent()';
}


}

/// @nodoc
class $BackupsEventCopyWith<$Res>  {
$BackupsEventCopyWith(BackupsEvent _, $Res Function(BackupsEvent) __);
}


/// Adds pattern-matching-related methods to [BackupsEvent].
extension BackupsEventPatterns on BackupsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( BackupsStartedEvent value)?  started,TResult Function( BackupsRefreshedEvent value)?  refreshed,TResult Function( BackupsBackUpNowEvent value)?  backUpNow,TResult Function( BackupsDeleteEvent value)?  delete,TResult Function( BackupsNoticeClearedEvent value)?  noticeCleared,required TResult orElse(),}){
final _that = this;
switch (_that) {
case BackupsStartedEvent() when started != null:
return started(_that);case BackupsRefreshedEvent() when refreshed != null:
return refreshed(_that);case BackupsBackUpNowEvent() when backUpNow != null:
return backUpNow(_that);case BackupsDeleteEvent() when delete != null:
return delete(_that);case BackupsNoticeClearedEvent() when noticeCleared != null:
return noticeCleared(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( BackupsStartedEvent value)  started,required TResult Function( BackupsRefreshedEvent value)  refreshed,required TResult Function( BackupsBackUpNowEvent value)  backUpNow,required TResult Function( BackupsDeleteEvent value)  delete,required TResult Function( BackupsNoticeClearedEvent value)  noticeCleared,}){
final _that = this;
switch (_that) {
case BackupsStartedEvent():
return started(_that);case BackupsRefreshedEvent():
return refreshed(_that);case BackupsBackUpNowEvent():
return backUpNow(_that);case BackupsDeleteEvent():
return delete(_that);case BackupsNoticeClearedEvent():
return noticeCleared(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( BackupsStartedEvent value)?  started,TResult? Function( BackupsRefreshedEvent value)?  refreshed,TResult? Function( BackupsBackUpNowEvent value)?  backUpNow,TResult? Function( BackupsDeleteEvent value)?  delete,TResult? Function( BackupsNoticeClearedEvent value)?  noticeCleared,}){
final _that = this;
switch (_that) {
case BackupsStartedEvent() when started != null:
return started(_that);case BackupsRefreshedEvent() when refreshed != null:
return refreshed(_that);case BackupsBackUpNowEvent() when backUpNow != null:
return backUpNow(_that);case BackupsDeleteEvent() when delete != null:
return delete(_that);case BackupsNoticeClearedEvent() when noticeCleared != null:
return noticeCleared(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  refreshed,TResult Function()?  backUpNow,TResult Function( String path)?  delete,TResult Function()?  noticeCleared,required TResult orElse(),}) {final _that = this;
switch (_that) {
case BackupsStartedEvent() when started != null:
return started();case BackupsRefreshedEvent() when refreshed != null:
return refreshed();case BackupsBackUpNowEvent() when backUpNow != null:
return backUpNow();case BackupsDeleteEvent() when delete != null:
return delete(_that.path);case BackupsNoticeClearedEvent() when noticeCleared != null:
return noticeCleared();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  refreshed,required TResult Function()  backUpNow,required TResult Function( String path)  delete,required TResult Function()  noticeCleared,}) {final _that = this;
switch (_that) {
case BackupsStartedEvent():
return started();case BackupsRefreshedEvent():
return refreshed();case BackupsBackUpNowEvent():
return backUpNow();case BackupsDeleteEvent():
return delete(_that.path);case BackupsNoticeClearedEvent():
return noticeCleared();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  refreshed,TResult? Function()?  backUpNow,TResult? Function( String path)?  delete,TResult? Function()?  noticeCleared,}) {final _that = this;
switch (_that) {
case BackupsStartedEvent() when started != null:
return started();case BackupsRefreshedEvent() when refreshed != null:
return refreshed();case BackupsBackUpNowEvent() when backUpNow != null:
return backUpNow();case BackupsDeleteEvent() when delete != null:
return delete(_that.path);case BackupsNoticeClearedEvent() when noticeCleared != null:
return noticeCleared();case _:
  return null;

}
}

}

/// @nodoc


class BackupsStartedEvent implements BackupsEvent {
  const BackupsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BackupsEvent.started()';
}


}




/// @nodoc


class BackupsRefreshedEvent implements BackupsEvent {
  const BackupsRefreshedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupsRefreshedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BackupsEvent.refreshed()';
}


}




/// @nodoc


class BackupsBackUpNowEvent implements BackupsEvent {
  const BackupsBackUpNowEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupsBackUpNowEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BackupsEvent.backUpNow()';
}


}




/// @nodoc


class BackupsDeleteEvent implements BackupsEvent {
  const BackupsDeleteEvent(this.path);
  

 final  String path;

/// Create a copy of BackupsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BackupsDeleteEventCopyWith<BackupsDeleteEvent> get copyWith => _$BackupsDeleteEventCopyWithImpl<BackupsDeleteEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupsDeleteEvent&&(identical(other.path, path) || other.path == path));
}


@override
int get hashCode => Object.hash(runtimeType,path);

@override
String toString() {
  return 'BackupsEvent.delete(path: $path)';
}


}

/// @nodoc
abstract mixin class $BackupsDeleteEventCopyWith<$Res> implements $BackupsEventCopyWith<$Res> {
  factory $BackupsDeleteEventCopyWith(BackupsDeleteEvent value, $Res Function(BackupsDeleteEvent) _then) = _$BackupsDeleteEventCopyWithImpl;
@useResult
$Res call({
 String path
});




}
/// @nodoc
class _$BackupsDeleteEventCopyWithImpl<$Res>
    implements $BackupsDeleteEventCopyWith<$Res> {
  _$BackupsDeleteEventCopyWithImpl(this._self, this._then);

  final BackupsDeleteEvent _self;
  final $Res Function(BackupsDeleteEvent) _then;

/// Create a copy of BackupsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? path = null,}) {
  return _then(BackupsDeleteEvent(
null == path ? _self.path : path // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class BackupsNoticeClearedEvent implements BackupsEvent {
  const BackupsNoticeClearedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupsNoticeClearedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BackupsEvent.noticeCleared()';
}


}




/// @nodoc
mixin _$BackupsState {

 LoadStatus get status;/// The backups, newest first.
 List<BackupFile> get backups;/// A backup is being made on request.
 bool get working;/// The backups being deleted now, by path.
 Set<String> get deleting;/// What was just done.
 BackupsNotice? get notice;/// Why the last thing did not happen.
 BackupsProblem? get problem;
/// Create a copy of BackupsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BackupsStateCopyWith<BackupsState> get copyWith => _$BackupsStateCopyWithImpl<BackupsState>(this as BackupsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BackupsState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.backups, backups)&&(identical(other.working, working) || other.working == working)&&const DeepCollectionEquality().equals(other.deleting, deleting)&&(identical(other.notice, notice) || other.notice == notice)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(backups),working,const DeepCollectionEquality().hash(deleting),notice,problem);

@override
String toString() {
  return 'BackupsState(status: $status, backups: $backups, working: $working, deleting: $deleting, notice: $notice, problem: $problem)';
}


}

/// @nodoc
abstract mixin class $BackupsStateCopyWith<$Res>  {
  factory $BackupsStateCopyWith(BackupsState value, $Res Function(BackupsState) _then) = _$BackupsStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<BackupFile> backups, bool working, Set<String> deleting, BackupsNotice? notice, BackupsProblem? problem
});




}
/// @nodoc
class _$BackupsStateCopyWithImpl<$Res>
    implements $BackupsStateCopyWith<$Res> {
  _$BackupsStateCopyWithImpl(this._self, this._then);

  final BackupsState _self;
  final $Res Function(BackupsState) _then;

/// Create a copy of BackupsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? backups = null,Object? working = null,Object? deleting = null,Object? notice = freezed,Object? problem = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,backups: null == backups ? _self.backups : backups // ignore: cast_nullable_to_non_nullable
as List<BackupFile>,working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as bool,deleting: null == deleting ? _self.deleting : deleting // ignore: cast_nullable_to_non_nullable
as Set<String>,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as BackupsNotice?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as BackupsProblem?,
  ));
}

}


/// Adds pattern-matching-related methods to [BackupsState].
extension BackupsStatePatterns on BackupsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _BackupsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _BackupsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _BackupsState value)  $default,){
final _that = this;
switch (_that) {
case _BackupsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _BackupsState value)?  $default,){
final _that = this;
switch (_that) {
case _BackupsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<BackupFile> backups,  bool working,  Set<String> deleting,  BackupsNotice? notice,  BackupsProblem? problem)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _BackupsState() when $default != null:
return $default(_that.status,_that.backups,_that.working,_that.deleting,_that.notice,_that.problem);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<BackupFile> backups,  bool working,  Set<String> deleting,  BackupsNotice? notice,  BackupsProblem? problem)  $default,) {final _that = this;
switch (_that) {
case _BackupsState():
return $default(_that.status,_that.backups,_that.working,_that.deleting,_that.notice,_that.problem);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<BackupFile> backups,  bool working,  Set<String> deleting,  BackupsNotice? notice,  BackupsProblem? problem)?  $default,) {final _that = this;
switch (_that) {
case _BackupsState() when $default != null:
return $default(_that.status,_that.backups,_that.working,_that.deleting,_that.notice,_that.problem);case _:
  return null;

}
}

}

/// @nodoc


class _BackupsState implements BackupsState {
  const _BackupsState({this.status = LoadStatus.loading, final  List<BackupFile> backups = const <BackupFile>[], this.working = false, final  Set<String> deleting = const <String>{}, this.notice, this.problem}): _backups = backups,_deleting = deleting;
  

@override@JsonKey() final  LoadStatus status;
/// The backups, newest first.
 final  List<BackupFile> _backups;
/// The backups, newest first.
@override@JsonKey() List<BackupFile> get backups {
  if (_backups is EqualUnmodifiableListView) return _backups;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_backups);
}

/// A backup is being made on request.
@override@JsonKey() final  bool working;
/// The backups being deleted now, by path.
 final  Set<String> _deleting;
/// The backups being deleted now, by path.
@override@JsonKey() Set<String> get deleting {
  if (_deleting is EqualUnmodifiableSetView) return _deleting;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_deleting);
}

/// What was just done.
@override final  BackupsNotice? notice;
/// Why the last thing did not happen.
@override final  BackupsProblem? problem;

/// Create a copy of BackupsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$BackupsStateCopyWith<_BackupsState> get copyWith => __$BackupsStateCopyWithImpl<_BackupsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _BackupsState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._backups, _backups)&&(identical(other.working, working) || other.working == working)&&const DeepCollectionEquality().equals(other._deleting, _deleting)&&(identical(other.notice, notice) || other.notice == notice)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_backups),working,const DeepCollectionEquality().hash(_deleting),notice,problem);

@override
String toString() {
  return 'BackupsState(status: $status, backups: $backups, working: $working, deleting: $deleting, notice: $notice, problem: $problem)';
}


}

/// @nodoc
abstract mixin class _$BackupsStateCopyWith<$Res> implements $BackupsStateCopyWith<$Res> {
  factory _$BackupsStateCopyWith(_BackupsState value, $Res Function(_BackupsState) _then) = __$BackupsStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<BackupFile> backups, bool working, Set<String> deleting, BackupsNotice? notice, BackupsProblem? problem
});




}
/// @nodoc
class __$BackupsStateCopyWithImpl<$Res>
    implements _$BackupsStateCopyWith<$Res> {
  __$BackupsStateCopyWithImpl(this._self, this._then);

  final _BackupsState _self;
  final $Res Function(_BackupsState) _then;

/// Create a copy of BackupsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? backups = null,Object? working = null,Object? deleting = null,Object? notice = freezed,Object? problem = freezed,}) {
  return _then(_BackupsState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,backups: null == backups ? _self._backups : backups // ignore: cast_nullable_to_non_nullable
as List<BackupFile>,working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as bool,deleting: null == deleting ? _self._deleting : deleting // ignore: cast_nullable_to_non_nullable
as Set<String>,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as BackupsNotice?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as BackupsProblem?,
  ));
}


}

// dart format on

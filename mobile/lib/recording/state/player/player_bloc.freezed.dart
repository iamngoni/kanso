// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'player_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PlayerEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlayerEvent()';
}


}

/// @nodoc
class $PlayerEventCopyWith<$Res>  {
$PlayerEventCopyWith(PlayerEvent _, $Res Function(PlayerEvent) __);
}


/// Adds pattern-matching-related methods to [PlayerEvent].
extension PlayerEventPatterns on PlayerEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PlayerStartedEvent value)?  started,TResult Function( PlayerRefreshedEvent value)?  refreshed,TResult Function( PlayerToggledEvent value)?  toggled,TResult Function( PlayerSeekedEvent value)?  seeked,TResult Function( PlayerSkippedEvent value)?  skipped,TResult Function( PlayerSpeedCycledEvent value)?  speedCycled,TResult Function( PlayerVolumeChangedEvent value)?  volumeChanged,TResult Function( PlayerMuteToggledEvent value)?  muteToggled,TResult Function( PlayerMomentTappedEvent value)?  momentTapped,TResult Function( PlayerRetriedEvent value)?  retried,TResult Function( PlayerTickedEvent value)?  ticked,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PlayerStartedEvent() when started != null:
return started(_that);case PlayerRefreshedEvent() when refreshed != null:
return refreshed(_that);case PlayerToggledEvent() when toggled != null:
return toggled(_that);case PlayerSeekedEvent() when seeked != null:
return seeked(_that);case PlayerSkippedEvent() when skipped != null:
return skipped(_that);case PlayerSpeedCycledEvent() when speedCycled != null:
return speedCycled(_that);case PlayerVolumeChangedEvent() when volumeChanged != null:
return volumeChanged(_that);case PlayerMuteToggledEvent() when muteToggled != null:
return muteToggled(_that);case PlayerMomentTappedEvent() when momentTapped != null:
return momentTapped(_that);case PlayerRetriedEvent() when retried != null:
return retried(_that);case PlayerTickedEvent() when ticked != null:
return ticked(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PlayerStartedEvent value)  started,required TResult Function( PlayerRefreshedEvent value)  refreshed,required TResult Function( PlayerToggledEvent value)  toggled,required TResult Function( PlayerSeekedEvent value)  seeked,required TResult Function( PlayerSkippedEvent value)  skipped,required TResult Function( PlayerSpeedCycledEvent value)  speedCycled,required TResult Function( PlayerVolumeChangedEvent value)  volumeChanged,required TResult Function( PlayerMuteToggledEvent value)  muteToggled,required TResult Function( PlayerMomentTappedEvent value)  momentTapped,required TResult Function( PlayerRetriedEvent value)  retried,required TResult Function( PlayerTickedEvent value)  ticked,}){
final _that = this;
switch (_that) {
case PlayerStartedEvent():
return started(_that);case PlayerRefreshedEvent():
return refreshed(_that);case PlayerToggledEvent():
return toggled(_that);case PlayerSeekedEvent():
return seeked(_that);case PlayerSkippedEvent():
return skipped(_that);case PlayerSpeedCycledEvent():
return speedCycled(_that);case PlayerVolumeChangedEvent():
return volumeChanged(_that);case PlayerMuteToggledEvent():
return muteToggled(_that);case PlayerMomentTappedEvent():
return momentTapped(_that);case PlayerRetriedEvent():
return retried(_that);case PlayerTickedEvent():
return ticked(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PlayerStartedEvent value)?  started,TResult? Function( PlayerRefreshedEvent value)?  refreshed,TResult? Function( PlayerToggledEvent value)?  toggled,TResult? Function( PlayerSeekedEvent value)?  seeked,TResult? Function( PlayerSkippedEvent value)?  skipped,TResult? Function( PlayerSpeedCycledEvent value)?  speedCycled,TResult? Function( PlayerVolumeChangedEvent value)?  volumeChanged,TResult? Function( PlayerMuteToggledEvent value)?  muteToggled,TResult? Function( PlayerMomentTappedEvent value)?  momentTapped,TResult? Function( PlayerRetriedEvent value)?  retried,TResult? Function( PlayerTickedEvent value)?  ticked,}){
final _that = this;
switch (_that) {
case PlayerStartedEvent() when started != null:
return started(_that);case PlayerRefreshedEvent() when refreshed != null:
return refreshed(_that);case PlayerToggledEvent() when toggled != null:
return toggled(_that);case PlayerSeekedEvent() when seeked != null:
return seeked(_that);case PlayerSkippedEvent() when skipped != null:
return skipped(_that);case PlayerSpeedCycledEvent() when speedCycled != null:
return speedCycled(_that);case PlayerVolumeChangedEvent() when volumeChanged != null:
return volumeChanged(_that);case PlayerMuteToggledEvent() when muteToggled != null:
return muteToggled(_that);case PlayerMomentTappedEvent() when momentTapped != null:
return momentTapped(_that);case PlayerRetriedEvent() when retried != null:
return retried(_that);case PlayerTickedEvent() when ticked != null:
return ticked(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  started,TResult Function()?  refreshed,TResult Function()?  toggled,TResult Function( int ms)?  seeked,TResult Function( int deltaMs)?  skipped,TResult Function()?  speedCycled,TResult Function( double volume)?  volumeChanged,TResult Function()?  muteToggled,TResult Function( String fileId,  int seconds)?  momentTapped,TResult Function()?  retried,TResult Function()?  ticked,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PlayerStartedEvent() when started != null:
return started(_that.noteId);case PlayerRefreshedEvent() when refreshed != null:
return refreshed();case PlayerToggledEvent() when toggled != null:
return toggled();case PlayerSeekedEvent() when seeked != null:
return seeked(_that.ms);case PlayerSkippedEvent() when skipped != null:
return skipped(_that.deltaMs);case PlayerSpeedCycledEvent() when speedCycled != null:
return speedCycled();case PlayerVolumeChangedEvent() when volumeChanged != null:
return volumeChanged(_that.volume);case PlayerMuteToggledEvent() when muteToggled != null:
return muteToggled();case PlayerMomentTappedEvent() when momentTapped != null:
return momentTapped(_that.fileId,_that.seconds);case PlayerRetriedEvent() when retried != null:
return retried();case PlayerTickedEvent() when ticked != null:
return ticked();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  started,required TResult Function()  refreshed,required TResult Function()  toggled,required TResult Function( int ms)  seeked,required TResult Function( int deltaMs)  skipped,required TResult Function()  speedCycled,required TResult Function( double volume)  volumeChanged,required TResult Function()  muteToggled,required TResult Function( String fileId,  int seconds)  momentTapped,required TResult Function()  retried,required TResult Function()  ticked,}) {final _that = this;
switch (_that) {
case PlayerStartedEvent():
return started(_that.noteId);case PlayerRefreshedEvent():
return refreshed();case PlayerToggledEvent():
return toggled();case PlayerSeekedEvent():
return seeked(_that.ms);case PlayerSkippedEvent():
return skipped(_that.deltaMs);case PlayerSpeedCycledEvent():
return speedCycled();case PlayerVolumeChangedEvent():
return volumeChanged(_that.volume);case PlayerMuteToggledEvent():
return muteToggled();case PlayerMomentTappedEvent():
return momentTapped(_that.fileId,_that.seconds);case PlayerRetriedEvent():
return retried();case PlayerTickedEvent():
return ticked();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  started,TResult? Function()?  refreshed,TResult? Function()?  toggled,TResult? Function( int ms)?  seeked,TResult? Function( int deltaMs)?  skipped,TResult? Function()?  speedCycled,TResult? Function( double volume)?  volumeChanged,TResult? Function()?  muteToggled,TResult? Function( String fileId,  int seconds)?  momentTapped,TResult? Function()?  retried,TResult? Function()?  ticked,}) {final _that = this;
switch (_that) {
case PlayerStartedEvent() when started != null:
return started(_that.noteId);case PlayerRefreshedEvent() when refreshed != null:
return refreshed();case PlayerToggledEvent() when toggled != null:
return toggled();case PlayerSeekedEvent() when seeked != null:
return seeked(_that.ms);case PlayerSkippedEvent() when skipped != null:
return skipped(_that.deltaMs);case PlayerSpeedCycledEvent() when speedCycled != null:
return speedCycled();case PlayerVolumeChangedEvent() when volumeChanged != null:
return volumeChanged(_that.volume);case PlayerMuteToggledEvent() when muteToggled != null:
return muteToggled();case PlayerMomentTappedEvent() when momentTapped != null:
return momentTapped(_that.fileId,_that.seconds);case PlayerRetriedEvent() when retried != null:
return retried();case PlayerTickedEvent() when ticked != null:
return ticked();case _:
  return null;

}
}

}

/// @nodoc


class PlayerStartedEvent implements PlayerEvent {
  const PlayerStartedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of PlayerEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerStartedEventCopyWith<PlayerStartedEvent> get copyWith => _$PlayerStartedEventCopyWithImpl<PlayerStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'PlayerEvent.started(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $PlayerStartedEventCopyWith<$Res> implements $PlayerEventCopyWith<$Res> {
  factory $PlayerStartedEventCopyWith(PlayerStartedEvent value, $Res Function(PlayerStartedEvent) _then) = _$PlayerStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$PlayerStartedEventCopyWithImpl<$Res>
    implements $PlayerStartedEventCopyWith<$Res> {
  _$PlayerStartedEventCopyWithImpl(this._self, this._then);

  final PlayerStartedEvent _self;
  final $Res Function(PlayerStartedEvent) _then;

/// Create a copy of PlayerEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(PlayerStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PlayerRefreshedEvent implements PlayerEvent {
  const PlayerRefreshedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerRefreshedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlayerEvent.refreshed()';
}


}




/// @nodoc


class PlayerToggledEvent implements PlayerEvent {
  const PlayerToggledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerToggledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlayerEvent.toggled()';
}


}




/// @nodoc


class PlayerSeekedEvent implements PlayerEvent {
  const PlayerSeekedEvent(this.ms);
  

 final  int ms;

/// Create a copy of PlayerEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerSeekedEventCopyWith<PlayerSeekedEvent> get copyWith => _$PlayerSeekedEventCopyWithImpl<PlayerSeekedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerSeekedEvent&&(identical(other.ms, ms) || other.ms == ms));
}


@override
int get hashCode => Object.hash(runtimeType,ms);

@override
String toString() {
  return 'PlayerEvent.seeked(ms: $ms)';
}


}

/// @nodoc
abstract mixin class $PlayerSeekedEventCopyWith<$Res> implements $PlayerEventCopyWith<$Res> {
  factory $PlayerSeekedEventCopyWith(PlayerSeekedEvent value, $Res Function(PlayerSeekedEvent) _then) = _$PlayerSeekedEventCopyWithImpl;
@useResult
$Res call({
 int ms
});




}
/// @nodoc
class _$PlayerSeekedEventCopyWithImpl<$Res>
    implements $PlayerSeekedEventCopyWith<$Res> {
  _$PlayerSeekedEventCopyWithImpl(this._self, this._then);

  final PlayerSeekedEvent _self;
  final $Res Function(PlayerSeekedEvent) _then;

/// Create a copy of PlayerEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ms = null,}) {
  return _then(PlayerSeekedEvent(
null == ms ? _self.ms : ms // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class PlayerSkippedEvent implements PlayerEvent {
  const PlayerSkippedEvent(this.deltaMs);
  

 final  int deltaMs;

/// Create a copy of PlayerEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerSkippedEventCopyWith<PlayerSkippedEvent> get copyWith => _$PlayerSkippedEventCopyWithImpl<PlayerSkippedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerSkippedEvent&&(identical(other.deltaMs, deltaMs) || other.deltaMs == deltaMs));
}


@override
int get hashCode => Object.hash(runtimeType,deltaMs);

@override
String toString() {
  return 'PlayerEvent.skipped(deltaMs: $deltaMs)';
}


}

/// @nodoc
abstract mixin class $PlayerSkippedEventCopyWith<$Res> implements $PlayerEventCopyWith<$Res> {
  factory $PlayerSkippedEventCopyWith(PlayerSkippedEvent value, $Res Function(PlayerSkippedEvent) _then) = _$PlayerSkippedEventCopyWithImpl;
@useResult
$Res call({
 int deltaMs
});




}
/// @nodoc
class _$PlayerSkippedEventCopyWithImpl<$Res>
    implements $PlayerSkippedEventCopyWith<$Res> {
  _$PlayerSkippedEventCopyWithImpl(this._self, this._then);

  final PlayerSkippedEvent _self;
  final $Res Function(PlayerSkippedEvent) _then;

/// Create a copy of PlayerEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? deltaMs = null,}) {
  return _then(PlayerSkippedEvent(
null == deltaMs ? _self.deltaMs : deltaMs // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class PlayerSpeedCycledEvent implements PlayerEvent {
  const PlayerSpeedCycledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerSpeedCycledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlayerEvent.speedCycled()';
}


}




/// @nodoc


class PlayerVolumeChangedEvent implements PlayerEvent {
  const PlayerVolumeChangedEvent(this.volume);
  

 final  double volume;

/// Create a copy of PlayerEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerVolumeChangedEventCopyWith<PlayerVolumeChangedEvent> get copyWith => _$PlayerVolumeChangedEventCopyWithImpl<PlayerVolumeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerVolumeChangedEvent&&(identical(other.volume, volume) || other.volume == volume));
}


@override
int get hashCode => Object.hash(runtimeType,volume);

@override
String toString() {
  return 'PlayerEvent.volumeChanged(volume: $volume)';
}


}

/// @nodoc
abstract mixin class $PlayerVolumeChangedEventCopyWith<$Res> implements $PlayerEventCopyWith<$Res> {
  factory $PlayerVolumeChangedEventCopyWith(PlayerVolumeChangedEvent value, $Res Function(PlayerVolumeChangedEvent) _then) = _$PlayerVolumeChangedEventCopyWithImpl;
@useResult
$Res call({
 double volume
});




}
/// @nodoc
class _$PlayerVolumeChangedEventCopyWithImpl<$Res>
    implements $PlayerVolumeChangedEventCopyWith<$Res> {
  _$PlayerVolumeChangedEventCopyWithImpl(this._self, this._then);

  final PlayerVolumeChangedEvent _self;
  final $Res Function(PlayerVolumeChangedEvent) _then;

/// Create a copy of PlayerEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? volume = null,}) {
  return _then(PlayerVolumeChangedEvent(
null == volume ? _self.volume : volume // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class PlayerMuteToggledEvent implements PlayerEvent {
  const PlayerMuteToggledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerMuteToggledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlayerEvent.muteToggled()';
}


}




/// @nodoc


class PlayerMomentTappedEvent implements PlayerEvent {
  const PlayerMomentTappedEvent(this.fileId, this.seconds);
  

 final  String fileId;
 final  int seconds;

/// Create a copy of PlayerEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerMomentTappedEventCopyWith<PlayerMomentTappedEvent> get copyWith => _$PlayerMomentTappedEventCopyWithImpl<PlayerMomentTappedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerMomentTappedEvent&&(identical(other.fileId, fileId) || other.fileId == fileId)&&(identical(other.seconds, seconds) || other.seconds == seconds));
}


@override
int get hashCode => Object.hash(runtimeType,fileId,seconds);

@override
String toString() {
  return 'PlayerEvent.momentTapped(fileId: $fileId, seconds: $seconds)';
}


}

/// @nodoc
abstract mixin class $PlayerMomentTappedEventCopyWith<$Res> implements $PlayerEventCopyWith<$Res> {
  factory $PlayerMomentTappedEventCopyWith(PlayerMomentTappedEvent value, $Res Function(PlayerMomentTappedEvent) _then) = _$PlayerMomentTappedEventCopyWithImpl;
@useResult
$Res call({
 String fileId, int seconds
});




}
/// @nodoc
class _$PlayerMomentTappedEventCopyWithImpl<$Res>
    implements $PlayerMomentTappedEventCopyWith<$Res> {
  _$PlayerMomentTappedEventCopyWithImpl(this._self, this._then);

  final PlayerMomentTappedEvent _self;
  final $Res Function(PlayerMomentTappedEvent) _then;

/// Create a copy of PlayerEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? fileId = null,Object? seconds = null,}) {
  return _then(PlayerMomentTappedEvent(
null == fileId ? _self.fileId : fileId // ignore: cast_nullable_to_non_nullable
as String,null == seconds ? _self.seconds : seconds // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class PlayerRetriedEvent implements PlayerEvent {
  const PlayerRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlayerEvent.retried()';
}


}




/// @nodoc


class PlayerTickedEvent implements PlayerEvent {
  const PlayerTickedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerTickedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PlayerEvent.ticked()';
}


}




/// @nodoc
mixin _$PlayerState {

 PlayerPhase get phase; String get noteId;/// The parts of the recording, in playing order. The first is the file
/// every timestamp in the note names.
 List<String> get fileIds; int get positionMs; int get lengthMs; bool get playing; double get speed; double get volume;/// Counts the timestamp taps: each asks the card to scroll into view.
 int get reveals; String? get message;
/// Create a copy of PlayerState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PlayerStateCopyWith<PlayerState> get copyWith => _$PlayerStateCopyWithImpl<PlayerState>(this as PlayerState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PlayerState&&(identical(other.phase, phase) || other.phase == phase)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&const DeepCollectionEquality().equals(other.fileIds, fileIds)&&(identical(other.positionMs, positionMs) || other.positionMs == positionMs)&&(identical(other.lengthMs, lengthMs) || other.lengthMs == lengthMs)&&(identical(other.playing, playing) || other.playing == playing)&&(identical(other.speed, speed) || other.speed == speed)&&(identical(other.volume, volume) || other.volume == volume)&&(identical(other.reveals, reveals) || other.reveals == reveals)&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,phase,noteId,const DeepCollectionEquality().hash(fileIds),positionMs,lengthMs,playing,speed,volume,reveals,message);

@override
String toString() {
  return 'PlayerState(phase: $phase, noteId: $noteId, fileIds: $fileIds, positionMs: $positionMs, lengthMs: $lengthMs, playing: $playing, speed: $speed, volume: $volume, reveals: $reveals, message: $message)';
}


}

/// @nodoc
abstract mixin class $PlayerStateCopyWith<$Res>  {
  factory $PlayerStateCopyWith(PlayerState value, $Res Function(PlayerState) _then) = _$PlayerStateCopyWithImpl;
@useResult
$Res call({
 PlayerPhase phase, String noteId, List<String> fileIds, int positionMs, int lengthMs, bool playing, double speed, double volume, int reveals, String? message
});




}
/// @nodoc
class _$PlayerStateCopyWithImpl<$Res>
    implements $PlayerStateCopyWith<$Res> {
  _$PlayerStateCopyWithImpl(this._self, this._then);

  final PlayerState _self;
  final $Res Function(PlayerState) _then;

/// Create a copy of PlayerState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? phase = null,Object? noteId = null,Object? fileIds = null,Object? positionMs = null,Object? lengthMs = null,Object? playing = null,Object? speed = null,Object? volume = null,Object? reveals = null,Object? message = freezed,}) {
  return _then(_self.copyWith(
phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as PlayerPhase,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,fileIds: null == fileIds ? _self.fileIds : fileIds // ignore: cast_nullable_to_non_nullable
as List<String>,positionMs: null == positionMs ? _self.positionMs : positionMs // ignore: cast_nullable_to_non_nullable
as int,lengthMs: null == lengthMs ? _self.lengthMs : lengthMs // ignore: cast_nullable_to_non_nullable
as int,playing: null == playing ? _self.playing : playing // ignore: cast_nullable_to_non_nullable
as bool,speed: null == speed ? _self.speed : speed // ignore: cast_nullable_to_non_nullable
as double,volume: null == volume ? _self.volume : volume // ignore: cast_nullable_to_non_nullable
as double,reveals: null == reveals ? _self.reveals : reveals // ignore: cast_nullable_to_non_nullable
as int,message: freezed == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [PlayerState].
extension PlayerStatePatterns on PlayerState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _PlayerState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _PlayerState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _PlayerState value)  $default,){
final _that = this;
switch (_that) {
case _PlayerState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _PlayerState value)?  $default,){
final _that = this;
switch (_that) {
case _PlayerState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( PlayerPhase phase,  String noteId,  List<String> fileIds,  int positionMs,  int lengthMs,  bool playing,  double speed,  double volume,  int reveals,  String? message)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _PlayerState() when $default != null:
return $default(_that.phase,_that.noteId,_that.fileIds,_that.positionMs,_that.lengthMs,_that.playing,_that.speed,_that.volume,_that.reveals,_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( PlayerPhase phase,  String noteId,  List<String> fileIds,  int positionMs,  int lengthMs,  bool playing,  double speed,  double volume,  int reveals,  String? message)  $default,) {final _that = this;
switch (_that) {
case _PlayerState():
return $default(_that.phase,_that.noteId,_that.fileIds,_that.positionMs,_that.lengthMs,_that.playing,_that.speed,_that.volume,_that.reveals,_that.message);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( PlayerPhase phase,  String noteId,  List<String> fileIds,  int positionMs,  int lengthMs,  bool playing,  double speed,  double volume,  int reveals,  String? message)?  $default,) {final _that = this;
switch (_that) {
case _PlayerState() when $default != null:
return $default(_that.phase,_that.noteId,_that.fileIds,_that.positionMs,_that.lengthMs,_that.playing,_that.speed,_that.volume,_that.reveals,_that.message);case _:
  return null;

}
}

}

/// @nodoc


class _PlayerState extends PlayerState {
  const _PlayerState({this.phase = PlayerPhase.none, this.noteId = '', final  List<String> fileIds = const <String>[], this.positionMs = 0, this.lengthMs = 0, this.playing = false, this.speed = 1.0, this.volume = 1.0, this.reveals = 0, this.message}): _fileIds = fileIds,super._();
  

@override@JsonKey() final  PlayerPhase phase;
@override@JsonKey() final  String noteId;
/// The parts of the recording, in playing order. The first is the file
/// every timestamp in the note names.
 final  List<String> _fileIds;
/// The parts of the recording, in playing order. The first is the file
/// every timestamp in the note names.
@override@JsonKey() List<String> get fileIds {
  if (_fileIds is EqualUnmodifiableListView) return _fileIds;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_fileIds);
}

@override@JsonKey() final  int positionMs;
@override@JsonKey() final  int lengthMs;
@override@JsonKey() final  bool playing;
@override@JsonKey() final  double speed;
@override@JsonKey() final  double volume;
/// Counts the timestamp taps: each asks the card to scroll into view.
@override@JsonKey() final  int reveals;
@override final  String? message;

/// Create a copy of PlayerState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$PlayerStateCopyWith<_PlayerState> get copyWith => __$PlayerStateCopyWithImpl<_PlayerState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _PlayerState&&(identical(other.phase, phase) || other.phase == phase)&&(identical(other.noteId, noteId) || other.noteId == noteId)&&const DeepCollectionEquality().equals(other._fileIds, _fileIds)&&(identical(other.positionMs, positionMs) || other.positionMs == positionMs)&&(identical(other.lengthMs, lengthMs) || other.lengthMs == lengthMs)&&(identical(other.playing, playing) || other.playing == playing)&&(identical(other.speed, speed) || other.speed == speed)&&(identical(other.volume, volume) || other.volume == volume)&&(identical(other.reveals, reveals) || other.reveals == reveals)&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,phase,noteId,const DeepCollectionEquality().hash(_fileIds),positionMs,lengthMs,playing,speed,volume,reveals,message);

@override
String toString() {
  return 'PlayerState(phase: $phase, noteId: $noteId, fileIds: $fileIds, positionMs: $positionMs, lengthMs: $lengthMs, playing: $playing, speed: $speed, volume: $volume, reveals: $reveals, message: $message)';
}


}

/// @nodoc
abstract mixin class _$PlayerStateCopyWith<$Res> implements $PlayerStateCopyWith<$Res> {
  factory _$PlayerStateCopyWith(_PlayerState value, $Res Function(_PlayerState) _then) = __$PlayerStateCopyWithImpl;
@override @useResult
$Res call({
 PlayerPhase phase, String noteId, List<String> fileIds, int positionMs, int lengthMs, bool playing, double speed, double volume, int reveals, String? message
});




}
/// @nodoc
class __$PlayerStateCopyWithImpl<$Res>
    implements _$PlayerStateCopyWith<$Res> {
  __$PlayerStateCopyWithImpl(this._self, this._then);

  final _PlayerState _self;
  final $Res Function(_PlayerState) _then;

/// Create a copy of PlayerState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? phase = null,Object? noteId = null,Object? fileIds = null,Object? positionMs = null,Object? lengthMs = null,Object? playing = null,Object? speed = null,Object? volume = null,Object? reveals = null,Object? message = freezed,}) {
  return _then(_PlayerState(
phase: null == phase ? _self.phase : phase // ignore: cast_nullable_to_non_nullable
as PlayerPhase,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,fileIds: null == fileIds ? _self._fileIds : fileIds // ignore: cast_nullable_to_non_nullable
as List<String>,positionMs: null == positionMs ? _self.positionMs : positionMs // ignore: cast_nullable_to_non_nullable
as int,lengthMs: null == lengthMs ? _self.lengthMs : lengthMs // ignore: cast_nullable_to_non_nullable
as int,playing: null == playing ? _self.playing : playing // ignore: cast_nullable_to_non_nullable
as bool,speed: null == speed ? _self.speed : speed // ignore: cast_nullable_to_non_nullable
as double,volume: null == volume ? _self.volume : volume // ignore: cast_nullable_to_non_nullable
as double,reveals: null == reveals ? _self.reveals : reveals // ignore: cast_nullable_to_non_nullable
as int,message: freezed == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on

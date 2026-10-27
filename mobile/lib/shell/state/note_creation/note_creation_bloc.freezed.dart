// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'note_creation_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$NoteCreationEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteCreationEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteCreationEvent()';
}


}

/// @nodoc
class $NoteCreationEventCopyWith<$Res>  {
$NoteCreationEventCopyWith(NoteCreationEvent _, $Res Function(NoteCreationEvent) __);
}


/// Adds pattern-matching-related methods to [NoteCreationEvent].
extension NoteCreationEventPatterns on NoteCreationEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteCreationRequestedEvent value)?  requested,TResult Function( NoteCreationCapturedEvent value)?  captured,TResult Function( NoteCreationTitledEvent value)?  titled,TResult Function( NoteCreationOpeningHandledEvent value)?  openingHandled,TResult Function( NoteCreationNoticeDismissedEvent value)?  noticeDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteCreationRequestedEvent() when requested != null:
return requested(_that);case NoteCreationCapturedEvent() when captured != null:
return captured(_that);case NoteCreationTitledEvent() when titled != null:
return titled(_that);case NoteCreationOpeningHandledEvent() when openingHandled != null:
return openingHandled(_that);case NoteCreationNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteCreationRequestedEvent value)  requested,required TResult Function( NoteCreationCapturedEvent value)  captured,required TResult Function( NoteCreationTitledEvent value)  titled,required TResult Function( NoteCreationOpeningHandledEvent value)  openingHandled,required TResult Function( NoteCreationNoticeDismissedEvent value)  noticeDismissed,}){
final _that = this;
switch (_that) {
case NoteCreationRequestedEvent():
return requested(_that);case NoteCreationCapturedEvent():
return captured(_that);case NoteCreationTitledEvent():
return titled(_that);case NoteCreationOpeningHandledEvent():
return openingHandled(_that);case NoteCreationNoticeDismissedEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteCreationRequestedEvent value)?  requested,TResult? Function( NoteCreationCapturedEvent value)?  captured,TResult? Function( NoteCreationTitledEvent value)?  titled,TResult? Function( NoteCreationOpeningHandledEvent value)?  openingHandled,TResult? Function( NoteCreationNoticeDismissedEvent value)?  noticeDismissed,}){
final _that = this;
switch (_that) {
case NoteCreationRequestedEvent() when requested != null:
return requested(_that);case NoteCreationCapturedEvent() when captured != null:
return captured(_that);case NoteCreationTitledEvent() when titled != null:
return titled(_that);case NoteCreationOpeningHandledEvent() when openingHandled != null:
return openingHandled(_that);case NoteCreationNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( ListPlace place,  bool pinned,  NewNoteStart start)?  requested,TResult Function( MediaSource source)?  captured,TResult Function( String title)?  titled,TResult Function()?  openingHandled,TResult Function()?  noticeDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteCreationRequestedEvent() when requested != null:
return requested(_that.place,_that.pinned,_that.start);case NoteCreationCapturedEvent() when captured != null:
return captured(_that.source);case NoteCreationTitledEvent() when titled != null:
return titled(_that.title);case NoteCreationOpeningHandledEvent() when openingHandled != null:
return openingHandled();case NoteCreationNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( ListPlace place,  bool pinned,  NewNoteStart start)  requested,required TResult Function( MediaSource source)  captured,required TResult Function( String title)  titled,required TResult Function()  openingHandled,required TResult Function()  noticeDismissed,}) {final _that = this;
switch (_that) {
case NoteCreationRequestedEvent():
return requested(_that.place,_that.pinned,_that.start);case NoteCreationCapturedEvent():
return captured(_that.source);case NoteCreationTitledEvent():
return titled(_that.title);case NoteCreationOpeningHandledEvent():
return openingHandled();case NoteCreationNoticeDismissedEvent():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( ListPlace place,  bool pinned,  NewNoteStart start)?  requested,TResult? Function( MediaSource source)?  captured,TResult? Function( String title)?  titled,TResult? Function()?  openingHandled,TResult? Function()?  noticeDismissed,}) {final _that = this;
switch (_that) {
case NoteCreationRequestedEvent() when requested != null:
return requested(_that.place,_that.pinned,_that.start);case NoteCreationCapturedEvent() when captured != null:
return captured(_that.source);case NoteCreationTitledEvent() when titled != null:
return titled(_that.title);case NoteCreationOpeningHandledEvent() when openingHandled != null:
return openingHandled();case NoteCreationNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case _:
  return null;

}
}

}

/// @nodoc


class NoteCreationRequestedEvent implements NoteCreationEvent {
  const NoteCreationRequestedEvent(this.place, {this.pinned = false, this.start = NewNoteStart.title});
  

 final  ListPlace place;
@JsonKey() final  bool pinned;
@JsonKey() final  NewNoteStart start;

/// Create a copy of NoteCreationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteCreationRequestedEventCopyWith<NoteCreationRequestedEvent> get copyWith => _$NoteCreationRequestedEventCopyWithImpl<NoteCreationRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteCreationRequestedEvent&&(identical(other.place, place) || other.place == place)&&(identical(other.pinned, pinned) || other.pinned == pinned)&&(identical(other.start, start) || other.start == start));
}


@override
int get hashCode => Object.hash(runtimeType,place,pinned,start);

@override
String toString() {
  return 'NoteCreationEvent.requested(place: $place, pinned: $pinned, start: $start)';
}


}

/// @nodoc
abstract mixin class $NoteCreationRequestedEventCopyWith<$Res> implements $NoteCreationEventCopyWith<$Res> {
  factory $NoteCreationRequestedEventCopyWith(NoteCreationRequestedEvent value, $Res Function(NoteCreationRequestedEvent) _then) = _$NoteCreationRequestedEventCopyWithImpl;
@useResult
$Res call({
 ListPlace place, bool pinned, NewNoteStart start
});


$ListPlaceCopyWith<$Res> get place;

}
/// @nodoc
class _$NoteCreationRequestedEventCopyWithImpl<$Res>
    implements $NoteCreationRequestedEventCopyWith<$Res> {
  _$NoteCreationRequestedEventCopyWithImpl(this._self, this._then);

  final NoteCreationRequestedEvent _self;
  final $Res Function(NoteCreationRequestedEvent) _then;

/// Create a copy of NoteCreationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? place = null,Object? pinned = null,Object? start = null,}) {
  return _then(NoteCreationRequestedEvent(
null == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as ListPlace,pinned: null == pinned ? _self.pinned : pinned // ignore: cast_nullable_to_non_nullable
as bool,start: null == start ? _self.start : start // ignore: cast_nullable_to_non_nullable
as NewNoteStart,
  ));
}

/// Create a copy of NoteCreationEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ListPlaceCopyWith<$Res> get place {
  
  return $ListPlaceCopyWith<$Res>(_self.place, (value) {
    return _then(_self.copyWith(place: value));
  });
}
}

/// @nodoc


class NoteCreationCapturedEvent implements NoteCreationEvent {
  const NoteCreationCapturedEvent(this.source);
  

 final  MediaSource source;

/// Create a copy of NoteCreationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteCreationCapturedEventCopyWith<NoteCreationCapturedEvent> get copyWith => _$NoteCreationCapturedEventCopyWithImpl<NoteCreationCapturedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteCreationCapturedEvent&&(identical(other.source, source) || other.source == source));
}


@override
int get hashCode => Object.hash(runtimeType,source);

@override
String toString() {
  return 'NoteCreationEvent.captured(source: $source)';
}


}

/// @nodoc
abstract mixin class $NoteCreationCapturedEventCopyWith<$Res> implements $NoteCreationEventCopyWith<$Res> {
  factory $NoteCreationCapturedEventCopyWith(NoteCreationCapturedEvent value, $Res Function(NoteCreationCapturedEvent) _then) = _$NoteCreationCapturedEventCopyWithImpl;
@useResult
$Res call({
 MediaSource source
});




}
/// @nodoc
class _$NoteCreationCapturedEventCopyWithImpl<$Res>
    implements $NoteCreationCapturedEventCopyWith<$Res> {
  _$NoteCreationCapturedEventCopyWithImpl(this._self, this._then);

  final NoteCreationCapturedEvent _self;
  final $Res Function(NoteCreationCapturedEvent) _then;

/// Create a copy of NoteCreationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? source = null,}) {
  return _then(NoteCreationCapturedEvent(
null == source ? _self.source : source // ignore: cast_nullable_to_non_nullable
as MediaSource,
  ));
}


}

/// @nodoc


class NoteCreationTitledEvent implements NoteCreationEvent {
  const NoteCreationTitledEvent(this.title);
  

 final  String title;

/// Create a copy of NoteCreationEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteCreationTitledEventCopyWith<NoteCreationTitledEvent> get copyWith => _$NoteCreationTitledEventCopyWithImpl<NoteCreationTitledEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteCreationTitledEvent&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,title);

@override
String toString() {
  return 'NoteCreationEvent.titled(title: $title)';
}


}

/// @nodoc
abstract mixin class $NoteCreationTitledEventCopyWith<$Res> implements $NoteCreationEventCopyWith<$Res> {
  factory $NoteCreationTitledEventCopyWith(NoteCreationTitledEvent value, $Res Function(NoteCreationTitledEvent) _then) = _$NoteCreationTitledEventCopyWithImpl;
@useResult
$Res call({
 String title
});




}
/// @nodoc
class _$NoteCreationTitledEventCopyWithImpl<$Res>
    implements $NoteCreationTitledEventCopyWith<$Res> {
  _$NoteCreationTitledEventCopyWithImpl(this._self, this._then);

  final NoteCreationTitledEvent _self;
  final $Res Function(NoteCreationTitledEvent) _then;

/// Create a copy of NoteCreationEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? title = null,}) {
  return _then(NoteCreationTitledEvent(
null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteCreationOpeningHandledEvent implements NoteCreationEvent {
  const NoteCreationOpeningHandledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteCreationOpeningHandledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteCreationEvent.openingHandled()';
}


}




/// @nodoc


class NoteCreationNoticeDismissedEvent implements NoteCreationEvent {
  const NoteCreationNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteCreationNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteCreationEvent.noticeDismissed()';
}


}




/// @nodoc
mixin _$NoteCreationState {

/// Whether a note is being made.
 bool get creating;/// The note just made, for the screen to go to; it says so with
/// `openingHandled` when it has.
 NoteOpening? get opening;/// Why the last note was not made, until it is dismissed.
 NoteCreationFailure? get failure;/// What was left out of the note just captured, a sentence each, until
/// it is dismissed.
 List<String> get problems;
/// Create a copy of NoteCreationState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteCreationStateCopyWith<NoteCreationState> get copyWith => _$NoteCreationStateCopyWithImpl<NoteCreationState>(this as NoteCreationState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteCreationState&&(identical(other.creating, creating) || other.creating == creating)&&(identical(other.opening, opening) || other.opening == opening)&&(identical(other.failure, failure) || other.failure == failure)&&const DeepCollectionEquality().equals(other.problems, problems));
}


@override
int get hashCode => Object.hash(runtimeType,creating,opening,failure,const DeepCollectionEquality().hash(problems));

@override
String toString() {
  return 'NoteCreationState(creating: $creating, opening: $opening, failure: $failure, problems: $problems)';
}


}

/// @nodoc
abstract mixin class $NoteCreationStateCopyWith<$Res>  {
  factory $NoteCreationStateCopyWith(NoteCreationState value, $Res Function(NoteCreationState) _then) = _$NoteCreationStateCopyWithImpl;
@useResult
$Res call({
 bool creating, NoteOpening? opening, NoteCreationFailure? failure, List<String> problems
});


$NoteOpeningCopyWith<$Res>? get opening;

}
/// @nodoc
class _$NoteCreationStateCopyWithImpl<$Res>
    implements $NoteCreationStateCopyWith<$Res> {
  _$NoteCreationStateCopyWithImpl(this._self, this._then);

  final NoteCreationState _self;
  final $Res Function(NoteCreationState) _then;

/// Create a copy of NoteCreationState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? creating = null,Object? opening = freezed,Object? failure = freezed,Object? problems = null,}) {
  return _then(_self.copyWith(
creating: null == creating ? _self.creating : creating // ignore: cast_nullable_to_non_nullable
as bool,opening: freezed == opening ? _self.opening : opening // ignore: cast_nullable_to_non_nullable
as NoteOpening?,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as NoteCreationFailure?,problems: null == problems ? _self.problems : problems // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}
/// Create a copy of NoteCreationState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$NoteOpeningCopyWith<$Res>? get opening {
    if (_self.opening == null) {
    return null;
  }

  return $NoteOpeningCopyWith<$Res>(_self.opening!, (value) {
    return _then(_self.copyWith(opening: value));
  });
}
}


/// Adds pattern-matching-related methods to [NoteCreationState].
extension NoteCreationStatePatterns on NoteCreationState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _NoteCreationState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _NoteCreationState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _NoteCreationState value)  $default,){
final _that = this;
switch (_that) {
case _NoteCreationState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _NoteCreationState value)?  $default,){
final _that = this;
switch (_that) {
case _NoteCreationState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool creating,  NoteOpening? opening,  NoteCreationFailure? failure,  List<String> problems)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _NoteCreationState() when $default != null:
return $default(_that.creating,_that.opening,_that.failure,_that.problems);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool creating,  NoteOpening? opening,  NoteCreationFailure? failure,  List<String> problems)  $default,) {final _that = this;
switch (_that) {
case _NoteCreationState():
return $default(_that.creating,_that.opening,_that.failure,_that.problems);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool creating,  NoteOpening? opening,  NoteCreationFailure? failure,  List<String> problems)?  $default,) {final _that = this;
switch (_that) {
case _NoteCreationState() when $default != null:
return $default(_that.creating,_that.opening,_that.failure,_that.problems);case _:
  return null;

}
}

}

/// @nodoc


class _NoteCreationState implements NoteCreationState {
  const _NoteCreationState({this.creating = false, this.opening, this.failure, final  List<String> problems = const <String>[]}): _problems = problems;
  

/// Whether a note is being made.
@override@JsonKey() final  bool creating;
/// The note just made, for the screen to go to; it says so with
/// `openingHandled` when it has.
@override final  NoteOpening? opening;
/// Why the last note was not made, until it is dismissed.
@override final  NoteCreationFailure? failure;
/// What was left out of the note just captured, a sentence each, until
/// it is dismissed.
 final  List<String> _problems;
/// What was left out of the note just captured, a sentence each, until
/// it is dismissed.
@override@JsonKey() List<String> get problems {
  if (_problems is EqualUnmodifiableListView) return _problems;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_problems);
}


/// Create a copy of NoteCreationState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$NoteCreationStateCopyWith<_NoteCreationState> get copyWith => __$NoteCreationStateCopyWithImpl<_NoteCreationState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _NoteCreationState&&(identical(other.creating, creating) || other.creating == creating)&&(identical(other.opening, opening) || other.opening == opening)&&(identical(other.failure, failure) || other.failure == failure)&&const DeepCollectionEquality().equals(other._problems, _problems));
}


@override
int get hashCode => Object.hash(runtimeType,creating,opening,failure,const DeepCollectionEquality().hash(_problems));

@override
String toString() {
  return 'NoteCreationState(creating: $creating, opening: $opening, failure: $failure, problems: $problems)';
}


}

/// @nodoc
abstract mixin class _$NoteCreationStateCopyWith<$Res> implements $NoteCreationStateCopyWith<$Res> {
  factory _$NoteCreationStateCopyWith(_NoteCreationState value, $Res Function(_NoteCreationState) _then) = __$NoteCreationStateCopyWithImpl;
@override @useResult
$Res call({
 bool creating, NoteOpening? opening, NoteCreationFailure? failure, List<String> problems
});


@override $NoteOpeningCopyWith<$Res>? get opening;

}
/// @nodoc
class __$NoteCreationStateCopyWithImpl<$Res>
    implements _$NoteCreationStateCopyWith<$Res> {
  __$NoteCreationStateCopyWithImpl(this._self, this._then);

  final _NoteCreationState _self;
  final $Res Function(_NoteCreationState) _then;

/// Create a copy of NoteCreationState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? creating = null,Object? opening = freezed,Object? failure = freezed,Object? problems = null,}) {
  return _then(_NoteCreationState(
creating: null == creating ? _self.creating : creating // ignore: cast_nullable_to_non_nullable
as bool,opening: freezed == opening ? _self.opening : opening // ignore: cast_nullable_to_non_nullable
as NoteOpening?,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as NoteCreationFailure?,problems: null == problems ? _self._problems : problems // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}

/// Create a copy of NoteCreationState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$NoteOpeningCopyWith<$Res>? get opening {
    if (_self.opening == null) {
    return null;
  }

  return $NoteOpeningCopyWith<$Res>(_self.opening!, (value) {
    return _then(_self.copyWith(opening: value));
  });
}
}

// dart format on

// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'board_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$BoardEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BoardEvent()';
}


}

/// @nodoc
class $BoardEventCopyWith<$Res>  {
$BoardEventCopyWith(BoardEvent _, $Res Function(BoardEvent) __);
}


/// Adds pattern-matching-related methods to [BoardEvent].
extension BoardEventPatterns on BoardEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( BoardStartedEvent value)?  started,TResult Function( BoardKindChangedEvent value)?  kindChanged,TResult Function( BoardChangesReceivedEvent value)?  changesReceived,TResult Function( BoardRetriedEvent value)?  retried,TResult Function( BoardCardMovedEvent value)?  cardMoved,TResult Function( BoardNoticeDismissedEvent value)?  noticeDismissed,TResult Function( BoardGroupChangedEvent value)?  groupChanged,TResult Function( BoardShowUnsetChangedEvent value)?  showUnsetChanged,TResult Function( BoardLaneEditedEvent value)?  laneEdited,TResult Function( BoardLaneRemovalRequestedEvent value)?  laneRemovalRequested,TResult Function( BoardRemovalTargetChosenEvent value)?  removalTargetChosen,TResult Function( BoardRemovalConfirmedEvent value)?  removalConfirmed,TResult Function( BoardRemovalCancelledEvent value)?  removalCancelled,TResult Function( BoardUndoneEvent value)?  undone,TResult Function( BoardSettingsClosedEvent value)?  settingsClosed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case BoardStartedEvent() when started != null:
return started(_that);case BoardKindChangedEvent() when kindChanged != null:
return kindChanged(_that);case BoardChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case BoardRetriedEvent() when retried != null:
return retried(_that);case BoardCardMovedEvent() when cardMoved != null:
return cardMoved(_that);case BoardNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed(_that);case BoardGroupChangedEvent() when groupChanged != null:
return groupChanged(_that);case BoardShowUnsetChangedEvent() when showUnsetChanged != null:
return showUnsetChanged(_that);case BoardLaneEditedEvent() when laneEdited != null:
return laneEdited(_that);case BoardLaneRemovalRequestedEvent() when laneRemovalRequested != null:
return laneRemovalRequested(_that);case BoardRemovalTargetChosenEvent() when removalTargetChosen != null:
return removalTargetChosen(_that);case BoardRemovalConfirmedEvent() when removalConfirmed != null:
return removalConfirmed(_that);case BoardRemovalCancelledEvent() when removalCancelled != null:
return removalCancelled(_that);case BoardUndoneEvent() when undone != null:
return undone(_that);case BoardSettingsClosedEvent() when settingsClosed != null:
return settingsClosed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( BoardStartedEvent value)  started,required TResult Function( BoardKindChangedEvent value)  kindChanged,required TResult Function( BoardChangesReceivedEvent value)  changesReceived,required TResult Function( BoardRetriedEvent value)  retried,required TResult Function( BoardCardMovedEvent value)  cardMoved,required TResult Function( BoardNoticeDismissedEvent value)  noticeDismissed,required TResult Function( BoardGroupChangedEvent value)  groupChanged,required TResult Function( BoardShowUnsetChangedEvent value)  showUnsetChanged,required TResult Function( BoardLaneEditedEvent value)  laneEdited,required TResult Function( BoardLaneRemovalRequestedEvent value)  laneRemovalRequested,required TResult Function( BoardRemovalTargetChosenEvent value)  removalTargetChosen,required TResult Function( BoardRemovalConfirmedEvent value)  removalConfirmed,required TResult Function( BoardRemovalCancelledEvent value)  removalCancelled,required TResult Function( BoardUndoneEvent value)  undone,required TResult Function( BoardSettingsClosedEvent value)  settingsClosed,}){
final _that = this;
switch (_that) {
case BoardStartedEvent():
return started(_that);case BoardKindChangedEvent():
return kindChanged(_that);case BoardChangesReceivedEvent():
return changesReceived(_that);case BoardRetriedEvent():
return retried(_that);case BoardCardMovedEvent():
return cardMoved(_that);case BoardNoticeDismissedEvent():
return noticeDismissed(_that);case BoardGroupChangedEvent():
return groupChanged(_that);case BoardShowUnsetChangedEvent():
return showUnsetChanged(_that);case BoardLaneEditedEvent():
return laneEdited(_that);case BoardLaneRemovalRequestedEvent():
return laneRemovalRequested(_that);case BoardRemovalTargetChosenEvent():
return removalTargetChosen(_that);case BoardRemovalConfirmedEvent():
return removalConfirmed(_that);case BoardRemovalCancelledEvent():
return removalCancelled(_that);case BoardUndoneEvent():
return undone(_that);case BoardSettingsClosedEvent():
return settingsClosed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( BoardStartedEvent value)?  started,TResult? Function( BoardKindChangedEvent value)?  kindChanged,TResult? Function( BoardChangesReceivedEvent value)?  changesReceived,TResult? Function( BoardRetriedEvent value)?  retried,TResult? Function( BoardCardMovedEvent value)?  cardMoved,TResult? Function( BoardNoticeDismissedEvent value)?  noticeDismissed,TResult? Function( BoardGroupChangedEvent value)?  groupChanged,TResult? Function( BoardShowUnsetChangedEvent value)?  showUnsetChanged,TResult? Function( BoardLaneEditedEvent value)?  laneEdited,TResult? Function( BoardLaneRemovalRequestedEvent value)?  laneRemovalRequested,TResult? Function( BoardRemovalTargetChosenEvent value)?  removalTargetChosen,TResult? Function( BoardRemovalConfirmedEvent value)?  removalConfirmed,TResult? Function( BoardRemovalCancelledEvent value)?  removalCancelled,TResult? Function( BoardUndoneEvent value)?  undone,TResult? Function( BoardSettingsClosedEvent value)?  settingsClosed,}){
final _that = this;
switch (_that) {
case BoardStartedEvent() when started != null:
return started(_that);case BoardKindChangedEvent() when kindChanged != null:
return kindChanged(_that);case BoardChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case BoardRetriedEvent() when retried != null:
return retried(_that);case BoardCardMovedEvent() when cardMoved != null:
return cardMoved(_that);case BoardNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed(_that);case BoardGroupChangedEvent() when groupChanged != null:
return groupChanged(_that);case BoardShowUnsetChangedEvent() when showUnsetChanged != null:
return showUnsetChanged(_that);case BoardLaneEditedEvent() when laneEdited != null:
return laneEdited(_that);case BoardLaneRemovalRequestedEvent() when laneRemovalRequested != null:
return laneRemovalRequested(_that);case BoardRemovalTargetChosenEvent() when removalTargetChosen != null:
return removalTargetChosen(_that);case BoardRemovalConfirmedEvent() when removalConfirmed != null:
return removalConfirmed(_that);case BoardRemovalCancelledEvent() when removalCancelled != null:
return removalCancelled(_that);case BoardUndoneEvent() when undone != null:
return undone(_that);case BoardSettingsClosedEvent() when settingsClosed != null:
return settingsClosed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( ListQuery query,  BoardKind kind)?  started,TResult Function( BoardKind kind)?  kindChanged,TResult Function( List<String> ids)?  changesReceived,TResult Function()?  retried,TResult Function( CardKey card,  BoardLane lane)?  cardMoved,TResult Function()?  noticeDismissed,TResult Function( String? by)?  groupChanged,TResult Function( bool show)?  showUnsetChanged,TResult Function( LaneChange change)?  laneEdited,TResult Function( String lane)?  laneRemovalRequested,TResult Function( String? target)?  removalTargetChosen,TResult Function()?  removalConfirmed,TResult Function()?  removalCancelled,TResult Function( int ticket)?  undone,TResult Function()?  settingsClosed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case BoardStartedEvent() when started != null:
return started(_that.query,_that.kind);case BoardKindChangedEvent() when kindChanged != null:
return kindChanged(_that.kind);case BoardChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case BoardRetriedEvent() when retried != null:
return retried();case BoardCardMovedEvent() when cardMoved != null:
return cardMoved(_that.card,_that.lane);case BoardNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case BoardGroupChangedEvent() when groupChanged != null:
return groupChanged(_that.by);case BoardShowUnsetChangedEvent() when showUnsetChanged != null:
return showUnsetChanged(_that.show);case BoardLaneEditedEvent() when laneEdited != null:
return laneEdited(_that.change);case BoardLaneRemovalRequestedEvent() when laneRemovalRequested != null:
return laneRemovalRequested(_that.lane);case BoardRemovalTargetChosenEvent() when removalTargetChosen != null:
return removalTargetChosen(_that.target);case BoardRemovalConfirmedEvent() when removalConfirmed != null:
return removalConfirmed();case BoardRemovalCancelledEvent() when removalCancelled != null:
return removalCancelled();case BoardUndoneEvent() when undone != null:
return undone(_that.ticket);case BoardSettingsClosedEvent() when settingsClosed != null:
return settingsClosed();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( ListQuery query,  BoardKind kind)  started,required TResult Function( BoardKind kind)  kindChanged,required TResult Function( List<String> ids)  changesReceived,required TResult Function()  retried,required TResult Function( CardKey card,  BoardLane lane)  cardMoved,required TResult Function()  noticeDismissed,required TResult Function( String? by)  groupChanged,required TResult Function( bool show)  showUnsetChanged,required TResult Function( LaneChange change)  laneEdited,required TResult Function( String lane)  laneRemovalRequested,required TResult Function( String? target)  removalTargetChosen,required TResult Function()  removalConfirmed,required TResult Function()  removalCancelled,required TResult Function( int ticket)  undone,required TResult Function()  settingsClosed,}) {final _that = this;
switch (_that) {
case BoardStartedEvent():
return started(_that.query,_that.kind);case BoardKindChangedEvent():
return kindChanged(_that.kind);case BoardChangesReceivedEvent():
return changesReceived(_that.ids);case BoardRetriedEvent():
return retried();case BoardCardMovedEvent():
return cardMoved(_that.card,_that.lane);case BoardNoticeDismissedEvent():
return noticeDismissed();case BoardGroupChangedEvent():
return groupChanged(_that.by);case BoardShowUnsetChangedEvent():
return showUnsetChanged(_that.show);case BoardLaneEditedEvent():
return laneEdited(_that.change);case BoardLaneRemovalRequestedEvent():
return laneRemovalRequested(_that.lane);case BoardRemovalTargetChosenEvent():
return removalTargetChosen(_that.target);case BoardRemovalConfirmedEvent():
return removalConfirmed();case BoardRemovalCancelledEvent():
return removalCancelled();case BoardUndoneEvent():
return undone(_that.ticket);case BoardSettingsClosedEvent():
return settingsClosed();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( ListQuery query,  BoardKind kind)?  started,TResult? Function( BoardKind kind)?  kindChanged,TResult? Function( List<String> ids)?  changesReceived,TResult? Function()?  retried,TResult? Function( CardKey card,  BoardLane lane)?  cardMoved,TResult? Function()?  noticeDismissed,TResult? Function( String? by)?  groupChanged,TResult? Function( bool show)?  showUnsetChanged,TResult? Function( LaneChange change)?  laneEdited,TResult? Function( String lane)?  laneRemovalRequested,TResult? Function( String? target)?  removalTargetChosen,TResult? Function()?  removalConfirmed,TResult? Function()?  removalCancelled,TResult? Function( int ticket)?  undone,TResult? Function()?  settingsClosed,}) {final _that = this;
switch (_that) {
case BoardStartedEvent() when started != null:
return started(_that.query,_that.kind);case BoardKindChangedEvent() when kindChanged != null:
return kindChanged(_that.kind);case BoardChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case BoardRetriedEvent() when retried != null:
return retried();case BoardCardMovedEvent() when cardMoved != null:
return cardMoved(_that.card,_that.lane);case BoardNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case BoardGroupChangedEvent() when groupChanged != null:
return groupChanged(_that.by);case BoardShowUnsetChangedEvent() when showUnsetChanged != null:
return showUnsetChanged(_that.show);case BoardLaneEditedEvent() when laneEdited != null:
return laneEdited(_that.change);case BoardLaneRemovalRequestedEvent() when laneRemovalRequested != null:
return laneRemovalRequested(_that.lane);case BoardRemovalTargetChosenEvent() when removalTargetChosen != null:
return removalTargetChosen(_that.target);case BoardRemovalConfirmedEvent() when removalConfirmed != null:
return removalConfirmed();case BoardRemovalCancelledEvent() when removalCancelled != null:
return removalCancelled();case BoardUndoneEvent() when undone != null:
return undone(_that.ticket);case BoardSettingsClosedEvent() when settingsClosed != null:
return settingsClosed();case _:
  return null;

}
}

}

/// @nodoc


class BoardStartedEvent implements BoardEvent {
  const BoardStartedEvent({required this.query, this.kind = BoardKind.notes});
  

 final  ListQuery query;
@JsonKey() final  BoardKind kind;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardStartedEventCopyWith<BoardStartedEvent> get copyWith => _$BoardStartedEventCopyWithImpl<BoardStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardStartedEvent&&(identical(other.query, query) || other.query == query)&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,query,kind);

@override
String toString() {
  return 'BoardEvent.started(query: $query, kind: $kind)';
}


}

/// @nodoc
abstract mixin class $BoardStartedEventCopyWith<$Res> implements $BoardEventCopyWith<$Res> {
  factory $BoardStartedEventCopyWith(BoardStartedEvent value, $Res Function(BoardStartedEvent) _then) = _$BoardStartedEventCopyWithImpl;
@useResult
$Res call({
 ListQuery query, BoardKind kind
});




}
/// @nodoc
class _$BoardStartedEventCopyWithImpl<$Res>
    implements $BoardStartedEventCopyWith<$Res> {
  _$BoardStartedEventCopyWithImpl(this._self, this._then);

  final BoardStartedEvent _self;
  final $Res Function(BoardStartedEvent) _then;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? query = null,Object? kind = null,}) {
  return _then(BoardStartedEvent(
query: null == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as ListQuery,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as BoardKind,
  ));
}


}

/// @nodoc


class BoardKindChangedEvent implements BoardEvent {
  const BoardKindChangedEvent(this.kind);
  

 final  BoardKind kind;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardKindChangedEventCopyWith<BoardKindChangedEvent> get copyWith => _$BoardKindChangedEventCopyWithImpl<BoardKindChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardKindChangedEvent&&(identical(other.kind, kind) || other.kind == kind));
}


@override
int get hashCode => Object.hash(runtimeType,kind);

@override
String toString() {
  return 'BoardEvent.kindChanged(kind: $kind)';
}


}

/// @nodoc
abstract mixin class $BoardKindChangedEventCopyWith<$Res> implements $BoardEventCopyWith<$Res> {
  factory $BoardKindChangedEventCopyWith(BoardKindChangedEvent value, $Res Function(BoardKindChangedEvent) _then) = _$BoardKindChangedEventCopyWithImpl;
@useResult
$Res call({
 BoardKind kind
});




}
/// @nodoc
class _$BoardKindChangedEventCopyWithImpl<$Res>
    implements $BoardKindChangedEventCopyWith<$Res> {
  _$BoardKindChangedEventCopyWithImpl(this._self, this._then);

  final BoardKindChangedEvent _self;
  final $Res Function(BoardKindChangedEvent) _then;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? kind = null,}) {
  return _then(BoardKindChangedEvent(
null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as BoardKind,
  ));
}


}

/// @nodoc


class BoardChangesReceivedEvent implements BoardEvent {
  const BoardChangesReceivedEvent(final  List<String> ids): _ids = ids;
  

 final  List<String> _ids;
 List<String> get ids {
  if (_ids is EqualUnmodifiableListView) return _ids;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_ids);
}


/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardChangesReceivedEventCopyWith<BoardChangesReceivedEvent> get copyWith => _$BoardChangesReceivedEventCopyWithImpl<BoardChangesReceivedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardChangesReceivedEvent&&const DeepCollectionEquality().equals(other._ids, _ids));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_ids));

@override
String toString() {
  return 'BoardEvent.changesReceived(ids: $ids)';
}


}

/// @nodoc
abstract mixin class $BoardChangesReceivedEventCopyWith<$Res> implements $BoardEventCopyWith<$Res> {
  factory $BoardChangesReceivedEventCopyWith(BoardChangesReceivedEvent value, $Res Function(BoardChangesReceivedEvent) _then) = _$BoardChangesReceivedEventCopyWithImpl;
@useResult
$Res call({
 List<String> ids
});




}
/// @nodoc
class _$BoardChangesReceivedEventCopyWithImpl<$Res>
    implements $BoardChangesReceivedEventCopyWith<$Res> {
  _$BoardChangesReceivedEventCopyWithImpl(this._self, this._then);

  final BoardChangesReceivedEvent _self;
  final $Res Function(BoardChangesReceivedEvent) _then;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ids = null,}) {
  return _then(BoardChangesReceivedEvent(
null == ids ? _self._ids : ids // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class BoardRetriedEvent implements BoardEvent {
  const BoardRetriedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardRetriedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BoardEvent.retried()';
}


}




/// @nodoc


class BoardCardMovedEvent implements BoardEvent {
  const BoardCardMovedEvent({required this.card, required this.lane});
  

 final  CardKey card;
 final  BoardLane lane;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardCardMovedEventCopyWith<BoardCardMovedEvent> get copyWith => _$BoardCardMovedEventCopyWithImpl<BoardCardMovedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardCardMovedEvent&&(identical(other.card, card) || other.card == card)&&(identical(other.lane, lane) || other.lane == lane));
}


@override
int get hashCode => Object.hash(runtimeType,card,lane);

@override
String toString() {
  return 'BoardEvent.cardMoved(card: $card, lane: $lane)';
}


}

/// @nodoc
abstract mixin class $BoardCardMovedEventCopyWith<$Res> implements $BoardEventCopyWith<$Res> {
  factory $BoardCardMovedEventCopyWith(BoardCardMovedEvent value, $Res Function(BoardCardMovedEvent) _then) = _$BoardCardMovedEventCopyWithImpl;
@useResult
$Res call({
 CardKey card, BoardLane lane
});


$CardKeyCopyWith<$Res> get card;$BoardLaneCopyWith<$Res> get lane;

}
/// @nodoc
class _$BoardCardMovedEventCopyWithImpl<$Res>
    implements $BoardCardMovedEventCopyWith<$Res> {
  _$BoardCardMovedEventCopyWithImpl(this._self, this._then);

  final BoardCardMovedEvent _self;
  final $Res Function(BoardCardMovedEvent) _then;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? card = null,Object? lane = null,}) {
  return _then(BoardCardMovedEvent(
card: null == card ? _self.card : card // ignore: cast_nullable_to_non_nullable
as CardKey,lane: null == lane ? _self.lane : lane // ignore: cast_nullable_to_non_nullable
as BoardLane,
  ));
}

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$CardKeyCopyWith<$Res> get card {
  
  return $CardKeyCopyWith<$Res>(_self.card, (value) {
    return _then(_self.copyWith(card: value));
  });
}/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$BoardLaneCopyWith<$Res> get lane {
  
  return $BoardLaneCopyWith<$Res>(_self.lane, (value) {
    return _then(_self.copyWith(lane: value));
  });
}
}

/// @nodoc


class BoardNoticeDismissedEvent implements BoardEvent {
  const BoardNoticeDismissedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardNoticeDismissedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BoardEvent.noticeDismissed()';
}


}




/// @nodoc


class BoardGroupChangedEvent implements BoardEvent {
  const BoardGroupChangedEvent(this.by);
  

 final  String? by;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardGroupChangedEventCopyWith<BoardGroupChangedEvent> get copyWith => _$BoardGroupChangedEventCopyWithImpl<BoardGroupChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardGroupChangedEvent&&(identical(other.by, by) || other.by == by));
}


@override
int get hashCode => Object.hash(runtimeType,by);

@override
String toString() {
  return 'BoardEvent.groupChanged(by: $by)';
}


}

/// @nodoc
abstract mixin class $BoardGroupChangedEventCopyWith<$Res> implements $BoardEventCopyWith<$Res> {
  factory $BoardGroupChangedEventCopyWith(BoardGroupChangedEvent value, $Res Function(BoardGroupChangedEvent) _then) = _$BoardGroupChangedEventCopyWithImpl;
@useResult
$Res call({
 String? by
});




}
/// @nodoc
class _$BoardGroupChangedEventCopyWithImpl<$Res>
    implements $BoardGroupChangedEventCopyWith<$Res> {
  _$BoardGroupChangedEventCopyWithImpl(this._self, this._then);

  final BoardGroupChangedEvent _self;
  final $Res Function(BoardGroupChangedEvent) _then;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? by = freezed,}) {
  return _then(BoardGroupChangedEvent(
freezed == by ? _self.by : by // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class BoardShowUnsetChangedEvent implements BoardEvent {
  const BoardShowUnsetChangedEvent({required this.show});
  

 final  bool show;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardShowUnsetChangedEventCopyWith<BoardShowUnsetChangedEvent> get copyWith => _$BoardShowUnsetChangedEventCopyWithImpl<BoardShowUnsetChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardShowUnsetChangedEvent&&(identical(other.show, show) || other.show == show));
}


@override
int get hashCode => Object.hash(runtimeType,show);

@override
String toString() {
  return 'BoardEvent.showUnsetChanged(show: $show)';
}


}

/// @nodoc
abstract mixin class $BoardShowUnsetChangedEventCopyWith<$Res> implements $BoardEventCopyWith<$Res> {
  factory $BoardShowUnsetChangedEventCopyWith(BoardShowUnsetChangedEvent value, $Res Function(BoardShowUnsetChangedEvent) _then) = _$BoardShowUnsetChangedEventCopyWithImpl;
@useResult
$Res call({
 bool show
});




}
/// @nodoc
class _$BoardShowUnsetChangedEventCopyWithImpl<$Res>
    implements $BoardShowUnsetChangedEventCopyWith<$Res> {
  _$BoardShowUnsetChangedEventCopyWithImpl(this._self, this._then);

  final BoardShowUnsetChangedEvent _self;
  final $Res Function(BoardShowUnsetChangedEvent) _then;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? show = null,}) {
  return _then(BoardShowUnsetChangedEvent(
show: null == show ? _self.show : show // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class BoardLaneEditedEvent implements BoardEvent {
  const BoardLaneEditedEvent(this.change);
  

 final  LaneChange change;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardLaneEditedEventCopyWith<BoardLaneEditedEvent> get copyWith => _$BoardLaneEditedEventCopyWithImpl<BoardLaneEditedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardLaneEditedEvent&&(identical(other.change, change) || other.change == change));
}


@override
int get hashCode => Object.hash(runtimeType,change);

@override
String toString() {
  return 'BoardEvent.laneEdited(change: $change)';
}


}

/// @nodoc
abstract mixin class $BoardLaneEditedEventCopyWith<$Res> implements $BoardEventCopyWith<$Res> {
  factory $BoardLaneEditedEventCopyWith(BoardLaneEditedEvent value, $Res Function(BoardLaneEditedEvent) _then) = _$BoardLaneEditedEventCopyWithImpl;
@useResult
$Res call({
 LaneChange change
});


$LaneChangeCopyWith<$Res> get change;

}
/// @nodoc
class _$BoardLaneEditedEventCopyWithImpl<$Res>
    implements $BoardLaneEditedEventCopyWith<$Res> {
  _$BoardLaneEditedEventCopyWithImpl(this._self, this._then);

  final BoardLaneEditedEvent _self;
  final $Res Function(BoardLaneEditedEvent) _then;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? change = null,}) {
  return _then(BoardLaneEditedEvent(
null == change ? _self.change : change // ignore: cast_nullable_to_non_nullable
as LaneChange,
  ));
}

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$LaneChangeCopyWith<$Res> get change {
  
  return $LaneChangeCopyWith<$Res>(_self.change, (value) {
    return _then(_self.copyWith(change: value));
  });
}
}

/// @nodoc


class BoardLaneRemovalRequestedEvent implements BoardEvent {
  const BoardLaneRemovalRequestedEvent(this.lane);
  

 final  String lane;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardLaneRemovalRequestedEventCopyWith<BoardLaneRemovalRequestedEvent> get copyWith => _$BoardLaneRemovalRequestedEventCopyWithImpl<BoardLaneRemovalRequestedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardLaneRemovalRequestedEvent&&(identical(other.lane, lane) || other.lane == lane));
}


@override
int get hashCode => Object.hash(runtimeType,lane);

@override
String toString() {
  return 'BoardEvent.laneRemovalRequested(lane: $lane)';
}


}

/// @nodoc
abstract mixin class $BoardLaneRemovalRequestedEventCopyWith<$Res> implements $BoardEventCopyWith<$Res> {
  factory $BoardLaneRemovalRequestedEventCopyWith(BoardLaneRemovalRequestedEvent value, $Res Function(BoardLaneRemovalRequestedEvent) _then) = _$BoardLaneRemovalRequestedEventCopyWithImpl;
@useResult
$Res call({
 String lane
});




}
/// @nodoc
class _$BoardLaneRemovalRequestedEventCopyWithImpl<$Res>
    implements $BoardLaneRemovalRequestedEventCopyWith<$Res> {
  _$BoardLaneRemovalRequestedEventCopyWithImpl(this._self, this._then);

  final BoardLaneRemovalRequestedEvent _self;
  final $Res Function(BoardLaneRemovalRequestedEvent) _then;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? lane = null,}) {
  return _then(BoardLaneRemovalRequestedEvent(
null == lane ? _self.lane : lane // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class BoardRemovalTargetChosenEvent implements BoardEvent {
  const BoardRemovalTargetChosenEvent(this.target);
  

 final  String? target;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardRemovalTargetChosenEventCopyWith<BoardRemovalTargetChosenEvent> get copyWith => _$BoardRemovalTargetChosenEventCopyWithImpl<BoardRemovalTargetChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardRemovalTargetChosenEvent&&(identical(other.target, target) || other.target == target));
}


@override
int get hashCode => Object.hash(runtimeType,target);

@override
String toString() {
  return 'BoardEvent.removalTargetChosen(target: $target)';
}


}

/// @nodoc
abstract mixin class $BoardRemovalTargetChosenEventCopyWith<$Res> implements $BoardEventCopyWith<$Res> {
  factory $BoardRemovalTargetChosenEventCopyWith(BoardRemovalTargetChosenEvent value, $Res Function(BoardRemovalTargetChosenEvent) _then) = _$BoardRemovalTargetChosenEventCopyWithImpl;
@useResult
$Res call({
 String? target
});




}
/// @nodoc
class _$BoardRemovalTargetChosenEventCopyWithImpl<$Res>
    implements $BoardRemovalTargetChosenEventCopyWith<$Res> {
  _$BoardRemovalTargetChosenEventCopyWithImpl(this._self, this._then);

  final BoardRemovalTargetChosenEvent _self;
  final $Res Function(BoardRemovalTargetChosenEvent) _then;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? target = freezed,}) {
  return _then(BoardRemovalTargetChosenEvent(
freezed == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class BoardRemovalConfirmedEvent implements BoardEvent {
  const BoardRemovalConfirmedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardRemovalConfirmedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BoardEvent.removalConfirmed()';
}


}




/// @nodoc


class BoardRemovalCancelledEvent implements BoardEvent {
  const BoardRemovalCancelledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardRemovalCancelledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BoardEvent.removalCancelled()';
}


}




/// @nodoc


class BoardUndoneEvent implements BoardEvent {
  const BoardUndoneEvent(this.ticket);
  

 final  int ticket;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardUndoneEventCopyWith<BoardUndoneEvent> get copyWith => _$BoardUndoneEventCopyWithImpl<BoardUndoneEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardUndoneEvent&&(identical(other.ticket, ticket) || other.ticket == ticket));
}


@override
int get hashCode => Object.hash(runtimeType,ticket);

@override
String toString() {
  return 'BoardEvent.undone(ticket: $ticket)';
}


}

/// @nodoc
abstract mixin class $BoardUndoneEventCopyWith<$Res> implements $BoardEventCopyWith<$Res> {
  factory $BoardUndoneEventCopyWith(BoardUndoneEvent value, $Res Function(BoardUndoneEvent) _then) = _$BoardUndoneEventCopyWithImpl;
@useResult
$Res call({
 int ticket
});




}
/// @nodoc
class _$BoardUndoneEventCopyWithImpl<$Res>
    implements $BoardUndoneEventCopyWith<$Res> {
  _$BoardUndoneEventCopyWithImpl(this._self, this._then);

  final BoardUndoneEvent _self;
  final $Res Function(BoardUndoneEvent) _then;

/// Create a copy of BoardEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? ticket = null,}) {
  return _then(BoardUndoneEvent(
null == ticket ? _self.ticket : ticket // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class BoardSettingsClosedEvent implements BoardEvent {
  const BoardSettingsClosedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardSettingsClosedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BoardEvent.settingsClosed()';
}


}




/// @nodoc
mixin _$BoardNotice {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardNotice);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BoardNotice()';
}


}

/// @nodoc
class $BoardNoticeCopyWith<$Res>  {
$BoardNoticeCopyWith(BoardNotice _, $Res Function(BoardNotice) __);
}


/// Adds pattern-matching-related methods to [BoardNotice].
extension BoardNoticePatterns on BoardNotice {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( BoardNoticeFailed value)?  failed,TResult Function( BoardNoticeChangedInNote value)?  changedInNote,TResult Function( BoardNoticeDone value)?  done,required TResult orElse(),}){
final _that = this;
switch (_that) {
case BoardNoticeFailed() when failed != null:
return failed(_that);case BoardNoticeChangedInNote() when changedInNote != null:
return changedInNote(_that);case BoardNoticeDone() when done != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( BoardNoticeFailed value)  failed,required TResult Function( BoardNoticeChangedInNote value)  changedInNote,required TResult Function( BoardNoticeDone value)  done,}){
final _that = this;
switch (_that) {
case BoardNoticeFailed():
return failed(_that);case BoardNoticeChangedInNote():
return changedInNote(_that);case BoardNoticeDone():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( BoardNoticeFailed value)?  failed,TResult? Function( BoardNoticeChangedInNote value)?  changedInNote,TResult? Function( BoardNoticeDone value)?  done,}){
final _that = this;
switch (_that) {
case BoardNoticeFailed() when failed != null:
return failed(_that);case BoardNoticeChangedInNote() when changedInNote != null:
return changedInNote(_that);case BoardNoticeDone() when done != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( ApplicationError error)?  failed,TResult Function()?  changedInNote,TResult Function( String message,  int? undo)?  done,required TResult orElse(),}) {final _that = this;
switch (_that) {
case BoardNoticeFailed() when failed != null:
return failed(_that.error);case BoardNoticeChangedInNote() when changedInNote != null:
return changedInNote();case BoardNoticeDone() when done != null:
return done(_that.message,_that.undo);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( ApplicationError error)  failed,required TResult Function()  changedInNote,required TResult Function( String message,  int? undo)  done,}) {final _that = this;
switch (_that) {
case BoardNoticeFailed():
return failed(_that.error);case BoardNoticeChangedInNote():
return changedInNote();case BoardNoticeDone():
return done(_that.message,_that.undo);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( ApplicationError error)?  failed,TResult? Function()?  changedInNote,TResult? Function( String message,  int? undo)?  done,}) {final _that = this;
switch (_that) {
case BoardNoticeFailed() when failed != null:
return failed(_that.error);case BoardNoticeChangedInNote() when changedInNote != null:
return changedInNote();case BoardNoticeDone() when done != null:
return done(_that.message,_that.undo);case _:
  return null;

}
}

}

/// @nodoc


class BoardNoticeFailed implements BoardNotice {
  const BoardNoticeFailed(this.error);
  

 final  ApplicationError error;

/// Create a copy of BoardNotice
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardNoticeFailedCopyWith<BoardNoticeFailed> get copyWith => _$BoardNoticeFailedCopyWithImpl<BoardNoticeFailed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardNoticeFailed&&(identical(other.error, error) || other.error == error));
}


@override
int get hashCode => Object.hash(runtimeType,error);

@override
String toString() {
  return 'BoardNotice.failed(error: $error)';
}


}

/// @nodoc
abstract mixin class $BoardNoticeFailedCopyWith<$Res> implements $BoardNoticeCopyWith<$Res> {
  factory $BoardNoticeFailedCopyWith(BoardNoticeFailed value, $Res Function(BoardNoticeFailed) _then) = _$BoardNoticeFailedCopyWithImpl;
@useResult
$Res call({
 ApplicationError error
});


$ApplicationErrorCopyWith<$Res> get error;

}
/// @nodoc
class _$BoardNoticeFailedCopyWithImpl<$Res>
    implements $BoardNoticeFailedCopyWith<$Res> {
  _$BoardNoticeFailedCopyWithImpl(this._self, this._then);

  final BoardNoticeFailed _self;
  final $Res Function(BoardNoticeFailed) _then;

/// Create a copy of BoardNotice
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? error = null,}) {
  return _then(BoardNoticeFailed(
null == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError,
  ));
}

/// Create a copy of BoardNotice
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ApplicationErrorCopyWith<$Res> get error {
  
  return $ApplicationErrorCopyWith<$Res>(_self.error, (value) {
    return _then(_self.copyWith(error: value));
  });
}
}

/// @nodoc


class BoardNoticeChangedInNote implements BoardNotice {
  const BoardNoticeChangedInNote();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardNoticeChangedInNote);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'BoardNotice.changedInNote()';
}


}




/// @nodoc


class BoardNoticeDone implements BoardNotice {
  const BoardNoticeDone({required this.message, this.undo});
  

 final  String message;
 final  int? undo;

/// Create a copy of BoardNotice
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardNoticeDoneCopyWith<BoardNoticeDone> get copyWith => _$BoardNoticeDoneCopyWithImpl<BoardNoticeDone>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardNoticeDone&&(identical(other.message, message) || other.message == message)&&(identical(other.undo, undo) || other.undo == undo));
}


@override
int get hashCode => Object.hash(runtimeType,message,undo);

@override
String toString() {
  return 'BoardNotice.done(message: $message, undo: $undo)';
}


}

/// @nodoc
abstract mixin class $BoardNoticeDoneCopyWith<$Res> implements $BoardNoticeCopyWith<$Res> {
  factory $BoardNoticeDoneCopyWith(BoardNoticeDone value, $Res Function(BoardNoticeDone) _then) = _$BoardNoticeDoneCopyWithImpl;
@useResult
$Res call({
 String message, int? undo
});




}
/// @nodoc
class _$BoardNoticeDoneCopyWithImpl<$Res>
    implements $BoardNoticeDoneCopyWith<$Res> {
  _$BoardNoticeDoneCopyWithImpl(this._self, this._then);

  final BoardNoticeDone _self;
  final $Res Function(BoardNoticeDone) _then;

/// Create a copy of BoardNotice
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,Object? undo = freezed,}) {
  return _then(BoardNoticeDone(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,undo: freezed == undo ? _self.undo : undo // ignore: cast_nullable_to_non_nullable
as int?,
  ));
}


}

/// @nodoc
mixin _$BoardRemoval {

 String get lane; LaneRemoval get plan;/// The lane the notes go to; null is "no value".
 String? get target;
/// Create a copy of BoardRemoval
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardRemovalCopyWith<BoardRemoval> get copyWith => _$BoardRemovalCopyWithImpl<BoardRemoval>(this as BoardRemoval, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardRemoval&&(identical(other.lane, lane) || other.lane == lane)&&(identical(other.plan, plan) || other.plan == plan)&&(identical(other.target, target) || other.target == target));
}


@override
int get hashCode => Object.hash(runtimeType,lane,plan,target);

@override
String toString() {
  return 'BoardRemoval(lane: $lane, plan: $plan, target: $target)';
}


}

/// @nodoc
abstract mixin class $BoardRemovalCopyWith<$Res>  {
  factory $BoardRemovalCopyWith(BoardRemoval value, $Res Function(BoardRemoval) _then) = _$BoardRemovalCopyWithImpl;
@useResult
$Res call({
 String lane, LaneRemoval plan, String? target
});




}
/// @nodoc
class _$BoardRemovalCopyWithImpl<$Res>
    implements $BoardRemovalCopyWith<$Res> {
  _$BoardRemovalCopyWithImpl(this._self, this._then);

  final BoardRemoval _self;
  final $Res Function(BoardRemoval) _then;

/// Create a copy of BoardRemoval
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? lane = null,Object? plan = null,Object? target = freezed,}) {
  return _then(_self.copyWith(
lane: null == lane ? _self.lane : lane // ignore: cast_nullable_to_non_nullable
as String,plan: null == plan ? _self.plan : plan // ignore: cast_nullable_to_non_nullable
as LaneRemoval,target: freezed == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [BoardRemoval].
extension BoardRemovalPatterns on BoardRemoval {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _BoardRemoval value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _BoardRemoval() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _BoardRemoval value)  $default,){
final _that = this;
switch (_that) {
case _BoardRemoval():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _BoardRemoval value)?  $default,){
final _that = this;
switch (_that) {
case _BoardRemoval() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String lane,  LaneRemoval plan,  String? target)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _BoardRemoval() when $default != null:
return $default(_that.lane,_that.plan,_that.target);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String lane,  LaneRemoval plan,  String? target)  $default,) {final _that = this;
switch (_that) {
case _BoardRemoval():
return $default(_that.lane,_that.plan,_that.target);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String lane,  LaneRemoval plan,  String? target)?  $default,) {final _that = this;
switch (_that) {
case _BoardRemoval() when $default != null:
return $default(_that.lane,_that.plan,_that.target);case _:
  return null;

}
}

}

/// @nodoc


class _BoardRemoval implements BoardRemoval {
  const _BoardRemoval({required this.lane, required this.plan, this.target});
  

@override final  String lane;
@override final  LaneRemoval plan;
/// The lane the notes go to; null is "no value".
@override final  String? target;

/// Create a copy of BoardRemoval
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$BoardRemovalCopyWith<_BoardRemoval> get copyWith => __$BoardRemovalCopyWithImpl<_BoardRemoval>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _BoardRemoval&&(identical(other.lane, lane) || other.lane == lane)&&(identical(other.plan, plan) || other.plan == plan)&&(identical(other.target, target) || other.target == target));
}


@override
int get hashCode => Object.hash(runtimeType,lane,plan,target);

@override
String toString() {
  return 'BoardRemoval(lane: $lane, plan: $plan, target: $target)';
}


}

/// @nodoc
abstract mixin class _$BoardRemovalCopyWith<$Res> implements $BoardRemovalCopyWith<$Res> {
  factory _$BoardRemovalCopyWith(_BoardRemoval value, $Res Function(_BoardRemoval) _then) = __$BoardRemovalCopyWithImpl;
@override @useResult
$Res call({
 String lane, LaneRemoval plan, String? target
});




}
/// @nodoc
class __$BoardRemovalCopyWithImpl<$Res>
    implements _$BoardRemovalCopyWith<$Res> {
  __$BoardRemovalCopyWithImpl(this._self, this._then);

  final _BoardRemoval _self;
  final $Res Function(_BoardRemoval) _then;

/// Create a copy of BoardRemoval
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? lane = null,Object? plan = null,Object? target = freezed,}) {
  return _then(_BoardRemoval(
lane: null == lane ? _self.lane : lane // ignore: cast_nullable_to_non_nullable
as String,plan: null == plan ? _self.plan : plan // ignore: cast_nullable_to_non_nullable
as LaneRemoval,target: freezed == target ? _self.target : target // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc
mixin _$BoardState {

/// The list the board lays out; null until started.
 ListQuery? get query; BoardKind get kind; LoadStatus get status;/// The columns, as last read.
 BoardView? get data;/// Why reading failed, while [status] is failed.
 ApplicationError? get error;/// Why the last move did not happen, until it is dismissed.
 BoardNotice? get notice;/// The lane the settings are asking about, while they are.
 BoardRemoval? get removal;/// Why the last change in the settings was refused, in the core's words.
 String? get settingsError;
/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardStateCopyWith<BoardState> get copyWith => _$BoardStateCopyWithImpl<BoardState>(this as BoardState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardState&&(identical(other.query, query) || other.query == query)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.status, status) || other.status == status)&&(identical(other.data, data) || other.data == data)&&(identical(other.error, error) || other.error == error)&&(identical(other.notice, notice) || other.notice == notice)&&(identical(other.removal, removal) || other.removal == removal)&&(identical(other.settingsError, settingsError) || other.settingsError == settingsError));
}


@override
int get hashCode => Object.hash(runtimeType,query,kind,status,data,error,notice,removal,settingsError);

@override
String toString() {
  return 'BoardState(query: $query, kind: $kind, status: $status, data: $data, error: $error, notice: $notice, removal: $removal, settingsError: $settingsError)';
}


}

/// @nodoc
abstract mixin class $BoardStateCopyWith<$Res>  {
  factory $BoardStateCopyWith(BoardState value, $Res Function(BoardState) _then) = _$BoardStateCopyWithImpl;
@useResult
$Res call({
 ListQuery? query, BoardKind kind, LoadStatus status, BoardView? data, ApplicationError? error, BoardNotice? notice, BoardRemoval? removal, String? settingsError
});


$ApplicationErrorCopyWith<$Res>? get error;$BoardNoticeCopyWith<$Res>? get notice;$BoardRemovalCopyWith<$Res>? get removal;

}
/// @nodoc
class _$BoardStateCopyWithImpl<$Res>
    implements $BoardStateCopyWith<$Res> {
  _$BoardStateCopyWithImpl(this._self, this._then);

  final BoardState _self;
  final $Res Function(BoardState) _then;

/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? query = freezed,Object? kind = null,Object? status = null,Object? data = freezed,Object? error = freezed,Object? notice = freezed,Object? removal = freezed,Object? settingsError = freezed,}) {
  return _then(_self.copyWith(
query: freezed == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as ListQuery?,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as BoardKind,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,data: freezed == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as BoardView?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as BoardNotice?,removal: freezed == removal ? _self.removal : removal // ignore: cast_nullable_to_non_nullable
as BoardRemoval?,settingsError: freezed == settingsError ? _self.settingsError : settingsError // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}
/// Create a copy of BoardState
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
}/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$BoardNoticeCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $BoardNoticeCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$BoardRemovalCopyWith<$Res>? get removal {
    if (_self.removal == null) {
    return null;
  }

  return $BoardRemovalCopyWith<$Res>(_self.removal!, (value) {
    return _then(_self.copyWith(removal: value));
  });
}
}


/// Adds pattern-matching-related methods to [BoardState].
extension BoardStatePatterns on BoardState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _BoardState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _BoardState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _BoardState value)  $default,){
final _that = this;
switch (_that) {
case _BoardState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _BoardState value)?  $default,){
final _that = this;
switch (_that) {
case _BoardState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( ListQuery? query,  BoardKind kind,  LoadStatus status,  BoardView? data,  ApplicationError? error,  BoardNotice? notice,  BoardRemoval? removal,  String? settingsError)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _BoardState() when $default != null:
return $default(_that.query,_that.kind,_that.status,_that.data,_that.error,_that.notice,_that.removal,_that.settingsError);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( ListQuery? query,  BoardKind kind,  LoadStatus status,  BoardView? data,  ApplicationError? error,  BoardNotice? notice,  BoardRemoval? removal,  String? settingsError)  $default,) {final _that = this;
switch (_that) {
case _BoardState():
return $default(_that.query,_that.kind,_that.status,_that.data,_that.error,_that.notice,_that.removal,_that.settingsError);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( ListQuery? query,  BoardKind kind,  LoadStatus status,  BoardView? data,  ApplicationError? error,  BoardNotice? notice,  BoardRemoval? removal,  String? settingsError)?  $default,) {final _that = this;
switch (_that) {
case _BoardState() when $default != null:
return $default(_that.query,_that.kind,_that.status,_that.data,_that.error,_that.notice,_that.removal,_that.settingsError);case _:
  return null;

}
}

}

/// @nodoc


class _BoardState implements BoardState {
  const _BoardState({this.query, this.kind = BoardKind.notes, this.status = LoadStatus.loading, this.data, this.error, this.notice, this.removal, this.settingsError});
  

/// The list the board lays out; null until started.
@override final  ListQuery? query;
@override@JsonKey() final  BoardKind kind;
@override@JsonKey() final  LoadStatus status;
/// The columns, as last read.
@override final  BoardView? data;
/// Why reading failed, while [status] is failed.
@override final  ApplicationError? error;
/// Why the last move did not happen, until it is dismissed.
@override final  BoardNotice? notice;
/// The lane the settings are asking about, while they are.
@override final  BoardRemoval? removal;
/// Why the last change in the settings was refused, in the core's words.
@override final  String? settingsError;

/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$BoardStateCopyWith<_BoardState> get copyWith => __$BoardStateCopyWithImpl<_BoardState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _BoardState&&(identical(other.query, query) || other.query == query)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.status, status) || other.status == status)&&(identical(other.data, data) || other.data == data)&&(identical(other.error, error) || other.error == error)&&(identical(other.notice, notice) || other.notice == notice)&&(identical(other.removal, removal) || other.removal == removal)&&(identical(other.settingsError, settingsError) || other.settingsError == settingsError));
}


@override
int get hashCode => Object.hash(runtimeType,query,kind,status,data,error,notice,removal,settingsError);

@override
String toString() {
  return 'BoardState(query: $query, kind: $kind, status: $status, data: $data, error: $error, notice: $notice, removal: $removal, settingsError: $settingsError)';
}


}

/// @nodoc
abstract mixin class _$BoardStateCopyWith<$Res> implements $BoardStateCopyWith<$Res> {
  factory _$BoardStateCopyWith(_BoardState value, $Res Function(_BoardState) _then) = __$BoardStateCopyWithImpl;
@override @useResult
$Res call({
 ListQuery? query, BoardKind kind, LoadStatus status, BoardView? data, ApplicationError? error, BoardNotice? notice, BoardRemoval? removal, String? settingsError
});


@override $ApplicationErrorCopyWith<$Res>? get error;@override $BoardNoticeCopyWith<$Res>? get notice;@override $BoardRemovalCopyWith<$Res>? get removal;

}
/// @nodoc
class __$BoardStateCopyWithImpl<$Res>
    implements _$BoardStateCopyWith<$Res> {
  __$BoardStateCopyWithImpl(this._self, this._then);

  final _BoardState _self;
  final $Res Function(_BoardState) _then;

/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? query = freezed,Object? kind = null,Object? status = null,Object? data = freezed,Object? error = freezed,Object? notice = freezed,Object? removal = freezed,Object? settingsError = freezed,}) {
  return _then(_BoardState(
query: freezed == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as ListQuery?,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as BoardKind,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,data: freezed == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as BoardView?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as BoardNotice?,removal: freezed == removal ? _self.removal : removal // ignore: cast_nullable_to_non_nullable
as BoardRemoval?,settingsError: freezed == settingsError ? _self.settingsError : settingsError // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of BoardState
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
}/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$BoardNoticeCopyWith<$Res>? get notice {
    if (_self.notice == null) {
    return null;
  }

  return $BoardNoticeCopyWith<$Res>(_self.notice!, (value) {
    return _then(_self.copyWith(notice: value));
  });
}/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$BoardRemovalCopyWith<$Res>? get removal {
    if (_self.removal == null) {
    return null;
  }

  return $BoardRemovalCopyWith<$Res>(_self.removal!, (value) {
    return _then(_self.copyWith(removal: value));
  });
}
}

// dart format on

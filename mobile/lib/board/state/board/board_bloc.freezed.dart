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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( BoardStartedEvent value)?  started,TResult Function( BoardKindChangedEvent value)?  kindChanged,TResult Function( BoardChangesReceivedEvent value)?  changesReceived,TResult Function( BoardRetriedEvent value)?  retried,TResult Function( BoardCardMovedEvent value)?  cardMoved,TResult Function( BoardNoticeDismissedEvent value)?  noticeDismissed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case BoardStartedEvent() when started != null:
return started(_that);case BoardKindChangedEvent() when kindChanged != null:
return kindChanged(_that);case BoardChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case BoardRetriedEvent() when retried != null:
return retried(_that);case BoardCardMovedEvent() when cardMoved != null:
return cardMoved(_that);case BoardNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( BoardStartedEvent value)  started,required TResult Function( BoardKindChangedEvent value)  kindChanged,required TResult Function( BoardChangesReceivedEvent value)  changesReceived,required TResult Function( BoardRetriedEvent value)  retried,required TResult Function( BoardCardMovedEvent value)  cardMoved,required TResult Function( BoardNoticeDismissedEvent value)  noticeDismissed,}){
final _that = this;
switch (_that) {
case BoardStartedEvent():
return started(_that);case BoardKindChangedEvent():
return kindChanged(_that);case BoardChangesReceivedEvent():
return changesReceived(_that);case BoardRetriedEvent():
return retried(_that);case BoardCardMovedEvent():
return cardMoved(_that);case BoardNoticeDismissedEvent():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( BoardStartedEvent value)?  started,TResult? Function( BoardKindChangedEvent value)?  kindChanged,TResult? Function( BoardChangesReceivedEvent value)?  changesReceived,TResult? Function( BoardRetriedEvent value)?  retried,TResult? Function( BoardCardMovedEvent value)?  cardMoved,TResult? Function( BoardNoticeDismissedEvent value)?  noticeDismissed,}){
final _that = this;
switch (_that) {
case BoardStartedEvent() when started != null:
return started(_that);case BoardKindChangedEvent() when kindChanged != null:
return kindChanged(_that);case BoardChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that);case BoardRetriedEvent() when retried != null:
return retried(_that);case BoardCardMovedEvent() when cardMoved != null:
return cardMoved(_that);case BoardNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( ListQuery query,  BoardKind kind)?  started,TResult Function( BoardKind kind)?  kindChanged,TResult Function( List<String> ids)?  changesReceived,TResult Function()?  retried,TResult Function( CardKey card,  BoardLane lane)?  cardMoved,TResult Function()?  noticeDismissed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case BoardStartedEvent() when started != null:
return started(_that.query,_that.kind);case BoardKindChangedEvent() when kindChanged != null:
return kindChanged(_that.kind);case BoardChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case BoardRetriedEvent() when retried != null:
return retried();case BoardCardMovedEvent() when cardMoved != null:
return cardMoved(_that.card,_that.lane);case BoardNoticeDismissedEvent() when noticeDismissed != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( ListQuery query,  BoardKind kind)  started,required TResult Function( BoardKind kind)  kindChanged,required TResult Function( List<String> ids)  changesReceived,required TResult Function()  retried,required TResult Function( CardKey card,  BoardLane lane)  cardMoved,required TResult Function()  noticeDismissed,}) {final _that = this;
switch (_that) {
case BoardStartedEvent():
return started(_that.query,_that.kind);case BoardKindChangedEvent():
return kindChanged(_that.kind);case BoardChangesReceivedEvent():
return changesReceived(_that.ids);case BoardRetriedEvent():
return retried();case BoardCardMovedEvent():
return cardMoved(_that.card,_that.lane);case BoardNoticeDismissedEvent():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( ListQuery query,  BoardKind kind)?  started,TResult? Function( BoardKind kind)?  kindChanged,TResult? Function( List<String> ids)?  changesReceived,TResult? Function()?  retried,TResult? Function( CardKey card,  BoardLane lane)?  cardMoved,TResult? Function()?  noticeDismissed,}) {final _that = this;
switch (_that) {
case BoardStartedEvent() when started != null:
return started(_that.query,_that.kind);case BoardKindChangedEvent() when kindChanged != null:
return kindChanged(_that.kind);case BoardChangesReceivedEvent() when changesReceived != null:
return changesReceived(_that.ids);case BoardRetriedEvent() when retried != null:
return retried();case BoardCardMovedEvent() when cardMoved != null:
return cardMoved(_that.card,_that.lane);case BoardNoticeDismissedEvent() when noticeDismissed != null:
return noticeDismissed();case _:
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( BoardNoticeFailed value)?  failed,TResult Function( BoardNoticeChangedInNote value)?  changedInNote,required TResult orElse(),}){
final _that = this;
switch (_that) {
case BoardNoticeFailed() when failed != null:
return failed(_that);case BoardNoticeChangedInNote() when changedInNote != null:
return changedInNote(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( BoardNoticeFailed value)  failed,required TResult Function( BoardNoticeChangedInNote value)  changedInNote,}){
final _that = this;
switch (_that) {
case BoardNoticeFailed():
return failed(_that);case BoardNoticeChangedInNote():
return changedInNote(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( BoardNoticeFailed value)?  failed,TResult? Function( BoardNoticeChangedInNote value)?  changedInNote,}){
final _that = this;
switch (_that) {
case BoardNoticeFailed() when failed != null:
return failed(_that);case BoardNoticeChangedInNote() when changedInNote != null:
return changedInNote(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( ApplicationError error)?  failed,TResult Function()?  changedInNote,required TResult orElse(),}) {final _that = this;
switch (_that) {
case BoardNoticeFailed() when failed != null:
return failed(_that.error);case BoardNoticeChangedInNote() when changedInNote != null:
return changedInNote();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( ApplicationError error)  failed,required TResult Function()  changedInNote,}) {final _that = this;
switch (_that) {
case BoardNoticeFailed():
return failed(_that.error);case BoardNoticeChangedInNote():
return changedInNote();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( ApplicationError error)?  failed,TResult? Function()?  changedInNote,}) {final _that = this;
switch (_that) {
case BoardNoticeFailed() when failed != null:
return failed(_that.error);case BoardNoticeChangedInNote() when changedInNote != null:
return changedInNote();case _:
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
mixin _$BoardState {

/// The list the board lays out; null until started.
 ListQuery? get query; BoardKind get kind; LoadStatus get status;/// The columns, as last read.
 BoardView? get data;/// Why reading failed, while [status] is failed.
 ApplicationError? get error;/// Why the last move did not happen, until it is dismissed.
 BoardNotice? get notice;
/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$BoardStateCopyWith<BoardState> get copyWith => _$BoardStateCopyWithImpl<BoardState>(this as BoardState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is BoardState&&(identical(other.query, query) || other.query == query)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.status, status) || other.status == status)&&(identical(other.data, data) || other.data == data)&&(identical(other.error, error) || other.error == error)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,query,kind,status,data,error,notice);

@override
String toString() {
  return 'BoardState(query: $query, kind: $kind, status: $status, data: $data, error: $error, notice: $notice)';
}


}

/// @nodoc
abstract mixin class $BoardStateCopyWith<$Res>  {
  factory $BoardStateCopyWith(BoardState value, $Res Function(BoardState) _then) = _$BoardStateCopyWithImpl;
@useResult
$Res call({
 ListQuery? query, BoardKind kind, LoadStatus status, BoardView? data, ApplicationError? error, BoardNotice? notice
});


$ApplicationErrorCopyWith<$Res>? get error;$BoardNoticeCopyWith<$Res>? get notice;

}
/// @nodoc
class _$BoardStateCopyWithImpl<$Res>
    implements $BoardStateCopyWith<$Res> {
  _$BoardStateCopyWithImpl(this._self, this._then);

  final BoardState _self;
  final $Res Function(BoardState) _then;

/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? query = freezed,Object? kind = null,Object? status = null,Object? data = freezed,Object? error = freezed,Object? notice = freezed,}) {
  return _then(_self.copyWith(
query: freezed == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as ListQuery?,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as BoardKind,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,data: freezed == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as BoardView?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as BoardNotice?,
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( ListQuery? query,  BoardKind kind,  LoadStatus status,  BoardView? data,  ApplicationError? error,  BoardNotice? notice)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _BoardState() when $default != null:
return $default(_that.query,_that.kind,_that.status,_that.data,_that.error,_that.notice);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( ListQuery? query,  BoardKind kind,  LoadStatus status,  BoardView? data,  ApplicationError? error,  BoardNotice? notice)  $default,) {final _that = this;
switch (_that) {
case _BoardState():
return $default(_that.query,_that.kind,_that.status,_that.data,_that.error,_that.notice);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( ListQuery? query,  BoardKind kind,  LoadStatus status,  BoardView? data,  ApplicationError? error,  BoardNotice? notice)?  $default,) {final _that = this;
switch (_that) {
case _BoardState() when $default != null:
return $default(_that.query,_that.kind,_that.status,_that.data,_that.error,_that.notice);case _:
  return null;

}
}

}

/// @nodoc


class _BoardState implements BoardState {
  const _BoardState({this.query, this.kind = BoardKind.notes, this.status = LoadStatus.loading, this.data, this.error, this.notice});
  

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

/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$BoardStateCopyWith<_BoardState> get copyWith => __$BoardStateCopyWithImpl<_BoardState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _BoardState&&(identical(other.query, query) || other.query == query)&&(identical(other.kind, kind) || other.kind == kind)&&(identical(other.status, status) || other.status == status)&&(identical(other.data, data) || other.data == data)&&(identical(other.error, error) || other.error == error)&&(identical(other.notice, notice) || other.notice == notice));
}


@override
int get hashCode => Object.hash(runtimeType,query,kind,status,data,error,notice);

@override
String toString() {
  return 'BoardState(query: $query, kind: $kind, status: $status, data: $data, error: $error, notice: $notice)';
}


}

/// @nodoc
abstract mixin class _$BoardStateCopyWith<$Res> implements $BoardStateCopyWith<$Res> {
  factory _$BoardStateCopyWith(_BoardState value, $Res Function(_BoardState) _then) = __$BoardStateCopyWithImpl;
@override @useResult
$Res call({
 ListQuery? query, BoardKind kind, LoadStatus status, BoardView? data, ApplicationError? error, BoardNotice? notice
});


@override $ApplicationErrorCopyWith<$Res>? get error;@override $BoardNoticeCopyWith<$Res>? get notice;

}
/// @nodoc
class __$BoardStateCopyWithImpl<$Res>
    implements _$BoardStateCopyWith<$Res> {
  __$BoardStateCopyWithImpl(this._self, this._then);

  final _BoardState _self;
  final $Res Function(_BoardState) _then;

/// Create a copy of BoardState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? query = freezed,Object? kind = null,Object? status = null,Object? data = freezed,Object? error = freezed,Object? notice = freezed,}) {
  return _then(_BoardState(
query: freezed == query ? _self.query : query // ignore: cast_nullable_to_non_nullable
as ListQuery?,kind: null == kind ? _self.kind : kind // ignore: cast_nullable_to_non_nullable
as BoardKind,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,data: freezed == data ? _self.data : data // ignore: cast_nullable_to_non_nullable
as BoardView?,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,notice: freezed == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as BoardNotice?,
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
}
}

// dart format on

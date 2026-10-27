// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'comments_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$CommentsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CommentsEvent()';
}


}

/// @nodoc
class $CommentsEventCopyWith<$Res>  {
$CommentsEventCopyWith(CommentsEvent _, $Res Function(CommentsEvent) __);
}


/// Adds pattern-matching-related methods to [CommentsEvent].
extension CommentsEventPatterns on CommentsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( CommentsStartedEvent value)?  started,TResult Function( CommentsRefreshedEvent value)?  refreshed,TResult Function( CommentsResolvedEvent value)?  resolved,TResult Function( CommentsEditedEvent value)?  edited,TResult Function( CommentsDeletedEvent value)?  deleted,TResult Function( CommentsSelectedEvent value)?  selected,TResult Function( CommentsSelectionHandledEvent value)?  selectionHandled,required TResult orElse(),}){
final _that = this;
switch (_that) {
case CommentsStartedEvent() when started != null:
return started(_that);case CommentsRefreshedEvent() when refreshed != null:
return refreshed(_that);case CommentsResolvedEvent() when resolved != null:
return resolved(_that);case CommentsEditedEvent() when edited != null:
return edited(_that);case CommentsDeletedEvent() when deleted != null:
return deleted(_that);case CommentsSelectedEvent() when selected != null:
return selected(_that);case CommentsSelectionHandledEvent() when selectionHandled != null:
return selectionHandled(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( CommentsStartedEvent value)  started,required TResult Function( CommentsRefreshedEvent value)  refreshed,required TResult Function( CommentsResolvedEvent value)  resolved,required TResult Function( CommentsEditedEvent value)  edited,required TResult Function( CommentsDeletedEvent value)  deleted,required TResult Function( CommentsSelectedEvent value)  selected,required TResult Function( CommentsSelectionHandledEvent value)  selectionHandled,}){
final _that = this;
switch (_that) {
case CommentsStartedEvent():
return started(_that);case CommentsRefreshedEvent():
return refreshed(_that);case CommentsResolvedEvent():
return resolved(_that);case CommentsEditedEvent():
return edited(_that);case CommentsDeletedEvent():
return deleted(_that);case CommentsSelectedEvent():
return selected(_that);case CommentsSelectionHandledEvent():
return selectionHandled(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( CommentsStartedEvent value)?  started,TResult? Function( CommentsRefreshedEvent value)?  refreshed,TResult? Function( CommentsResolvedEvent value)?  resolved,TResult? Function( CommentsEditedEvent value)?  edited,TResult? Function( CommentsDeletedEvent value)?  deleted,TResult? Function( CommentsSelectedEvent value)?  selected,TResult? Function( CommentsSelectionHandledEvent value)?  selectionHandled,}){
final _that = this;
switch (_that) {
case CommentsStartedEvent() when started != null:
return started(_that);case CommentsRefreshedEvent() when refreshed != null:
return refreshed(_that);case CommentsResolvedEvent() when resolved != null:
return resolved(_that);case CommentsEditedEvent() when edited != null:
return edited(_that);case CommentsDeletedEvent() when deleted != null:
return deleted(_that);case CommentsSelectedEvent() when selected != null:
return selected(_that);case CommentsSelectionHandledEvent() when selectionHandled != null:
return selectionHandled(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  started,TResult Function()?  refreshed,TResult Function( String commentId,  bool resolved)?  resolved,TResult Function( String commentId,  String body)?  edited,TResult Function( String commentId)?  deleted,TResult Function( String commentId)?  selected,TResult Function()?  selectionHandled,required TResult orElse(),}) {final _that = this;
switch (_that) {
case CommentsStartedEvent() when started != null:
return started(_that.noteId);case CommentsRefreshedEvent() when refreshed != null:
return refreshed();case CommentsResolvedEvent() when resolved != null:
return resolved(_that.commentId,_that.resolved);case CommentsEditedEvent() when edited != null:
return edited(_that.commentId,_that.body);case CommentsDeletedEvent() when deleted != null:
return deleted(_that.commentId);case CommentsSelectedEvent() when selected != null:
return selected(_that.commentId);case CommentsSelectionHandledEvent() when selectionHandled != null:
return selectionHandled();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  started,required TResult Function()  refreshed,required TResult Function( String commentId,  bool resolved)  resolved,required TResult Function( String commentId,  String body)  edited,required TResult Function( String commentId)  deleted,required TResult Function( String commentId)  selected,required TResult Function()  selectionHandled,}) {final _that = this;
switch (_that) {
case CommentsStartedEvent():
return started(_that.noteId);case CommentsRefreshedEvent():
return refreshed();case CommentsResolvedEvent():
return resolved(_that.commentId,_that.resolved);case CommentsEditedEvent():
return edited(_that.commentId,_that.body);case CommentsDeletedEvent():
return deleted(_that.commentId);case CommentsSelectedEvent():
return selected(_that.commentId);case CommentsSelectionHandledEvent():
return selectionHandled();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  started,TResult? Function()?  refreshed,TResult? Function( String commentId,  bool resolved)?  resolved,TResult? Function( String commentId,  String body)?  edited,TResult? Function( String commentId)?  deleted,TResult? Function( String commentId)?  selected,TResult? Function()?  selectionHandled,}) {final _that = this;
switch (_that) {
case CommentsStartedEvent() when started != null:
return started(_that.noteId);case CommentsRefreshedEvent() when refreshed != null:
return refreshed();case CommentsResolvedEvent() when resolved != null:
return resolved(_that.commentId,_that.resolved);case CommentsEditedEvent() when edited != null:
return edited(_that.commentId,_that.body);case CommentsDeletedEvent() when deleted != null:
return deleted(_that.commentId);case CommentsSelectedEvent() when selected != null:
return selected(_that.commentId);case CommentsSelectionHandledEvent() when selectionHandled != null:
return selectionHandled();case _:
  return null;

}
}

}

/// @nodoc


class CommentsStartedEvent implements CommentsEvent {
  const CommentsStartedEvent(this.noteId);
  

 final  String noteId;

/// Create a copy of CommentsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CommentsStartedEventCopyWith<CommentsStartedEvent> get copyWith => _$CommentsStartedEventCopyWithImpl<CommentsStartedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentsStartedEvent&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'CommentsEvent.started(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $CommentsStartedEventCopyWith<$Res> implements $CommentsEventCopyWith<$Res> {
  factory $CommentsStartedEventCopyWith(CommentsStartedEvent value, $Res Function(CommentsStartedEvent) _then) = _$CommentsStartedEventCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$CommentsStartedEventCopyWithImpl<$Res>
    implements $CommentsStartedEventCopyWith<$Res> {
  _$CommentsStartedEventCopyWithImpl(this._self, this._then);

  final CommentsStartedEvent _self;
  final $Res Function(CommentsStartedEvent) _then;

/// Create a copy of CommentsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(CommentsStartedEvent(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CommentsRefreshedEvent implements CommentsEvent {
  const CommentsRefreshedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentsRefreshedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CommentsEvent.refreshed()';
}


}




/// @nodoc


class CommentsResolvedEvent implements CommentsEvent {
  const CommentsResolvedEvent({required this.commentId, required this.resolved});
  

 final  String commentId;
 final  bool resolved;

/// Create a copy of CommentsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CommentsResolvedEventCopyWith<CommentsResolvedEvent> get copyWith => _$CommentsResolvedEventCopyWithImpl<CommentsResolvedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentsResolvedEvent&&(identical(other.commentId, commentId) || other.commentId == commentId)&&(identical(other.resolved, resolved) || other.resolved == resolved));
}


@override
int get hashCode => Object.hash(runtimeType,commentId,resolved);

@override
String toString() {
  return 'CommentsEvent.resolved(commentId: $commentId, resolved: $resolved)';
}


}

/// @nodoc
abstract mixin class $CommentsResolvedEventCopyWith<$Res> implements $CommentsEventCopyWith<$Res> {
  factory $CommentsResolvedEventCopyWith(CommentsResolvedEvent value, $Res Function(CommentsResolvedEvent) _then) = _$CommentsResolvedEventCopyWithImpl;
@useResult
$Res call({
 String commentId, bool resolved
});




}
/// @nodoc
class _$CommentsResolvedEventCopyWithImpl<$Res>
    implements $CommentsResolvedEventCopyWith<$Res> {
  _$CommentsResolvedEventCopyWithImpl(this._self, this._then);

  final CommentsResolvedEvent _self;
  final $Res Function(CommentsResolvedEvent) _then;

/// Create a copy of CommentsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? commentId = null,Object? resolved = null,}) {
  return _then(CommentsResolvedEvent(
commentId: null == commentId ? _self.commentId : commentId // ignore: cast_nullable_to_non_nullable
as String,resolved: null == resolved ? _self.resolved : resolved // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class CommentsEditedEvent implements CommentsEvent {
  const CommentsEditedEvent({required this.commentId, required this.body});
  

 final  String commentId;
 final  String body;

/// Create a copy of CommentsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CommentsEditedEventCopyWith<CommentsEditedEvent> get copyWith => _$CommentsEditedEventCopyWithImpl<CommentsEditedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentsEditedEvent&&(identical(other.commentId, commentId) || other.commentId == commentId)&&(identical(other.body, body) || other.body == body));
}


@override
int get hashCode => Object.hash(runtimeType,commentId,body);

@override
String toString() {
  return 'CommentsEvent.edited(commentId: $commentId, body: $body)';
}


}

/// @nodoc
abstract mixin class $CommentsEditedEventCopyWith<$Res> implements $CommentsEventCopyWith<$Res> {
  factory $CommentsEditedEventCopyWith(CommentsEditedEvent value, $Res Function(CommentsEditedEvent) _then) = _$CommentsEditedEventCopyWithImpl;
@useResult
$Res call({
 String commentId, String body
});




}
/// @nodoc
class _$CommentsEditedEventCopyWithImpl<$Res>
    implements $CommentsEditedEventCopyWith<$Res> {
  _$CommentsEditedEventCopyWithImpl(this._self, this._then);

  final CommentsEditedEvent _self;
  final $Res Function(CommentsEditedEvent) _then;

/// Create a copy of CommentsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? commentId = null,Object? body = null,}) {
  return _then(CommentsEditedEvent(
commentId: null == commentId ? _self.commentId : commentId // ignore: cast_nullable_to_non_nullable
as String,body: null == body ? _self.body : body // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CommentsDeletedEvent implements CommentsEvent {
  const CommentsDeletedEvent(this.commentId);
  

 final  String commentId;

/// Create a copy of CommentsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CommentsDeletedEventCopyWith<CommentsDeletedEvent> get copyWith => _$CommentsDeletedEventCopyWithImpl<CommentsDeletedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentsDeletedEvent&&(identical(other.commentId, commentId) || other.commentId == commentId));
}


@override
int get hashCode => Object.hash(runtimeType,commentId);

@override
String toString() {
  return 'CommentsEvent.deleted(commentId: $commentId)';
}


}

/// @nodoc
abstract mixin class $CommentsDeletedEventCopyWith<$Res> implements $CommentsEventCopyWith<$Res> {
  factory $CommentsDeletedEventCopyWith(CommentsDeletedEvent value, $Res Function(CommentsDeletedEvent) _then) = _$CommentsDeletedEventCopyWithImpl;
@useResult
$Res call({
 String commentId
});




}
/// @nodoc
class _$CommentsDeletedEventCopyWithImpl<$Res>
    implements $CommentsDeletedEventCopyWith<$Res> {
  _$CommentsDeletedEventCopyWithImpl(this._self, this._then);

  final CommentsDeletedEvent _self;
  final $Res Function(CommentsDeletedEvent) _then;

/// Create a copy of CommentsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? commentId = null,}) {
  return _then(CommentsDeletedEvent(
null == commentId ? _self.commentId : commentId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CommentsSelectedEvent implements CommentsEvent {
  const CommentsSelectedEvent(this.commentId);
  

 final  String commentId;

/// Create a copy of CommentsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CommentsSelectedEventCopyWith<CommentsSelectedEvent> get copyWith => _$CommentsSelectedEventCopyWithImpl<CommentsSelectedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentsSelectedEvent&&(identical(other.commentId, commentId) || other.commentId == commentId));
}


@override
int get hashCode => Object.hash(runtimeType,commentId);

@override
String toString() {
  return 'CommentsEvent.selected(commentId: $commentId)';
}


}

/// @nodoc
abstract mixin class $CommentsSelectedEventCopyWith<$Res> implements $CommentsEventCopyWith<$Res> {
  factory $CommentsSelectedEventCopyWith(CommentsSelectedEvent value, $Res Function(CommentsSelectedEvent) _then) = _$CommentsSelectedEventCopyWithImpl;
@useResult
$Res call({
 String commentId
});




}
/// @nodoc
class _$CommentsSelectedEventCopyWithImpl<$Res>
    implements $CommentsSelectedEventCopyWith<$Res> {
  _$CommentsSelectedEventCopyWithImpl(this._self, this._then);

  final CommentsSelectedEvent _self;
  final $Res Function(CommentsSelectedEvent) _then;

/// Create a copy of CommentsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? commentId = null,}) {
  return _then(CommentsSelectedEvent(
null == commentId ? _self.commentId : commentId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class CommentsSelectionHandledEvent implements CommentsEvent {
  const CommentsSelectionHandledEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentsSelectionHandledEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'CommentsEvent.selectionHandled()';
}


}




/// @nodoc
mixin _$CommentsState {

/// The note whose comments these are.
 String get noteId; LoadStatus get status;/// This account's role in the note; null when it isn't shared.
 ShareRole? get role;/// In the order the desktop lists them: on words still in the text, in
/// text order, then the orphaned ones.
 List<CommentRow> get comments;/// "2 open · 1 resolved", empty with none.
 String get band;/// Why the comments could not be read, or the last change made.
 ApplicationError? get error;/// A comment whose words the editor is to select, once it is open.
 String? get selecting;
/// Create a copy of CommentsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$CommentsStateCopyWith<CommentsState> get copyWith => _$CommentsStateCopyWithImpl<CommentsState>(this as CommentsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is CommentsState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.role, role) || other.role == role)&&const DeepCollectionEquality().equals(other.comments, comments)&&(identical(other.band, band) || other.band == band)&&(identical(other.error, error) || other.error == error)&&(identical(other.selecting, selecting) || other.selecting == selecting));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,status,role,const DeepCollectionEquality().hash(comments),band,error,selecting);

@override
String toString() {
  return 'CommentsState(noteId: $noteId, status: $status, role: $role, comments: $comments, band: $band, error: $error, selecting: $selecting)';
}


}

/// @nodoc
abstract mixin class $CommentsStateCopyWith<$Res>  {
  factory $CommentsStateCopyWith(CommentsState value, $Res Function(CommentsState) _then) = _$CommentsStateCopyWithImpl;
@useResult
$Res call({
 String noteId, LoadStatus status, ShareRole? role, List<CommentRow> comments, String band, ApplicationError? error, String? selecting
});


$ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class _$CommentsStateCopyWithImpl<$Res>
    implements $CommentsStateCopyWith<$Res> {
  _$CommentsStateCopyWithImpl(this._self, this._then);

  final CommentsState _self;
  final $Res Function(CommentsState) _then;

/// Create a copy of CommentsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? noteId = null,Object? status = null,Object? role = freezed,Object? comments = null,Object? band = null,Object? error = freezed,Object? selecting = freezed,}) {
  return _then(_self.copyWith(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,role: freezed == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as ShareRole?,comments: null == comments ? _self.comments : comments // ignore: cast_nullable_to_non_nullable
as List<CommentRow>,band: null == band ? _self.band : band // ignore: cast_nullable_to_non_nullable
as String,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,selecting: freezed == selecting ? _self.selecting : selecting // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}
/// Create a copy of CommentsState
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


/// Adds pattern-matching-related methods to [CommentsState].
extension CommentsStatePatterns on CommentsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _CommentsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _CommentsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _CommentsState value)  $default,){
final _that = this;
switch (_that) {
case _CommentsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _CommentsState value)?  $default,){
final _that = this;
switch (_that) {
case _CommentsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String noteId,  LoadStatus status,  ShareRole? role,  List<CommentRow> comments,  String band,  ApplicationError? error,  String? selecting)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _CommentsState() when $default != null:
return $default(_that.noteId,_that.status,_that.role,_that.comments,_that.band,_that.error,_that.selecting);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String noteId,  LoadStatus status,  ShareRole? role,  List<CommentRow> comments,  String band,  ApplicationError? error,  String? selecting)  $default,) {final _that = this;
switch (_that) {
case _CommentsState():
return $default(_that.noteId,_that.status,_that.role,_that.comments,_that.band,_that.error,_that.selecting);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String noteId,  LoadStatus status,  ShareRole? role,  List<CommentRow> comments,  String band,  ApplicationError? error,  String? selecting)?  $default,) {final _that = this;
switch (_that) {
case _CommentsState() when $default != null:
return $default(_that.noteId,_that.status,_that.role,_that.comments,_that.band,_that.error,_that.selecting);case _:
  return null;

}
}

}

/// @nodoc


class _CommentsState extends CommentsState {
  const _CommentsState({this.noteId = '', this.status = LoadStatus.loading, this.role, final  List<CommentRow> comments = const <CommentRow>[], this.band = '', this.error, this.selecting}): _comments = comments,super._();
  

/// The note whose comments these are.
@override@JsonKey() final  String noteId;
@override@JsonKey() final  LoadStatus status;
/// This account's role in the note; null when it isn't shared.
@override final  ShareRole? role;
/// In the order the desktop lists them: on words still in the text, in
/// text order, then the orphaned ones.
 final  List<CommentRow> _comments;
/// In the order the desktop lists them: on words still in the text, in
/// text order, then the orphaned ones.
@override@JsonKey() List<CommentRow> get comments {
  if (_comments is EqualUnmodifiableListView) return _comments;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_comments);
}

/// "2 open · 1 resolved", empty with none.
@override@JsonKey() final  String band;
/// Why the comments could not be read, or the last change made.
@override final  ApplicationError? error;
/// A comment whose words the editor is to select, once it is open.
@override final  String? selecting;

/// Create a copy of CommentsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$CommentsStateCopyWith<_CommentsState> get copyWith => __$CommentsStateCopyWithImpl<_CommentsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _CommentsState&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.status, status) || other.status == status)&&(identical(other.role, role) || other.role == role)&&const DeepCollectionEquality().equals(other._comments, _comments)&&(identical(other.band, band) || other.band == band)&&(identical(other.error, error) || other.error == error)&&(identical(other.selecting, selecting) || other.selecting == selecting));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,status,role,const DeepCollectionEquality().hash(_comments),band,error,selecting);

@override
String toString() {
  return 'CommentsState(noteId: $noteId, status: $status, role: $role, comments: $comments, band: $band, error: $error, selecting: $selecting)';
}


}

/// @nodoc
abstract mixin class _$CommentsStateCopyWith<$Res> implements $CommentsStateCopyWith<$Res> {
  factory _$CommentsStateCopyWith(_CommentsState value, $Res Function(_CommentsState) _then) = __$CommentsStateCopyWithImpl;
@override @useResult
$Res call({
 String noteId, LoadStatus status, ShareRole? role, List<CommentRow> comments, String band, ApplicationError? error, String? selecting
});


@override $ApplicationErrorCopyWith<$Res>? get error;

}
/// @nodoc
class __$CommentsStateCopyWithImpl<$Res>
    implements _$CommentsStateCopyWith<$Res> {
  __$CommentsStateCopyWithImpl(this._self, this._then);

  final _CommentsState _self;
  final $Res Function(_CommentsState) _then;

/// Create a copy of CommentsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? status = null,Object? role = freezed,Object? comments = null,Object? band = null,Object? error = freezed,Object? selecting = freezed,}) {
  return _then(_CommentsState(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,role: freezed == role ? _self.role : role // ignore: cast_nullable_to_non_nullable
as ShareRole?,comments: null == comments ? _self._comments : comments // ignore: cast_nullable_to_non_nullable
as List<CommentRow>,band: null == band ? _self.band : band // ignore: cast_nullable_to_non_nullable
as String,error: freezed == error ? _self.error : error // ignore: cast_nullable_to_non_nullable
as ApplicationError?,selecting: freezed == selecting ? _self.selecting : selecting // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of CommentsState
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

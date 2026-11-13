// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'shared_notebooks_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$SharedNotebooksEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNotebooksEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SharedNotebooksEvent()';
}


}

/// @nodoc
class $SharedNotebooksEventCopyWith<$Res>  {
$SharedNotebooksEventCopyWith(SharedNotebooksEvent _, $Res Function(SharedNotebooksEvent) __);
}


/// Adds pattern-matching-related methods to [SharedNotebooksEvent].
extension SharedNotebooksEventPatterns on SharedNotebooksEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SharedNotebooksStartedEvent value)?  started,TResult Function( SharedNotebooksRefreshedEvent value)?  refreshed,TResult Function( SharedNotebooksAcceptedEvent value)?  accepted,TResult Function( SharedNotebooksDeclinedEvent value)?  declined,TResult Function( SharedNotebooksNoticeReadEvent value)?  noticeRead,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SharedNotebooksStartedEvent() when started != null:
return started(_that);case SharedNotebooksRefreshedEvent() when refreshed != null:
return refreshed(_that);case SharedNotebooksAcceptedEvent() when accepted != null:
return accepted(_that);case SharedNotebooksDeclinedEvent() when declined != null:
return declined(_that);case SharedNotebooksNoticeReadEvent() when noticeRead != null:
return noticeRead(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SharedNotebooksStartedEvent value)  started,required TResult Function( SharedNotebooksRefreshedEvent value)  refreshed,required TResult Function( SharedNotebooksAcceptedEvent value)  accepted,required TResult Function( SharedNotebooksDeclinedEvent value)  declined,required TResult Function( SharedNotebooksNoticeReadEvent value)  noticeRead,}){
final _that = this;
switch (_that) {
case SharedNotebooksStartedEvent():
return started(_that);case SharedNotebooksRefreshedEvent():
return refreshed(_that);case SharedNotebooksAcceptedEvent():
return accepted(_that);case SharedNotebooksDeclinedEvent():
return declined(_that);case SharedNotebooksNoticeReadEvent():
return noticeRead(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SharedNotebooksStartedEvent value)?  started,TResult? Function( SharedNotebooksRefreshedEvent value)?  refreshed,TResult? Function( SharedNotebooksAcceptedEvent value)?  accepted,TResult? Function( SharedNotebooksDeclinedEvent value)?  declined,TResult? Function( SharedNotebooksNoticeReadEvent value)?  noticeRead,}){
final _that = this;
switch (_that) {
case SharedNotebooksStartedEvent() when started != null:
return started(_that);case SharedNotebooksRefreshedEvent() when refreshed != null:
return refreshed(_that);case SharedNotebooksAcceptedEvent() when accepted != null:
return accepted(_that);case SharedNotebooksDeclinedEvent() when declined != null:
return declined(_that);case SharedNotebooksNoticeReadEvent() when noticeRead != null:
return noticeRead(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( List<String> notices)?  refreshed,TResult Function( String shareId)?  accepted,TResult Function( String shareId)?  declined,TResult Function()?  noticeRead,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SharedNotebooksStartedEvent() when started != null:
return started();case SharedNotebooksRefreshedEvent() when refreshed != null:
return refreshed(_that.notices);case SharedNotebooksAcceptedEvent() when accepted != null:
return accepted(_that.shareId);case SharedNotebooksDeclinedEvent() when declined != null:
return declined(_that.shareId);case SharedNotebooksNoticeReadEvent() when noticeRead != null:
return noticeRead();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( List<String> notices)  refreshed,required TResult Function( String shareId)  accepted,required TResult Function( String shareId)  declined,required TResult Function()  noticeRead,}) {final _that = this;
switch (_that) {
case SharedNotebooksStartedEvent():
return started();case SharedNotebooksRefreshedEvent():
return refreshed(_that.notices);case SharedNotebooksAcceptedEvent():
return accepted(_that.shareId);case SharedNotebooksDeclinedEvent():
return declined(_that.shareId);case SharedNotebooksNoticeReadEvent():
return noticeRead();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( List<String> notices)?  refreshed,TResult? Function( String shareId)?  accepted,TResult? Function( String shareId)?  declined,TResult? Function()?  noticeRead,}) {final _that = this;
switch (_that) {
case SharedNotebooksStartedEvent() when started != null:
return started();case SharedNotebooksRefreshedEvent() when refreshed != null:
return refreshed(_that.notices);case SharedNotebooksAcceptedEvent() when accepted != null:
return accepted(_that.shareId);case SharedNotebooksDeclinedEvent() when declined != null:
return declined(_that.shareId);case SharedNotebooksNoticeReadEvent() when noticeRead != null:
return noticeRead();case _:
  return null;

}
}

}

/// @nodoc


class SharedNotebooksStartedEvent implements SharedNotebooksEvent {
  const SharedNotebooksStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNotebooksStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SharedNotebooksEvent.started()';
}


}




/// @nodoc


class SharedNotebooksRefreshedEvent implements SharedNotebooksEvent {
  const SharedNotebooksRefreshedEvent({final  List<String> notices = const <String>[]}): _notices = notices;
  

 final  List<String> _notices;
@JsonKey() List<String> get notices {
  if (_notices is EqualUnmodifiableListView) return _notices;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_notices);
}


/// Create a copy of SharedNotebooksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharedNotebooksRefreshedEventCopyWith<SharedNotebooksRefreshedEvent> get copyWith => _$SharedNotebooksRefreshedEventCopyWithImpl<SharedNotebooksRefreshedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNotebooksRefreshedEvent&&const DeepCollectionEquality().equals(other._notices, _notices));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_notices));

@override
String toString() {
  return 'SharedNotebooksEvent.refreshed(notices: $notices)';
}


}

/// @nodoc
abstract mixin class $SharedNotebooksRefreshedEventCopyWith<$Res> implements $SharedNotebooksEventCopyWith<$Res> {
  factory $SharedNotebooksRefreshedEventCopyWith(SharedNotebooksRefreshedEvent value, $Res Function(SharedNotebooksRefreshedEvent) _then) = _$SharedNotebooksRefreshedEventCopyWithImpl;
@useResult
$Res call({
 List<String> notices
});




}
/// @nodoc
class _$SharedNotebooksRefreshedEventCopyWithImpl<$Res>
    implements $SharedNotebooksRefreshedEventCopyWith<$Res> {
  _$SharedNotebooksRefreshedEventCopyWithImpl(this._self, this._then);

  final SharedNotebooksRefreshedEvent _self;
  final $Res Function(SharedNotebooksRefreshedEvent) _then;

/// Create a copy of SharedNotebooksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notices = null,}) {
  return _then(SharedNotebooksRefreshedEvent(
notices: null == notices ? _self._notices : notices // ignore: cast_nullable_to_non_nullable
as List<String>,
  ));
}


}

/// @nodoc


class SharedNotebooksAcceptedEvent implements SharedNotebooksEvent {
  const SharedNotebooksAcceptedEvent(this.shareId);
  

 final  String shareId;

/// Create a copy of SharedNotebooksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharedNotebooksAcceptedEventCopyWith<SharedNotebooksAcceptedEvent> get copyWith => _$SharedNotebooksAcceptedEventCopyWithImpl<SharedNotebooksAcceptedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNotebooksAcceptedEvent&&(identical(other.shareId, shareId) || other.shareId == shareId));
}


@override
int get hashCode => Object.hash(runtimeType,shareId);

@override
String toString() {
  return 'SharedNotebooksEvent.accepted(shareId: $shareId)';
}


}

/// @nodoc
abstract mixin class $SharedNotebooksAcceptedEventCopyWith<$Res> implements $SharedNotebooksEventCopyWith<$Res> {
  factory $SharedNotebooksAcceptedEventCopyWith(SharedNotebooksAcceptedEvent value, $Res Function(SharedNotebooksAcceptedEvent) _then) = _$SharedNotebooksAcceptedEventCopyWithImpl;
@useResult
$Res call({
 String shareId
});




}
/// @nodoc
class _$SharedNotebooksAcceptedEventCopyWithImpl<$Res>
    implements $SharedNotebooksAcceptedEventCopyWith<$Res> {
  _$SharedNotebooksAcceptedEventCopyWithImpl(this._self, this._then);

  final SharedNotebooksAcceptedEvent _self;
  final $Res Function(SharedNotebooksAcceptedEvent) _then;

/// Create a copy of SharedNotebooksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? shareId = null,}) {
  return _then(SharedNotebooksAcceptedEvent(
null == shareId ? _self.shareId : shareId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SharedNotebooksDeclinedEvent implements SharedNotebooksEvent {
  const SharedNotebooksDeclinedEvent(this.shareId);
  

 final  String shareId;

/// Create a copy of SharedNotebooksEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharedNotebooksDeclinedEventCopyWith<SharedNotebooksDeclinedEvent> get copyWith => _$SharedNotebooksDeclinedEventCopyWithImpl<SharedNotebooksDeclinedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNotebooksDeclinedEvent&&(identical(other.shareId, shareId) || other.shareId == shareId));
}


@override
int get hashCode => Object.hash(runtimeType,shareId);

@override
String toString() {
  return 'SharedNotebooksEvent.declined(shareId: $shareId)';
}


}

/// @nodoc
abstract mixin class $SharedNotebooksDeclinedEventCopyWith<$Res> implements $SharedNotebooksEventCopyWith<$Res> {
  factory $SharedNotebooksDeclinedEventCopyWith(SharedNotebooksDeclinedEvent value, $Res Function(SharedNotebooksDeclinedEvent) _then) = _$SharedNotebooksDeclinedEventCopyWithImpl;
@useResult
$Res call({
 String shareId
});




}
/// @nodoc
class _$SharedNotebooksDeclinedEventCopyWithImpl<$Res>
    implements $SharedNotebooksDeclinedEventCopyWith<$Res> {
  _$SharedNotebooksDeclinedEventCopyWithImpl(this._self, this._then);

  final SharedNotebooksDeclinedEvent _self;
  final $Res Function(SharedNotebooksDeclinedEvent) _then;

/// Create a copy of SharedNotebooksEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? shareId = null,}) {
  return _then(SharedNotebooksDeclinedEvent(
null == shareId ? _self.shareId : shareId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SharedNotebooksNoticeReadEvent implements SharedNotebooksEvent {
  const SharedNotebooksNoticeReadEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNotebooksNoticeReadEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SharedNotebooksEvent.noticeRead()';
}


}




/// @nodoc
mixin _$SharedNotebooksState {

 LoadStatus get status; List<SharedBookRow> get books; List<BookInvitationRow> get invitations;/// Invitations being accepted or declined, by share ID.
 Set<String> get working;/// What to tell of notebooks that ended here, oldest first.
 List<String> get notices;/// "Update Kanso to open this shared notebook.": the sync server said
/// this Kanso is too old for shared notebooks. Null while it is happy.
 String? get updateLine;/// The notebook just joined, to open.
 String? get joined; ShareProblem? get problem;
/// Create a copy of SharedNotebooksState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SharedNotebooksStateCopyWith<SharedNotebooksState> get copyWith => _$SharedNotebooksStateCopyWithImpl<SharedNotebooksState>(this as SharedNotebooksState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SharedNotebooksState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.books, books)&&const DeepCollectionEquality().equals(other.invitations, invitations)&&const DeepCollectionEquality().equals(other.working, working)&&const DeepCollectionEquality().equals(other.notices, notices)&&(identical(other.updateLine, updateLine) || other.updateLine == updateLine)&&(identical(other.joined, joined) || other.joined == joined)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(books),const DeepCollectionEquality().hash(invitations),const DeepCollectionEquality().hash(working),const DeepCollectionEquality().hash(notices),updateLine,joined,problem);

@override
String toString() {
  return 'SharedNotebooksState(status: $status, books: $books, invitations: $invitations, working: $working, notices: $notices, updateLine: $updateLine, joined: $joined, problem: $problem)';
}


}

/// @nodoc
abstract mixin class $SharedNotebooksStateCopyWith<$Res>  {
  factory $SharedNotebooksStateCopyWith(SharedNotebooksState value, $Res Function(SharedNotebooksState) _then) = _$SharedNotebooksStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<SharedBookRow> books, List<BookInvitationRow> invitations, Set<String> working, List<String> notices, String? updateLine, String? joined, ShareProblem? problem
});


$ShareProblemCopyWith<$Res>? get problem;

}
/// @nodoc
class _$SharedNotebooksStateCopyWithImpl<$Res>
    implements $SharedNotebooksStateCopyWith<$Res> {
  _$SharedNotebooksStateCopyWithImpl(this._self, this._then);

  final SharedNotebooksState _self;
  final $Res Function(SharedNotebooksState) _then;

/// Create a copy of SharedNotebooksState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? books = null,Object? invitations = null,Object? working = null,Object? notices = null,Object? updateLine = freezed,Object? joined = freezed,Object? problem = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,books: null == books ? _self.books : books // ignore: cast_nullable_to_non_nullable
as List<SharedBookRow>,invitations: null == invitations ? _self.invitations : invitations // ignore: cast_nullable_to_non_nullable
as List<BookInvitationRow>,working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as Set<String>,notices: null == notices ? _self.notices : notices // ignore: cast_nullable_to_non_nullable
as List<String>,updateLine: freezed == updateLine ? _self.updateLine : updateLine // ignore: cast_nullable_to_non_nullable
as String?,joined: freezed == joined ? _self.joined : joined // ignore: cast_nullable_to_non_nullable
as String?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ShareProblem?,
  ));
}
/// Create a copy of SharedNotebooksState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ShareProblemCopyWith<$Res>? get problem {
    if (_self.problem == null) {
    return null;
  }

  return $ShareProblemCopyWith<$Res>(_self.problem!, (value) {
    return _then(_self.copyWith(problem: value));
  });
}
}


/// Adds pattern-matching-related methods to [SharedNotebooksState].
extension SharedNotebooksStatePatterns on SharedNotebooksState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _SharedNotebooksState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _SharedNotebooksState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _SharedNotebooksState value)  $default,){
final _that = this;
switch (_that) {
case _SharedNotebooksState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _SharedNotebooksState value)?  $default,){
final _that = this;
switch (_that) {
case _SharedNotebooksState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<SharedBookRow> books,  List<BookInvitationRow> invitations,  Set<String> working,  List<String> notices,  String? updateLine,  String? joined,  ShareProblem? problem)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _SharedNotebooksState() when $default != null:
return $default(_that.status,_that.books,_that.invitations,_that.working,_that.notices,_that.updateLine,_that.joined,_that.problem);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<SharedBookRow> books,  List<BookInvitationRow> invitations,  Set<String> working,  List<String> notices,  String? updateLine,  String? joined,  ShareProblem? problem)  $default,) {final _that = this;
switch (_that) {
case _SharedNotebooksState():
return $default(_that.status,_that.books,_that.invitations,_that.working,_that.notices,_that.updateLine,_that.joined,_that.problem);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<SharedBookRow> books,  List<BookInvitationRow> invitations,  Set<String> working,  List<String> notices,  String? updateLine,  String? joined,  ShareProblem? problem)?  $default,) {final _that = this;
switch (_that) {
case _SharedNotebooksState() when $default != null:
return $default(_that.status,_that.books,_that.invitations,_that.working,_that.notices,_that.updateLine,_that.joined,_that.problem);case _:
  return null;

}
}

}

/// @nodoc


class _SharedNotebooksState implements SharedNotebooksState {
  const _SharedNotebooksState({this.status = LoadStatus.loading, final  List<SharedBookRow> books = const <SharedBookRow>[], final  List<BookInvitationRow> invitations = const <BookInvitationRow>[], final  Set<String> working = const <String>{}, final  List<String> notices = const <String>[], this.updateLine, this.joined, this.problem}): _books = books,_invitations = invitations,_working = working,_notices = notices;
  

@override@JsonKey() final  LoadStatus status;
 final  List<SharedBookRow> _books;
@override@JsonKey() List<SharedBookRow> get books {
  if (_books is EqualUnmodifiableListView) return _books;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_books);
}

 final  List<BookInvitationRow> _invitations;
@override@JsonKey() List<BookInvitationRow> get invitations {
  if (_invitations is EqualUnmodifiableListView) return _invitations;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_invitations);
}

/// Invitations being accepted or declined, by share ID.
 final  Set<String> _working;
/// Invitations being accepted or declined, by share ID.
@override@JsonKey() Set<String> get working {
  if (_working is EqualUnmodifiableSetView) return _working;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_working);
}

/// What to tell of notebooks that ended here, oldest first.
 final  List<String> _notices;
/// What to tell of notebooks that ended here, oldest first.
@override@JsonKey() List<String> get notices {
  if (_notices is EqualUnmodifiableListView) return _notices;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_notices);
}

/// "Update Kanso to open this shared notebook.": the sync server said
/// this Kanso is too old for shared notebooks. Null while it is happy.
@override final  String? updateLine;
/// The notebook just joined, to open.
@override final  String? joined;
@override final  ShareProblem? problem;

/// Create a copy of SharedNotebooksState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$SharedNotebooksStateCopyWith<_SharedNotebooksState> get copyWith => __$SharedNotebooksStateCopyWithImpl<_SharedNotebooksState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _SharedNotebooksState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._books, _books)&&const DeepCollectionEquality().equals(other._invitations, _invitations)&&const DeepCollectionEquality().equals(other._working, _working)&&const DeepCollectionEquality().equals(other._notices, _notices)&&(identical(other.updateLine, updateLine) || other.updateLine == updateLine)&&(identical(other.joined, joined) || other.joined == joined)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_books),const DeepCollectionEquality().hash(_invitations),const DeepCollectionEquality().hash(_working),const DeepCollectionEquality().hash(_notices),updateLine,joined,problem);

@override
String toString() {
  return 'SharedNotebooksState(status: $status, books: $books, invitations: $invitations, working: $working, notices: $notices, updateLine: $updateLine, joined: $joined, problem: $problem)';
}


}

/// @nodoc
abstract mixin class _$SharedNotebooksStateCopyWith<$Res> implements $SharedNotebooksStateCopyWith<$Res> {
  factory _$SharedNotebooksStateCopyWith(_SharedNotebooksState value, $Res Function(_SharedNotebooksState) _then) = __$SharedNotebooksStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<SharedBookRow> books, List<BookInvitationRow> invitations, Set<String> working, List<String> notices, String? updateLine, String? joined, ShareProblem? problem
});


@override $ShareProblemCopyWith<$Res>? get problem;

}
/// @nodoc
class __$SharedNotebooksStateCopyWithImpl<$Res>
    implements _$SharedNotebooksStateCopyWith<$Res> {
  __$SharedNotebooksStateCopyWithImpl(this._self, this._then);

  final _SharedNotebooksState _self;
  final $Res Function(_SharedNotebooksState) _then;

/// Create a copy of SharedNotebooksState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? books = null,Object? invitations = null,Object? working = null,Object? notices = null,Object? updateLine = freezed,Object? joined = freezed,Object? problem = freezed,}) {
  return _then(_SharedNotebooksState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,books: null == books ? _self._books : books // ignore: cast_nullable_to_non_nullable
as List<SharedBookRow>,invitations: null == invitations ? _self._invitations : invitations // ignore: cast_nullable_to_non_nullable
as List<BookInvitationRow>,working: null == working ? _self._working : working // ignore: cast_nullable_to_non_nullable
as Set<String>,notices: null == notices ? _self._notices : notices // ignore: cast_nullable_to_non_nullable
as List<String>,updateLine: freezed == updateLine ? _self.updateLine : updateLine // ignore: cast_nullable_to_non_nullable
as String?,joined: freezed == joined ? _self.joined : joined // ignore: cast_nullable_to_non_nullable
as String?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ShareProblem?,
  ));
}

/// Create a copy of SharedNotebooksState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ShareProblemCopyWith<$Res>? get problem {
    if (_self.problem == null) {
    return null;
  }

  return $ShareProblemCopyWith<$Res>(_self.problem!, (value) {
    return _then(_self.copyWith(problem: value));
  });
}
}

// dart format on

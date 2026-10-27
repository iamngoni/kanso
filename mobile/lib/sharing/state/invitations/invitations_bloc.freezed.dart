// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'invitations_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$InvitationsEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InvitationsEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'InvitationsEvent()';
}


}

/// @nodoc
class $InvitationsEventCopyWith<$Res>  {
$InvitationsEventCopyWith(InvitationsEvent _, $Res Function(InvitationsEvent) __);
}


/// Adds pattern-matching-related methods to [InvitationsEvent].
extension InvitationsEventPatterns on InvitationsEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( InvitationsStartedEvent value)?  started,TResult Function( InvitationsRefreshedEvent value)?  refreshed,TResult Function( InvitationsAcceptedEvent value)?  accepted,TResult Function( InvitationsDeclinedEvent value)?  declined,required TResult orElse(),}){
final _that = this;
switch (_that) {
case InvitationsStartedEvent() when started != null:
return started(_that);case InvitationsRefreshedEvent() when refreshed != null:
return refreshed(_that);case InvitationsAcceptedEvent() when accepted != null:
return accepted(_that);case InvitationsDeclinedEvent() when declined != null:
return declined(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( InvitationsStartedEvent value)  started,required TResult Function( InvitationsRefreshedEvent value)  refreshed,required TResult Function( InvitationsAcceptedEvent value)  accepted,required TResult Function( InvitationsDeclinedEvent value)  declined,}){
final _that = this;
switch (_that) {
case InvitationsStartedEvent():
return started(_that);case InvitationsRefreshedEvent():
return refreshed(_that);case InvitationsAcceptedEvent():
return accepted(_that);case InvitationsDeclinedEvent():
return declined(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( InvitationsStartedEvent value)?  started,TResult? Function( InvitationsRefreshedEvent value)?  refreshed,TResult? Function( InvitationsAcceptedEvent value)?  accepted,TResult? Function( InvitationsDeclinedEvent value)?  declined,}){
final _that = this;
switch (_that) {
case InvitationsStartedEvent() when started != null:
return started(_that);case InvitationsRefreshedEvent() when refreshed != null:
return refreshed(_that);case InvitationsAcceptedEvent() when accepted != null:
return accepted(_that);case InvitationsDeclinedEvent() when declined != null:
return declined(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function()?  refreshed,TResult Function( String shareId)?  accepted,TResult Function( String shareId)?  declined,required TResult orElse(),}) {final _that = this;
switch (_that) {
case InvitationsStartedEvent() when started != null:
return started();case InvitationsRefreshedEvent() when refreshed != null:
return refreshed();case InvitationsAcceptedEvent() when accepted != null:
return accepted(_that.shareId);case InvitationsDeclinedEvent() when declined != null:
return declined(_that.shareId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function()  refreshed,required TResult Function( String shareId)  accepted,required TResult Function( String shareId)  declined,}) {final _that = this;
switch (_that) {
case InvitationsStartedEvent():
return started();case InvitationsRefreshedEvent():
return refreshed();case InvitationsAcceptedEvent():
return accepted(_that.shareId);case InvitationsDeclinedEvent():
return declined(_that.shareId);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function()?  refreshed,TResult? Function( String shareId)?  accepted,TResult? Function( String shareId)?  declined,}) {final _that = this;
switch (_that) {
case InvitationsStartedEvent() when started != null:
return started();case InvitationsRefreshedEvent() when refreshed != null:
return refreshed();case InvitationsAcceptedEvent() when accepted != null:
return accepted(_that.shareId);case InvitationsDeclinedEvent() when declined != null:
return declined(_that.shareId);case _:
  return null;

}
}

}

/// @nodoc


class InvitationsStartedEvent implements InvitationsEvent {
  const InvitationsStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InvitationsStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'InvitationsEvent.started()';
}


}




/// @nodoc


class InvitationsRefreshedEvent implements InvitationsEvent {
  const InvitationsRefreshedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InvitationsRefreshedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'InvitationsEvent.refreshed()';
}


}




/// @nodoc


class InvitationsAcceptedEvent implements InvitationsEvent {
  const InvitationsAcceptedEvent(this.shareId);
  

 final  String shareId;

/// Create a copy of InvitationsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InvitationsAcceptedEventCopyWith<InvitationsAcceptedEvent> get copyWith => _$InvitationsAcceptedEventCopyWithImpl<InvitationsAcceptedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InvitationsAcceptedEvent&&(identical(other.shareId, shareId) || other.shareId == shareId));
}


@override
int get hashCode => Object.hash(runtimeType,shareId);

@override
String toString() {
  return 'InvitationsEvent.accepted(shareId: $shareId)';
}


}

/// @nodoc
abstract mixin class $InvitationsAcceptedEventCopyWith<$Res> implements $InvitationsEventCopyWith<$Res> {
  factory $InvitationsAcceptedEventCopyWith(InvitationsAcceptedEvent value, $Res Function(InvitationsAcceptedEvent) _then) = _$InvitationsAcceptedEventCopyWithImpl;
@useResult
$Res call({
 String shareId
});




}
/// @nodoc
class _$InvitationsAcceptedEventCopyWithImpl<$Res>
    implements $InvitationsAcceptedEventCopyWith<$Res> {
  _$InvitationsAcceptedEventCopyWithImpl(this._self, this._then);

  final InvitationsAcceptedEvent _self;
  final $Res Function(InvitationsAcceptedEvent) _then;

/// Create a copy of InvitationsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? shareId = null,}) {
  return _then(InvitationsAcceptedEvent(
null == shareId ? _self.shareId : shareId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class InvitationsDeclinedEvent implements InvitationsEvent {
  const InvitationsDeclinedEvent(this.shareId);
  

 final  String shareId;

/// Create a copy of InvitationsEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InvitationsDeclinedEventCopyWith<InvitationsDeclinedEvent> get copyWith => _$InvitationsDeclinedEventCopyWithImpl<InvitationsDeclinedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InvitationsDeclinedEvent&&(identical(other.shareId, shareId) || other.shareId == shareId));
}


@override
int get hashCode => Object.hash(runtimeType,shareId);

@override
String toString() {
  return 'InvitationsEvent.declined(shareId: $shareId)';
}


}

/// @nodoc
abstract mixin class $InvitationsDeclinedEventCopyWith<$Res> implements $InvitationsEventCopyWith<$Res> {
  factory $InvitationsDeclinedEventCopyWith(InvitationsDeclinedEvent value, $Res Function(InvitationsDeclinedEvent) _then) = _$InvitationsDeclinedEventCopyWithImpl;
@useResult
$Res call({
 String shareId
});




}
/// @nodoc
class _$InvitationsDeclinedEventCopyWithImpl<$Res>
    implements $InvitationsDeclinedEventCopyWith<$Res> {
  _$InvitationsDeclinedEventCopyWithImpl(this._self, this._then);

  final InvitationsDeclinedEvent _self;
  final $Res Function(InvitationsDeclinedEvent) _then;

/// Create a copy of InvitationsEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? shareId = null,}) {
  return _then(InvitationsDeclinedEvent(
null == shareId ? _self.shareId : shareId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$InvitationsState {

 LoadStatus get status;/// The notes shared with this account that wait to be accepted, the
/// oldest first.
 List<InvitationRow> get invitations;/// No account is signed in, so none can be shared with it.
 bool get signedOut;/// The invitations being answered now, by share id.
 Set<String> get working;/// Why the last answer did not happen.
 ShareProblem? get problem;/// The note an invitation just accepted brought in, to open.
 String? get opened;
/// Create a copy of InvitationsState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$InvitationsStateCopyWith<InvitationsState> get copyWith => _$InvitationsStateCopyWithImpl<InvitationsState>(this as InvitationsState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is InvitationsState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other.invitations, invitations)&&(identical(other.signedOut, signedOut) || other.signedOut == signedOut)&&const DeepCollectionEquality().equals(other.working, working)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.opened, opened) || other.opened == opened));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(invitations),signedOut,const DeepCollectionEquality().hash(working),problem,opened);

@override
String toString() {
  return 'InvitationsState(status: $status, invitations: $invitations, signedOut: $signedOut, working: $working, problem: $problem, opened: $opened)';
}


}

/// @nodoc
abstract mixin class $InvitationsStateCopyWith<$Res>  {
  factory $InvitationsStateCopyWith(InvitationsState value, $Res Function(InvitationsState) _then) = _$InvitationsStateCopyWithImpl;
@useResult
$Res call({
 LoadStatus status, List<InvitationRow> invitations, bool signedOut, Set<String> working, ShareProblem? problem, String? opened
});


$ShareProblemCopyWith<$Res>? get problem;

}
/// @nodoc
class _$InvitationsStateCopyWithImpl<$Res>
    implements $InvitationsStateCopyWith<$Res> {
  _$InvitationsStateCopyWithImpl(this._self, this._then);

  final InvitationsState _self;
  final $Res Function(InvitationsState) _then;

/// Create a copy of InvitationsState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? status = null,Object? invitations = null,Object? signedOut = null,Object? working = null,Object? problem = freezed,Object? opened = freezed,}) {
  return _then(_self.copyWith(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,invitations: null == invitations ? _self.invitations : invitations // ignore: cast_nullable_to_non_nullable
as List<InvitationRow>,signedOut: null == signedOut ? _self.signedOut : signedOut // ignore: cast_nullable_to_non_nullable
as bool,working: null == working ? _self.working : working // ignore: cast_nullable_to_non_nullable
as Set<String>,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ShareProblem?,opened: freezed == opened ? _self.opened : opened // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}
/// Create a copy of InvitationsState
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


/// Adds pattern-matching-related methods to [InvitationsState].
extension InvitationsStatePatterns on InvitationsState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _InvitationsState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _InvitationsState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _InvitationsState value)  $default,){
final _that = this;
switch (_that) {
case _InvitationsState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _InvitationsState value)?  $default,){
final _that = this;
switch (_that) {
case _InvitationsState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( LoadStatus status,  List<InvitationRow> invitations,  bool signedOut,  Set<String> working,  ShareProblem? problem,  String? opened)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _InvitationsState() when $default != null:
return $default(_that.status,_that.invitations,_that.signedOut,_that.working,_that.problem,_that.opened);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( LoadStatus status,  List<InvitationRow> invitations,  bool signedOut,  Set<String> working,  ShareProblem? problem,  String? opened)  $default,) {final _that = this;
switch (_that) {
case _InvitationsState():
return $default(_that.status,_that.invitations,_that.signedOut,_that.working,_that.problem,_that.opened);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( LoadStatus status,  List<InvitationRow> invitations,  bool signedOut,  Set<String> working,  ShareProblem? problem,  String? opened)?  $default,) {final _that = this;
switch (_that) {
case _InvitationsState() when $default != null:
return $default(_that.status,_that.invitations,_that.signedOut,_that.working,_that.problem,_that.opened);case _:
  return null;

}
}

}

/// @nodoc


class _InvitationsState implements InvitationsState {
  const _InvitationsState({this.status = LoadStatus.loading, final  List<InvitationRow> invitations = const <InvitationRow>[], this.signedOut = false, final  Set<String> working = const <String>{}, this.problem, this.opened}): _invitations = invitations,_working = working;
  

@override@JsonKey() final  LoadStatus status;
/// The notes shared with this account that wait to be accepted, the
/// oldest first.
 final  List<InvitationRow> _invitations;
/// The notes shared with this account that wait to be accepted, the
/// oldest first.
@override@JsonKey() List<InvitationRow> get invitations {
  if (_invitations is EqualUnmodifiableListView) return _invitations;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_invitations);
}

/// No account is signed in, so none can be shared with it.
@override@JsonKey() final  bool signedOut;
/// The invitations being answered now, by share id.
 final  Set<String> _working;
/// The invitations being answered now, by share id.
@override@JsonKey() Set<String> get working {
  if (_working is EqualUnmodifiableSetView) return _working;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableSetView(_working);
}

/// Why the last answer did not happen.
@override final  ShareProblem? problem;
/// The note an invitation just accepted brought in, to open.
@override final  String? opened;

/// Create a copy of InvitationsState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$InvitationsStateCopyWith<_InvitationsState> get copyWith => __$InvitationsStateCopyWithImpl<_InvitationsState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _InvitationsState&&(identical(other.status, status) || other.status == status)&&const DeepCollectionEquality().equals(other._invitations, _invitations)&&(identical(other.signedOut, signedOut) || other.signedOut == signedOut)&&const DeepCollectionEquality().equals(other._working, _working)&&(identical(other.problem, problem) || other.problem == problem)&&(identical(other.opened, opened) || other.opened == opened));
}


@override
int get hashCode => Object.hash(runtimeType,status,const DeepCollectionEquality().hash(_invitations),signedOut,const DeepCollectionEquality().hash(_working),problem,opened);

@override
String toString() {
  return 'InvitationsState(status: $status, invitations: $invitations, signedOut: $signedOut, working: $working, problem: $problem, opened: $opened)';
}


}

/// @nodoc
abstract mixin class _$InvitationsStateCopyWith<$Res> implements $InvitationsStateCopyWith<$Res> {
  factory _$InvitationsStateCopyWith(_InvitationsState value, $Res Function(_InvitationsState) _then) = __$InvitationsStateCopyWithImpl;
@override @useResult
$Res call({
 LoadStatus status, List<InvitationRow> invitations, bool signedOut, Set<String> working, ShareProblem? problem, String? opened
});


@override $ShareProblemCopyWith<$Res>? get problem;

}
/// @nodoc
class __$InvitationsStateCopyWithImpl<$Res>
    implements _$InvitationsStateCopyWith<$Res> {
  __$InvitationsStateCopyWithImpl(this._self, this._then);

  final _InvitationsState _self;
  final $Res Function(_InvitationsState) _then;

/// Create a copy of InvitationsState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? status = null,Object? invitations = null,Object? signedOut = null,Object? working = null,Object? problem = freezed,Object? opened = freezed,}) {
  return _then(_InvitationsState(
status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as LoadStatus,invitations: null == invitations ? _self._invitations : invitations // ignore: cast_nullable_to_non_nullable
as List<InvitationRow>,signedOut: null == signedOut ? _self.signedOut : signedOut // ignore: cast_nullable_to_non_nullable
as bool,working: null == working ? _self._working : working // ignore: cast_nullable_to_non_nullable
as Set<String>,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ShareProblem?,opened: freezed == opened ? _self.opened : opened // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of InvitationsState
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

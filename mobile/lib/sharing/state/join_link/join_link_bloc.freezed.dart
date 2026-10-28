// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'join_link_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$JoinLinkEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JoinLinkEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'JoinLinkEvent()';
}


}

/// @nodoc
class $JoinLinkEventCopyWith<$Res>  {
$JoinLinkEventCopyWith(JoinLinkEvent _, $Res Function(JoinLinkEvent) __);
}


/// Adds pattern-matching-related methods to [JoinLinkEvent].
extension JoinLinkEventPatterns on JoinLinkEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( JoinLinkSubmittedEvent value)?  submitted,TResult Function( JoinLinkRestartedEvent value)?  restarted,required TResult orElse(),}){
final _that = this;
switch (_that) {
case JoinLinkSubmittedEvent() when submitted != null:
return submitted(_that);case JoinLinkRestartedEvent() when restarted != null:
return restarted(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( JoinLinkSubmittedEvent value)  submitted,required TResult Function( JoinLinkRestartedEvent value)  restarted,}){
final _that = this;
switch (_that) {
case JoinLinkSubmittedEvent():
return submitted(_that);case JoinLinkRestartedEvent():
return restarted(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( JoinLinkSubmittedEvent value)?  submitted,TResult? Function( JoinLinkRestartedEvent value)?  restarted,}){
final _that = this;
switch (_that) {
case JoinLinkSubmittedEvent() when submitted != null:
return submitted(_that);case JoinLinkRestartedEvent() when restarted != null:
return restarted(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String link)?  submitted,TResult Function()?  restarted,required TResult orElse(),}) {final _that = this;
switch (_that) {
case JoinLinkSubmittedEvent() when submitted != null:
return submitted(_that.link);case JoinLinkRestartedEvent() when restarted != null:
return restarted();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String link)  submitted,required TResult Function()  restarted,}) {final _that = this;
switch (_that) {
case JoinLinkSubmittedEvent():
return submitted(_that.link);case JoinLinkRestartedEvent():
return restarted();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String link)?  submitted,TResult? Function()?  restarted,}) {final _that = this;
switch (_that) {
case JoinLinkSubmittedEvent() when submitted != null:
return submitted(_that.link);case JoinLinkRestartedEvent() when restarted != null:
return restarted();case _:
  return null;

}
}

}

/// @nodoc


class JoinLinkSubmittedEvent implements JoinLinkEvent {
  const JoinLinkSubmittedEvent(this.link);
  

 final  String link;

/// Create a copy of JoinLinkEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$JoinLinkSubmittedEventCopyWith<JoinLinkSubmittedEvent> get copyWith => _$JoinLinkSubmittedEventCopyWithImpl<JoinLinkSubmittedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JoinLinkSubmittedEvent&&(identical(other.link, link) || other.link == link));
}


@override
int get hashCode => Object.hash(runtimeType,link);

@override
String toString() {
  return 'JoinLinkEvent.submitted(link: $link)';
}


}

/// @nodoc
abstract mixin class $JoinLinkSubmittedEventCopyWith<$Res> implements $JoinLinkEventCopyWith<$Res> {
  factory $JoinLinkSubmittedEventCopyWith(JoinLinkSubmittedEvent value, $Res Function(JoinLinkSubmittedEvent) _then) = _$JoinLinkSubmittedEventCopyWithImpl;
@useResult
$Res call({
 String link
});




}
/// @nodoc
class _$JoinLinkSubmittedEventCopyWithImpl<$Res>
    implements $JoinLinkSubmittedEventCopyWith<$Res> {
  _$JoinLinkSubmittedEventCopyWithImpl(this._self, this._then);

  final JoinLinkSubmittedEvent _self;
  final $Res Function(JoinLinkSubmittedEvent) _then;

/// Create a copy of JoinLinkEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? link = null,}) {
  return _then(JoinLinkSubmittedEvent(
null == link ? _self.link : link // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class JoinLinkRestartedEvent implements JoinLinkEvent {
  const JoinLinkRestartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JoinLinkRestartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'JoinLinkEvent.restarted()';
}


}




/// @nodoc
mixin _$JoinLinkState {

/// The request is being sent.
 bool get joining;/// Who was asked to let the person in, once the request is sent.
 JoinRow? get asked;/// Why the request was not sent.
 ShareProblem? get problem;
/// Create a copy of JoinLinkState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$JoinLinkStateCopyWith<JoinLinkState> get copyWith => _$JoinLinkStateCopyWithImpl<JoinLinkState>(this as JoinLinkState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is JoinLinkState&&(identical(other.joining, joining) || other.joining == joining)&&(identical(other.asked, asked) || other.asked == asked)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,joining,asked,problem);

@override
String toString() {
  return 'JoinLinkState(joining: $joining, asked: $asked, problem: $problem)';
}


}

/// @nodoc
abstract mixin class $JoinLinkStateCopyWith<$Res>  {
  factory $JoinLinkStateCopyWith(JoinLinkState value, $Res Function(JoinLinkState) _then) = _$JoinLinkStateCopyWithImpl;
@useResult
$Res call({
 bool joining, JoinRow? asked, ShareProblem? problem
});


$ShareProblemCopyWith<$Res>? get problem;

}
/// @nodoc
class _$JoinLinkStateCopyWithImpl<$Res>
    implements $JoinLinkStateCopyWith<$Res> {
  _$JoinLinkStateCopyWithImpl(this._self, this._then);

  final JoinLinkState _self;
  final $Res Function(JoinLinkState) _then;

/// Create a copy of JoinLinkState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? joining = null,Object? asked = freezed,Object? problem = freezed,}) {
  return _then(_self.copyWith(
joining: null == joining ? _self.joining : joining // ignore: cast_nullable_to_non_nullable
as bool,asked: freezed == asked ? _self.asked : asked // ignore: cast_nullable_to_non_nullable
as JoinRow?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ShareProblem?,
  ));
}
/// Create a copy of JoinLinkState
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


/// Adds pattern-matching-related methods to [JoinLinkState].
extension JoinLinkStatePatterns on JoinLinkState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _JoinLinkState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _JoinLinkState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _JoinLinkState value)  $default,){
final _that = this;
switch (_that) {
case _JoinLinkState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _JoinLinkState value)?  $default,){
final _that = this;
switch (_that) {
case _JoinLinkState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( bool joining,  JoinRow? asked,  ShareProblem? problem)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _JoinLinkState() when $default != null:
return $default(_that.joining,_that.asked,_that.problem);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( bool joining,  JoinRow? asked,  ShareProblem? problem)  $default,) {final _that = this;
switch (_that) {
case _JoinLinkState():
return $default(_that.joining,_that.asked,_that.problem);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( bool joining,  JoinRow? asked,  ShareProblem? problem)?  $default,) {final _that = this;
switch (_that) {
case _JoinLinkState() when $default != null:
return $default(_that.joining,_that.asked,_that.problem);case _:
  return null;

}
}

}

/// @nodoc


class _JoinLinkState implements JoinLinkState {
  const _JoinLinkState({this.joining = false, this.asked, this.problem});
  

/// The request is being sent.
@override@JsonKey() final  bool joining;
/// Who was asked to let the person in, once the request is sent.
@override final  JoinRow? asked;
/// Why the request was not sent.
@override final  ShareProblem? problem;

/// Create a copy of JoinLinkState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$JoinLinkStateCopyWith<_JoinLinkState> get copyWith => __$JoinLinkStateCopyWithImpl<_JoinLinkState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _JoinLinkState&&(identical(other.joining, joining) || other.joining == joining)&&(identical(other.asked, asked) || other.asked == asked)&&(identical(other.problem, problem) || other.problem == problem));
}


@override
int get hashCode => Object.hash(runtimeType,joining,asked,problem);

@override
String toString() {
  return 'JoinLinkState(joining: $joining, asked: $asked, problem: $problem)';
}


}

/// @nodoc
abstract mixin class _$JoinLinkStateCopyWith<$Res> implements $JoinLinkStateCopyWith<$Res> {
  factory _$JoinLinkStateCopyWith(_JoinLinkState value, $Res Function(_JoinLinkState) _then) = __$JoinLinkStateCopyWithImpl;
@override @useResult
$Res call({
 bool joining, JoinRow? asked, ShareProblem? problem
});


@override $ShareProblemCopyWith<$Res>? get problem;

}
/// @nodoc
class __$JoinLinkStateCopyWithImpl<$Res>
    implements _$JoinLinkStateCopyWith<$Res> {
  __$JoinLinkStateCopyWithImpl(this._self, this._then);

  final _JoinLinkState _self;
  final $Res Function(_JoinLinkState) _then;

/// Create a copy of JoinLinkState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? joining = null,Object? asked = freezed,Object? problem = freezed,}) {
  return _then(_JoinLinkState(
joining: null == joining ? _self.joining : joining // ignore: cast_nullable_to_non_nullable
as bool,asked: freezed == asked ? _self.asked : asked // ignore: cast_nullable_to_non_nullable
as JoinRow?,problem: freezed == problem ? _self.problem : problem // ignore: cast_nullable_to_non_nullable
as ShareProblem?,
  ));
}

/// Create a copy of JoinLinkState
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

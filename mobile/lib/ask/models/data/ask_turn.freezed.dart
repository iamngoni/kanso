// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ask_turn.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AskTurn {

 String get question;/// The answer as Markdown, with citations as links to notes.
 String get answer;/// The notes the answer was written from.
 List<AiNoteRef> get used; AskOutcome get outcome;/// Why the question failed, in plain words, while [outcome] is failed.
 String? get failure;
/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskTurnCopyWith<AskTurn> get copyWith => _$AskTurnCopyWithImpl<AskTurn>(this as AskTurn, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskTurn&&(identical(other.question, question) || other.question == question)&&(identical(other.answer, answer) || other.answer == answer)&&const DeepCollectionEquality().equals(other.used, used)&&(identical(other.outcome, outcome) || other.outcome == outcome)&&(identical(other.failure, failure) || other.failure == failure));
}


@override
int get hashCode => Object.hash(runtimeType,question,answer,const DeepCollectionEquality().hash(used),outcome,failure);

@override
String toString() {
  return 'AskTurn(question: $question, answer: $answer, used: $used, outcome: $outcome, failure: $failure)';
}


}

/// @nodoc
abstract mixin class $AskTurnCopyWith<$Res>  {
  factory $AskTurnCopyWith(AskTurn value, $Res Function(AskTurn) _then) = _$AskTurnCopyWithImpl;
@useResult
$Res call({
 String question, String answer, List<AiNoteRef> used, AskOutcome outcome, String? failure
});




}
/// @nodoc
class _$AskTurnCopyWithImpl<$Res>
    implements $AskTurnCopyWith<$Res> {
  _$AskTurnCopyWithImpl(this._self, this._then);

  final AskTurn _self;
  final $Res Function(AskTurn) _then;

/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? question = null,Object? answer = null,Object? used = null,Object? outcome = null,Object? failure = freezed,}) {
  return _then(_self.copyWith(
question: null == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as String,answer: null == answer ? _self.answer : answer // ignore: cast_nullable_to_non_nullable
as String,used: null == used ? _self.used : used // ignore: cast_nullable_to_non_nullable
as List<AiNoteRef>,outcome: null == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AskOutcome,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

}


/// Adds pattern-matching-related methods to [AskTurn].
extension AskTurnPatterns on AskTurn {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AskTurn value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AskTurn() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AskTurn value)  $default,){
final _that = this;
switch (_that) {
case _AskTurn():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AskTurn value)?  $default,){
final _that = this;
switch (_that) {
case _AskTurn() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String question,  String answer,  List<AiNoteRef> used,  AskOutcome outcome,  String? failure)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AskTurn() when $default != null:
return $default(_that.question,_that.answer,_that.used,_that.outcome,_that.failure);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String question,  String answer,  List<AiNoteRef> used,  AskOutcome outcome,  String? failure)  $default,) {final _that = this;
switch (_that) {
case _AskTurn():
return $default(_that.question,_that.answer,_that.used,_that.outcome,_that.failure);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String question,  String answer,  List<AiNoteRef> used,  AskOutcome outcome,  String? failure)?  $default,) {final _that = this;
switch (_that) {
case _AskTurn() when $default != null:
return $default(_that.question,_that.answer,_that.used,_that.outcome,_that.failure);case _:
  return null;

}
}

}

/// @nodoc


class _AskTurn implements AskTurn {
  const _AskTurn({required this.question, this.answer = '', final  List<AiNoteRef> used = const <AiNoteRef>[], this.outcome = AskOutcome.pending, this.failure}): _used = used;
  

@override final  String question;
/// The answer as Markdown, with citations as links to notes.
@override@JsonKey() final  String answer;
/// The notes the answer was written from.
 final  List<AiNoteRef> _used;
/// The notes the answer was written from.
@override@JsonKey() List<AiNoteRef> get used {
  if (_used is EqualUnmodifiableListView) return _used;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_used);
}

@override@JsonKey() final  AskOutcome outcome;
/// Why the question failed, in plain words, while [outcome] is failed.
@override final  String? failure;

/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AskTurnCopyWith<_AskTurn> get copyWith => __$AskTurnCopyWithImpl<_AskTurn>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AskTurn&&(identical(other.question, question) || other.question == question)&&(identical(other.answer, answer) || other.answer == answer)&&const DeepCollectionEquality().equals(other._used, _used)&&(identical(other.outcome, outcome) || other.outcome == outcome)&&(identical(other.failure, failure) || other.failure == failure));
}


@override
int get hashCode => Object.hash(runtimeType,question,answer,const DeepCollectionEquality().hash(_used),outcome,failure);

@override
String toString() {
  return 'AskTurn(question: $question, answer: $answer, used: $used, outcome: $outcome, failure: $failure)';
}


}

/// @nodoc
abstract mixin class _$AskTurnCopyWith<$Res> implements $AskTurnCopyWith<$Res> {
  factory _$AskTurnCopyWith(_AskTurn value, $Res Function(_AskTurn) _then) = __$AskTurnCopyWithImpl;
@override @useResult
$Res call({
 String question, String answer, List<AiNoteRef> used, AskOutcome outcome, String? failure
});




}
/// @nodoc
class __$AskTurnCopyWithImpl<$Res>
    implements _$AskTurnCopyWith<$Res> {
  __$AskTurnCopyWithImpl(this._self, this._then);

  final _AskTurn _self;
  final $Res Function(_AskTurn) _then;

/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? question = null,Object? answer = null,Object? used = null,Object? outcome = null,Object? failure = freezed,}) {
  return _then(_AskTurn(
question: null == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as String,answer: null == answer ? _self.answer : answer // ignore: cast_nullable_to_non_nullable
as String,used: null == used ? _self._used : used // ignore: cast_nullable_to_non_nullable
as List<AiNoteRef>,outcome: null == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AskOutcome,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

// dart format on

// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'agent_review.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AgentReview {

 String get header; String get fromAgent; List<String> get warnings; List<AgentCard> get cards;
/// Create a copy of AgentReview
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AgentReviewCopyWith<AgentReview> get copyWith => _$AgentReviewCopyWithImpl<AgentReview>(this as AgentReview, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AgentReview&&(identical(other.header, header) || other.header == header)&&(identical(other.fromAgent, fromAgent) || other.fromAgent == fromAgent)&&const DeepCollectionEquality().equals(other.warnings, warnings)&&const DeepCollectionEquality().equals(other.cards, cards));
}


@override
int get hashCode => Object.hash(runtimeType,header,fromAgent,const DeepCollectionEquality().hash(warnings),const DeepCollectionEquality().hash(cards));

@override
String toString() {
  return 'AgentReview(header: $header, fromAgent: $fromAgent, warnings: $warnings, cards: $cards)';
}


}

/// @nodoc
abstract mixin class $AgentReviewCopyWith<$Res>  {
  factory $AgentReviewCopyWith(AgentReview value, $Res Function(AgentReview) _then) = _$AgentReviewCopyWithImpl;
@useResult
$Res call({
 String header, String fromAgent, List<String> warnings, List<AgentCard> cards
});




}
/// @nodoc
class _$AgentReviewCopyWithImpl<$Res>
    implements $AgentReviewCopyWith<$Res> {
  _$AgentReviewCopyWithImpl(this._self, this._then);

  final AgentReview _self;
  final $Res Function(AgentReview) _then;

/// Create a copy of AgentReview
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? header = null,Object? fromAgent = null,Object? warnings = null,Object? cards = null,}) {
  return _then(_self.copyWith(
header: null == header ? _self.header : header // ignore: cast_nullable_to_non_nullable
as String,fromAgent: null == fromAgent ? _self.fromAgent : fromAgent // ignore: cast_nullable_to_non_nullable
as String,warnings: null == warnings ? _self.warnings : warnings // ignore: cast_nullable_to_non_nullable
as List<String>,cards: null == cards ? _self.cards : cards // ignore: cast_nullable_to_non_nullable
as List<AgentCard>,
  ));
}

}


/// Adds pattern-matching-related methods to [AgentReview].
extension AgentReviewPatterns on AgentReview {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AgentReview value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AgentReview() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AgentReview value)  $default,){
final _that = this;
switch (_that) {
case _AgentReview():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AgentReview value)?  $default,){
final _that = this;
switch (_that) {
case _AgentReview() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String header,  String fromAgent,  List<String> warnings,  List<AgentCard> cards)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AgentReview() when $default != null:
return $default(_that.header,_that.fromAgent,_that.warnings,_that.cards);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String header,  String fromAgent,  List<String> warnings,  List<AgentCard> cards)  $default,) {final _that = this;
switch (_that) {
case _AgentReview():
return $default(_that.header,_that.fromAgent,_that.warnings,_that.cards);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String header,  String fromAgent,  List<String> warnings,  List<AgentCard> cards)?  $default,) {final _that = this;
switch (_that) {
case _AgentReview() when $default != null:
return $default(_that.header,_that.fromAgent,_that.warnings,_that.cards);case _:
  return null;

}
}

}

/// @nodoc


class _AgentReview implements AgentReview {
  const _AgentReview({required this.header, this.fromAgent = '', final  List<String> warnings = const <String>[], final  List<AgentCard> cards = const <AgentCard>[]}): _warnings = warnings,_cards = cards;
  

@override final  String header;
@override@JsonKey() final  String fromAgent;
 final  List<String> _warnings;
@override@JsonKey() List<String> get warnings {
  if (_warnings is EqualUnmodifiableListView) return _warnings;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_warnings);
}

 final  List<AgentCard> _cards;
@override@JsonKey() List<AgentCard> get cards {
  if (_cards is EqualUnmodifiableListView) return _cards;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_cards);
}


/// Create a copy of AgentReview
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AgentReviewCopyWith<_AgentReview> get copyWith => __$AgentReviewCopyWithImpl<_AgentReview>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AgentReview&&(identical(other.header, header) || other.header == header)&&(identical(other.fromAgent, fromAgent) || other.fromAgent == fromAgent)&&const DeepCollectionEquality().equals(other._warnings, _warnings)&&const DeepCollectionEquality().equals(other._cards, _cards));
}


@override
int get hashCode => Object.hash(runtimeType,header,fromAgent,const DeepCollectionEquality().hash(_warnings),const DeepCollectionEquality().hash(_cards));

@override
String toString() {
  return 'AgentReview(header: $header, fromAgent: $fromAgent, warnings: $warnings, cards: $cards)';
}


}

/// @nodoc
abstract mixin class _$AgentReviewCopyWith<$Res> implements $AgentReviewCopyWith<$Res> {
  factory _$AgentReviewCopyWith(_AgentReview value, $Res Function(_AgentReview) _then) = __$AgentReviewCopyWithImpl;
@override @useResult
$Res call({
 String header, String fromAgent, List<String> warnings, List<AgentCard> cards
});




}
/// @nodoc
class __$AgentReviewCopyWithImpl<$Res>
    implements _$AgentReviewCopyWith<$Res> {
  __$AgentReviewCopyWithImpl(this._self, this._then);

  final _AgentReview _self;
  final $Res Function(_AgentReview) _then;

/// Create a copy of AgentReview
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? header = null,Object? fromAgent = null,Object? warnings = null,Object? cards = null,}) {
  return _then(_AgentReview(
header: null == header ? _self.header : header // ignore: cast_nullable_to_non_nullable
as String,fromAgent: null == fromAgent ? _self.fromAgent : fromAgent // ignore: cast_nullable_to_non_nullable
as String,warnings: null == warnings ? _self._warnings : warnings // ignore: cast_nullable_to_non_nullable
as List<String>,cards: null == cards ? _self._cards : cards // ignore: cast_nullable_to_non_nullable
as List<AgentCard>,
  ));
}


}

// dart format on

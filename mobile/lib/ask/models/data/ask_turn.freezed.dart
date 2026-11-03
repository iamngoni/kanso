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
 String? get failure;/// What the question searched.
 AskScope get scope;/// The lines the answer's sentences rest on, checked against what was
/// sent; the Sources list.
 List<AskReceipt> get receipts;/// Notes it read but did not cite.
 List<AiNoteRef> get alsoRead;/// The banner over the answer; empty when there is none.
 String get banner;/// How many sentences are marked as not found.
 int get unverified;/// How many sentences cite nothing in an answer that cites elsewhere.
 int get noSource;/// Said above the question when the conversation before it was ended.
 String get notice;/// The notes do not hold the answer: "You never wrote that down."
 bool get neverWrote;/// The closest notes, shown with [neverWrote].
 List<AskClosest> get closest;/// The time the question was understood to be about.
 AskTimeNote? get time;/// Nothing matched in a scope narrower than all notes.
 bool get narrow;/// The note the answer was saved as, once it was.
 String? get savedNoteId;
/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskTurnCopyWith<AskTurn> get copyWith => _$AskTurnCopyWithImpl<AskTurn>(this as AskTurn, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskTurn&&(identical(other.question, question) || other.question == question)&&(identical(other.answer, answer) || other.answer == answer)&&const DeepCollectionEquality().equals(other.used, used)&&(identical(other.outcome, outcome) || other.outcome == outcome)&&(identical(other.failure, failure) || other.failure == failure)&&(identical(other.scope, scope) || other.scope == scope)&&const DeepCollectionEquality().equals(other.receipts, receipts)&&const DeepCollectionEquality().equals(other.alsoRead, alsoRead)&&(identical(other.banner, banner) || other.banner == banner)&&(identical(other.unverified, unverified) || other.unverified == unverified)&&(identical(other.noSource, noSource) || other.noSource == noSource)&&(identical(other.notice, notice) || other.notice == notice)&&(identical(other.neverWrote, neverWrote) || other.neverWrote == neverWrote)&&const DeepCollectionEquality().equals(other.closest, closest)&&(identical(other.time, time) || other.time == time)&&(identical(other.narrow, narrow) || other.narrow == narrow)&&(identical(other.savedNoteId, savedNoteId) || other.savedNoteId == savedNoteId));
}


@override
int get hashCode => Object.hash(runtimeType,question,answer,const DeepCollectionEquality().hash(used),outcome,failure,scope,const DeepCollectionEquality().hash(receipts),const DeepCollectionEquality().hash(alsoRead),banner,unverified,noSource,notice,neverWrote,const DeepCollectionEquality().hash(closest),time,narrow,savedNoteId);

@override
String toString() {
  return 'AskTurn(question: $question, answer: $answer, used: $used, outcome: $outcome, failure: $failure, scope: $scope, receipts: $receipts, alsoRead: $alsoRead, banner: $banner, unverified: $unverified, noSource: $noSource, notice: $notice, neverWrote: $neverWrote, closest: $closest, time: $time, narrow: $narrow, savedNoteId: $savedNoteId)';
}


}

/// @nodoc
abstract mixin class $AskTurnCopyWith<$Res>  {
  factory $AskTurnCopyWith(AskTurn value, $Res Function(AskTurn) _then) = _$AskTurnCopyWithImpl;
@useResult
$Res call({
 String question, String answer, List<AiNoteRef> used, AskOutcome outcome, String? failure, AskScope scope, List<AskReceipt> receipts, List<AiNoteRef> alsoRead, String banner, int unverified, int noSource, String notice, bool neverWrote, List<AskClosest> closest, AskTimeNote? time, bool narrow, String? savedNoteId
});


$AskScopeCopyWith<$Res> get scope;$AskTimeNoteCopyWith<$Res>? get time;

}
/// @nodoc
class _$AskTurnCopyWithImpl<$Res>
    implements $AskTurnCopyWith<$Res> {
  _$AskTurnCopyWithImpl(this._self, this._then);

  final AskTurn _self;
  final $Res Function(AskTurn) _then;

/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? question = null,Object? answer = null,Object? used = null,Object? outcome = null,Object? failure = freezed,Object? scope = null,Object? receipts = null,Object? alsoRead = null,Object? banner = null,Object? unverified = null,Object? noSource = null,Object? notice = null,Object? neverWrote = null,Object? closest = null,Object? time = freezed,Object? narrow = null,Object? savedNoteId = freezed,}) {
  return _then(_self.copyWith(
question: null == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as String,answer: null == answer ? _self.answer : answer // ignore: cast_nullable_to_non_nullable
as String,used: null == used ? _self.used : used // ignore: cast_nullable_to_non_nullable
as List<AiNoteRef>,outcome: null == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AskOutcome,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as String?,scope: null == scope ? _self.scope : scope // ignore: cast_nullable_to_non_nullable
as AskScope,receipts: null == receipts ? _self.receipts : receipts // ignore: cast_nullable_to_non_nullable
as List<AskReceipt>,alsoRead: null == alsoRead ? _self.alsoRead : alsoRead // ignore: cast_nullable_to_non_nullable
as List<AiNoteRef>,banner: null == banner ? _self.banner : banner // ignore: cast_nullable_to_non_nullable
as String,unverified: null == unverified ? _self.unverified : unverified // ignore: cast_nullable_to_non_nullable
as int,noSource: null == noSource ? _self.noSource : noSource // ignore: cast_nullable_to_non_nullable
as int,notice: null == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as String,neverWrote: null == neverWrote ? _self.neverWrote : neverWrote // ignore: cast_nullable_to_non_nullable
as bool,closest: null == closest ? _self.closest : closest // ignore: cast_nullable_to_non_nullable
as List<AskClosest>,time: freezed == time ? _self.time : time // ignore: cast_nullable_to_non_nullable
as AskTimeNote?,narrow: null == narrow ? _self.narrow : narrow // ignore: cast_nullable_to_non_nullable
as bool,savedNoteId: freezed == savedNoteId ? _self.savedNoteId : savedNoteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}
/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AskScopeCopyWith<$Res> get scope {
  
  return $AskScopeCopyWith<$Res>(_self.scope, (value) {
    return _then(_self.copyWith(scope: value));
  });
}/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AskTimeNoteCopyWith<$Res>? get time {
    if (_self.time == null) {
    return null;
  }

  return $AskTimeNoteCopyWith<$Res>(_self.time!, (value) {
    return _then(_self.copyWith(time: value));
  });
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String question,  String answer,  List<AiNoteRef> used,  AskOutcome outcome,  String? failure,  AskScope scope,  List<AskReceipt> receipts,  List<AiNoteRef> alsoRead,  String banner,  int unverified,  int noSource,  String notice,  bool neverWrote,  List<AskClosest> closest,  AskTimeNote? time,  bool narrow,  String? savedNoteId)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AskTurn() when $default != null:
return $default(_that.question,_that.answer,_that.used,_that.outcome,_that.failure,_that.scope,_that.receipts,_that.alsoRead,_that.banner,_that.unverified,_that.noSource,_that.notice,_that.neverWrote,_that.closest,_that.time,_that.narrow,_that.savedNoteId);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String question,  String answer,  List<AiNoteRef> used,  AskOutcome outcome,  String? failure,  AskScope scope,  List<AskReceipt> receipts,  List<AiNoteRef> alsoRead,  String banner,  int unverified,  int noSource,  String notice,  bool neverWrote,  List<AskClosest> closest,  AskTimeNote? time,  bool narrow,  String? savedNoteId)  $default,) {final _that = this;
switch (_that) {
case _AskTurn():
return $default(_that.question,_that.answer,_that.used,_that.outcome,_that.failure,_that.scope,_that.receipts,_that.alsoRead,_that.banner,_that.unverified,_that.noSource,_that.notice,_that.neverWrote,_that.closest,_that.time,_that.narrow,_that.savedNoteId);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String question,  String answer,  List<AiNoteRef> used,  AskOutcome outcome,  String? failure,  AskScope scope,  List<AskReceipt> receipts,  List<AiNoteRef> alsoRead,  String banner,  int unverified,  int noSource,  String notice,  bool neverWrote,  List<AskClosest> closest,  AskTimeNote? time,  bool narrow,  String? savedNoteId)?  $default,) {final _that = this;
switch (_that) {
case _AskTurn() when $default != null:
return $default(_that.question,_that.answer,_that.used,_that.outcome,_that.failure,_that.scope,_that.receipts,_that.alsoRead,_that.banner,_that.unverified,_that.noSource,_that.notice,_that.neverWrote,_that.closest,_that.time,_that.narrow,_that.savedNoteId);case _:
  return null;

}
}

}

/// @nodoc


class _AskTurn implements AskTurn {
  const _AskTurn({required this.question, this.answer = '', final  List<AiNoteRef> used = const <AiNoteRef>[], this.outcome = AskOutcome.pending, this.failure, this.scope = AskScope.everything, final  List<AskReceipt> receipts = const <AskReceipt>[], final  List<AiNoteRef> alsoRead = const <AiNoteRef>[], this.banner = '', this.unverified = 0, this.noSource = 0, this.notice = '', this.neverWrote = false, final  List<AskClosest> closest = const <AskClosest>[], this.time, this.narrow = false, this.savedNoteId}): _used = used,_receipts = receipts,_alsoRead = alsoRead,_closest = closest;
  

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
/// What the question searched.
@override@JsonKey() final  AskScope scope;
/// The lines the answer's sentences rest on, checked against what was
/// sent; the Sources list.
 final  List<AskReceipt> _receipts;
/// The lines the answer's sentences rest on, checked against what was
/// sent; the Sources list.
@override@JsonKey() List<AskReceipt> get receipts {
  if (_receipts is EqualUnmodifiableListView) return _receipts;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_receipts);
}

/// Notes it read but did not cite.
 final  List<AiNoteRef> _alsoRead;
/// Notes it read but did not cite.
@override@JsonKey() List<AiNoteRef> get alsoRead {
  if (_alsoRead is EqualUnmodifiableListView) return _alsoRead;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_alsoRead);
}

/// The banner over the answer; empty when there is none.
@override@JsonKey() final  String banner;
/// How many sentences are marked as not found.
@override@JsonKey() final  int unverified;
/// How many sentences cite nothing in an answer that cites elsewhere.
@override@JsonKey() final  int noSource;
/// Said above the question when the conversation before it was ended.
@override@JsonKey() final  String notice;
/// The notes do not hold the answer: "You never wrote that down."
@override@JsonKey() final  bool neverWrote;
/// The closest notes, shown with [neverWrote].
 final  List<AskClosest> _closest;
/// The closest notes, shown with [neverWrote].
@override@JsonKey() List<AskClosest> get closest {
  if (_closest is EqualUnmodifiableListView) return _closest;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_closest);
}

/// The time the question was understood to be about.
@override final  AskTimeNote? time;
/// Nothing matched in a scope narrower than all notes.
@override@JsonKey() final  bool narrow;
/// The note the answer was saved as, once it was.
@override final  String? savedNoteId;

/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AskTurnCopyWith<_AskTurn> get copyWith => __$AskTurnCopyWithImpl<_AskTurn>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AskTurn&&(identical(other.question, question) || other.question == question)&&(identical(other.answer, answer) || other.answer == answer)&&const DeepCollectionEquality().equals(other._used, _used)&&(identical(other.outcome, outcome) || other.outcome == outcome)&&(identical(other.failure, failure) || other.failure == failure)&&(identical(other.scope, scope) || other.scope == scope)&&const DeepCollectionEquality().equals(other._receipts, _receipts)&&const DeepCollectionEquality().equals(other._alsoRead, _alsoRead)&&(identical(other.banner, banner) || other.banner == banner)&&(identical(other.unverified, unverified) || other.unverified == unverified)&&(identical(other.noSource, noSource) || other.noSource == noSource)&&(identical(other.notice, notice) || other.notice == notice)&&(identical(other.neverWrote, neverWrote) || other.neverWrote == neverWrote)&&const DeepCollectionEquality().equals(other._closest, _closest)&&(identical(other.time, time) || other.time == time)&&(identical(other.narrow, narrow) || other.narrow == narrow)&&(identical(other.savedNoteId, savedNoteId) || other.savedNoteId == savedNoteId));
}


@override
int get hashCode => Object.hash(runtimeType,question,answer,const DeepCollectionEquality().hash(_used),outcome,failure,scope,const DeepCollectionEquality().hash(_receipts),const DeepCollectionEquality().hash(_alsoRead),banner,unverified,noSource,notice,neverWrote,const DeepCollectionEquality().hash(_closest),time,narrow,savedNoteId);

@override
String toString() {
  return 'AskTurn(question: $question, answer: $answer, used: $used, outcome: $outcome, failure: $failure, scope: $scope, receipts: $receipts, alsoRead: $alsoRead, banner: $banner, unverified: $unverified, noSource: $noSource, notice: $notice, neverWrote: $neverWrote, closest: $closest, time: $time, narrow: $narrow, savedNoteId: $savedNoteId)';
}


}

/// @nodoc
abstract mixin class _$AskTurnCopyWith<$Res> implements $AskTurnCopyWith<$Res> {
  factory _$AskTurnCopyWith(_AskTurn value, $Res Function(_AskTurn) _then) = __$AskTurnCopyWithImpl;
@override @useResult
$Res call({
 String question, String answer, List<AiNoteRef> used, AskOutcome outcome, String? failure, AskScope scope, List<AskReceipt> receipts, List<AiNoteRef> alsoRead, String banner, int unverified, int noSource, String notice, bool neverWrote, List<AskClosest> closest, AskTimeNote? time, bool narrow, String? savedNoteId
});


@override $AskScopeCopyWith<$Res> get scope;@override $AskTimeNoteCopyWith<$Res>? get time;

}
/// @nodoc
class __$AskTurnCopyWithImpl<$Res>
    implements _$AskTurnCopyWith<$Res> {
  __$AskTurnCopyWithImpl(this._self, this._then);

  final _AskTurn _self;
  final $Res Function(_AskTurn) _then;

/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? question = null,Object? answer = null,Object? used = null,Object? outcome = null,Object? failure = freezed,Object? scope = null,Object? receipts = null,Object? alsoRead = null,Object? banner = null,Object? unverified = null,Object? noSource = null,Object? notice = null,Object? neverWrote = null,Object? closest = null,Object? time = freezed,Object? narrow = null,Object? savedNoteId = freezed,}) {
  return _then(_AskTurn(
question: null == question ? _self.question : question // ignore: cast_nullable_to_non_nullable
as String,answer: null == answer ? _self.answer : answer // ignore: cast_nullable_to_non_nullable
as String,used: null == used ? _self._used : used // ignore: cast_nullable_to_non_nullable
as List<AiNoteRef>,outcome: null == outcome ? _self.outcome : outcome // ignore: cast_nullable_to_non_nullable
as AskOutcome,failure: freezed == failure ? _self.failure : failure // ignore: cast_nullable_to_non_nullable
as String?,scope: null == scope ? _self.scope : scope // ignore: cast_nullable_to_non_nullable
as AskScope,receipts: null == receipts ? _self._receipts : receipts // ignore: cast_nullable_to_non_nullable
as List<AskReceipt>,alsoRead: null == alsoRead ? _self._alsoRead : alsoRead // ignore: cast_nullable_to_non_nullable
as List<AiNoteRef>,banner: null == banner ? _self.banner : banner // ignore: cast_nullable_to_non_nullable
as String,unverified: null == unverified ? _self.unverified : unverified // ignore: cast_nullable_to_non_nullable
as int,noSource: null == noSource ? _self.noSource : noSource // ignore: cast_nullable_to_non_nullable
as int,notice: null == notice ? _self.notice : notice // ignore: cast_nullable_to_non_nullable
as String,neverWrote: null == neverWrote ? _self.neverWrote : neverWrote // ignore: cast_nullable_to_non_nullable
as bool,closest: null == closest ? _self._closest : closest // ignore: cast_nullable_to_non_nullable
as List<AskClosest>,time: freezed == time ? _self.time : time // ignore: cast_nullable_to_non_nullable
as AskTimeNote?,narrow: null == narrow ? _self.narrow : narrow // ignore: cast_nullable_to_non_nullable
as bool,savedNoteId: freezed == savedNoteId ? _self.savedNoteId : savedNoteId // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}

/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AskScopeCopyWith<$Res> get scope {
  
  return $AskScopeCopyWith<$Res>(_self.scope, (value) {
    return _then(_self.copyWith(scope: value));
  });
}/// Create a copy of AskTurn
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AskTimeNoteCopyWith<$Res>? get time {
    if (_self.time == null) {
    return null;
  }

  return $AskTimeNoteCopyWith<$Res>(_self.time!, (value) {
    return _then(_self.copyWith(time: value));
  });
}
}

// dart format on

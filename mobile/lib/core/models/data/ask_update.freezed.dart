// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ask_update.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AskUpdate {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskUpdate);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskUpdate()';
}


}

/// @nodoc
class $AskUpdateCopyWith<$Res>  {
$AskUpdateCopyWith(AskUpdate _, $Res Function(AskUpdate) __);
}


/// Adds pattern-matching-related methods to [AskUpdate].
extension AskUpdatePatterns on AskUpdate {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AskSearching value)?  searching,TResult Function( AskThinking value)?  thinking,TResult Function( AskUsedNotes value)?  usedNotes,TResult Function( AskAnswer value)?  answer,TResult Function( AskTime value)?  time,TResult Function( AskRestarted value)?  restarted,TResult Function( AskDone value)?  done,TResult Function( AskNothingMatches value)?  nothingMatches,TResult Function( AskCutOff value)?  cutOff,TResult Function( AskFailed value)?  error,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AskSearching() when searching != null:
return searching(_that);case AskThinking() when thinking != null:
return thinking(_that);case AskUsedNotes() when usedNotes != null:
return usedNotes(_that);case AskAnswer() when answer != null:
return answer(_that);case AskTime() when time != null:
return time(_that);case AskRestarted() when restarted != null:
return restarted(_that);case AskDone() when done != null:
return done(_that);case AskNothingMatches() when nothingMatches != null:
return nothingMatches(_that);case AskCutOff() when cutOff != null:
return cutOff(_that);case AskFailed() when error != null:
return error(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AskSearching value)  searching,required TResult Function( AskThinking value)  thinking,required TResult Function( AskUsedNotes value)  usedNotes,required TResult Function( AskAnswer value)  answer,required TResult Function( AskTime value)  time,required TResult Function( AskRestarted value)  restarted,required TResult Function( AskDone value)  done,required TResult Function( AskNothingMatches value)  nothingMatches,required TResult Function( AskCutOff value)  cutOff,required TResult Function( AskFailed value)  error,}){
final _that = this;
switch (_that) {
case AskSearching():
return searching(_that);case AskThinking():
return thinking(_that);case AskUsedNotes():
return usedNotes(_that);case AskAnswer():
return answer(_that);case AskTime():
return time(_that);case AskRestarted():
return restarted(_that);case AskDone():
return done(_that);case AskNothingMatches():
return nothingMatches(_that);case AskCutOff():
return cutOff(_that);case AskFailed():
return error(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AskSearching value)?  searching,TResult? Function( AskThinking value)?  thinking,TResult? Function( AskUsedNotes value)?  usedNotes,TResult? Function( AskAnswer value)?  answer,TResult? Function( AskTime value)?  time,TResult? Function( AskRestarted value)?  restarted,TResult? Function( AskDone value)?  done,TResult? Function( AskNothingMatches value)?  nothingMatches,TResult? Function( AskCutOff value)?  cutOff,TResult? Function( AskFailed value)?  error,}){
final _that = this;
switch (_that) {
case AskSearching() when searching != null:
return searching(_that);case AskThinking() when thinking != null:
return thinking(_that);case AskUsedNotes() when usedNotes != null:
return usedNotes(_that);case AskAnswer() when answer != null:
return answer(_that);case AskTime() when time != null:
return time(_that);case AskRestarted() when restarted != null:
return restarted(_that);case AskDone() when done != null:
return done(_that);case AskNothingMatches() when nothingMatches != null:
return nothingMatches(_that);case AskCutOff() when cutOff != null:
return cutOff(_that);case AskFailed() when error != null:
return error(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  searching,TResult Function()?  thinking,TResult Function( List<AiNoteRef> notes)?  usedNotes,TResult Function( String text)?  answer,TResult Function( AskTimeNote time)?  time,TResult Function( String line)?  restarted,TResult Function( String answer,  List<AiNoteRef> used,  List<AskReceipt> receipts,  List<AiNoteRef> alsoRead,  String banner,  int unverified,  int noSource,  bool neverWrote,  List<AskClosest> closest,  AskTimeNote? time)?  done,TResult Function( bool narrow)?  nothingMatches,TResult Function()?  cutOff,TResult Function( String message)?  error,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AskSearching() when searching != null:
return searching();case AskThinking() when thinking != null:
return thinking();case AskUsedNotes() when usedNotes != null:
return usedNotes(_that.notes);case AskAnswer() when answer != null:
return answer(_that.text);case AskTime() when time != null:
return time(_that.time);case AskRestarted() when restarted != null:
return restarted(_that.line);case AskDone() when done != null:
return done(_that.answer,_that.used,_that.receipts,_that.alsoRead,_that.banner,_that.unverified,_that.noSource,_that.neverWrote,_that.closest,_that.time);case AskNothingMatches() when nothingMatches != null:
return nothingMatches(_that.narrow);case AskCutOff() when cutOff != null:
return cutOff();case AskFailed() when error != null:
return error(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  searching,required TResult Function()  thinking,required TResult Function( List<AiNoteRef> notes)  usedNotes,required TResult Function( String text)  answer,required TResult Function( AskTimeNote time)  time,required TResult Function( String line)  restarted,required TResult Function( String answer,  List<AiNoteRef> used,  List<AskReceipt> receipts,  List<AiNoteRef> alsoRead,  String banner,  int unverified,  int noSource,  bool neverWrote,  List<AskClosest> closest,  AskTimeNote? time)  done,required TResult Function( bool narrow)  nothingMatches,required TResult Function()  cutOff,required TResult Function( String message)  error,}) {final _that = this;
switch (_that) {
case AskSearching():
return searching();case AskThinking():
return thinking();case AskUsedNotes():
return usedNotes(_that.notes);case AskAnswer():
return answer(_that.text);case AskTime():
return time(_that.time);case AskRestarted():
return restarted(_that.line);case AskDone():
return done(_that.answer,_that.used,_that.receipts,_that.alsoRead,_that.banner,_that.unverified,_that.noSource,_that.neverWrote,_that.closest,_that.time);case AskNothingMatches():
return nothingMatches(_that.narrow);case AskCutOff():
return cutOff();case AskFailed():
return error(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  searching,TResult? Function()?  thinking,TResult? Function( List<AiNoteRef> notes)?  usedNotes,TResult? Function( String text)?  answer,TResult? Function( AskTimeNote time)?  time,TResult? Function( String line)?  restarted,TResult? Function( String answer,  List<AiNoteRef> used,  List<AskReceipt> receipts,  List<AiNoteRef> alsoRead,  String banner,  int unverified,  int noSource,  bool neverWrote,  List<AskClosest> closest,  AskTimeNote? time)?  done,TResult? Function( bool narrow)?  nothingMatches,TResult? Function()?  cutOff,TResult? Function( String message)?  error,}) {final _that = this;
switch (_that) {
case AskSearching() when searching != null:
return searching();case AskThinking() when thinking != null:
return thinking();case AskUsedNotes() when usedNotes != null:
return usedNotes(_that.notes);case AskAnswer() when answer != null:
return answer(_that.text);case AskTime() when time != null:
return time(_that.time);case AskRestarted() when restarted != null:
return restarted(_that.line);case AskDone() when done != null:
return done(_that.answer,_that.used,_that.receipts,_that.alsoRead,_that.banner,_that.unverified,_that.noSource,_that.neverWrote,_that.closest,_that.time);case AskNothingMatches() when nothingMatches != null:
return nothingMatches(_that.narrow);case AskCutOff() when cutOff != null:
return cutOff();case AskFailed() when error != null:
return error(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class AskSearching implements AskUpdate {
  const AskSearching();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskSearching);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskUpdate.searching()';
}


}




/// @nodoc


class AskThinking implements AskUpdate {
  const AskThinking();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskThinking);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskUpdate.thinking()';
}


}




/// @nodoc


class AskUsedNotes implements AskUpdate {
  const AskUsedNotes({required final  List<AiNoteRef> notes}): _notes = notes;
  

 final  List<AiNoteRef> _notes;
 List<AiNoteRef> get notes {
  if (_notes is EqualUnmodifiableListView) return _notes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_notes);
}


/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskUsedNotesCopyWith<AskUsedNotes> get copyWith => _$AskUsedNotesCopyWithImpl<AskUsedNotes>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskUsedNotes&&const DeepCollectionEquality().equals(other._notes, _notes));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_notes));

@override
String toString() {
  return 'AskUpdate.usedNotes(notes: $notes)';
}


}

/// @nodoc
abstract mixin class $AskUsedNotesCopyWith<$Res> implements $AskUpdateCopyWith<$Res> {
  factory $AskUsedNotesCopyWith(AskUsedNotes value, $Res Function(AskUsedNotes) _then) = _$AskUsedNotesCopyWithImpl;
@useResult
$Res call({
 List<AiNoteRef> notes
});




}
/// @nodoc
class _$AskUsedNotesCopyWithImpl<$Res>
    implements $AskUsedNotesCopyWith<$Res> {
  _$AskUsedNotesCopyWithImpl(this._self, this._then);

  final AskUsedNotes _self;
  final $Res Function(AskUsedNotes) _then;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notes = null,}) {
  return _then(AskUsedNotes(
notes: null == notes ? _self._notes : notes // ignore: cast_nullable_to_non_nullable
as List<AiNoteRef>,
  ));
}


}

/// @nodoc


class AskAnswer implements AskUpdate {
  const AskAnswer({required this.text});
  

 final  String text;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskAnswerCopyWith<AskAnswer> get copyWith => _$AskAnswerCopyWithImpl<AskAnswer>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskAnswer&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AskUpdate.answer(text: $text)';
}


}

/// @nodoc
abstract mixin class $AskAnswerCopyWith<$Res> implements $AskUpdateCopyWith<$Res> {
  factory $AskAnswerCopyWith(AskAnswer value, $Res Function(AskAnswer) _then) = _$AskAnswerCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AskAnswerCopyWithImpl<$Res>
    implements $AskAnswerCopyWith<$Res> {
  _$AskAnswerCopyWithImpl(this._self, this._then);

  final AskAnswer _self;
  final $Res Function(AskAnswer) _then;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AskAnswer(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AskTime implements AskUpdate {
  const AskTime({required this.time});
  

 final  AskTimeNote time;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskTimeCopyWith<AskTime> get copyWith => _$AskTimeCopyWithImpl<AskTime>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskTime&&(identical(other.time, time) || other.time == time));
}


@override
int get hashCode => Object.hash(runtimeType,time);

@override
String toString() {
  return 'AskUpdate.time(time: $time)';
}


}

/// @nodoc
abstract mixin class $AskTimeCopyWith<$Res> implements $AskUpdateCopyWith<$Res> {
  factory $AskTimeCopyWith(AskTime value, $Res Function(AskTime) _then) = _$AskTimeCopyWithImpl;
@useResult
$Res call({
 AskTimeNote time
});


$AskTimeNoteCopyWith<$Res> get time;

}
/// @nodoc
class _$AskTimeCopyWithImpl<$Res>
    implements $AskTimeCopyWith<$Res> {
  _$AskTimeCopyWithImpl(this._self, this._then);

  final AskTime _self;
  final $Res Function(AskTime) _then;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? time = null,}) {
  return _then(AskTime(
time: null == time ? _self.time : time // ignore: cast_nullable_to_non_nullable
as AskTimeNote,
  ));
}

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$AskTimeNoteCopyWith<$Res> get time {
  
  return $AskTimeNoteCopyWith<$Res>(_self.time, (value) {
    return _then(_self.copyWith(time: value));
  });
}
}

/// @nodoc


class AskRestarted implements AskUpdate {
  const AskRestarted({required this.line});
  

 final  String line;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskRestartedCopyWith<AskRestarted> get copyWith => _$AskRestartedCopyWithImpl<AskRestarted>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskRestarted&&(identical(other.line, line) || other.line == line));
}


@override
int get hashCode => Object.hash(runtimeType,line);

@override
String toString() {
  return 'AskUpdate.restarted(line: $line)';
}


}

/// @nodoc
abstract mixin class $AskRestartedCopyWith<$Res> implements $AskUpdateCopyWith<$Res> {
  factory $AskRestartedCopyWith(AskRestarted value, $Res Function(AskRestarted) _then) = _$AskRestartedCopyWithImpl;
@useResult
$Res call({
 String line
});




}
/// @nodoc
class _$AskRestartedCopyWithImpl<$Res>
    implements $AskRestartedCopyWith<$Res> {
  _$AskRestartedCopyWithImpl(this._self, this._then);

  final AskRestarted _self;
  final $Res Function(AskRestarted) _then;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? line = null,}) {
  return _then(AskRestarted(
line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AskDone implements AskUpdate {
  const AskDone({required this.answer, required final  List<AiNoteRef> used, final  List<AskReceipt> receipts = const <AskReceipt>[], final  List<AiNoteRef> alsoRead = const <AiNoteRef>[], this.banner = '', this.unverified = 0, this.noSource = 0, this.neverWrote = false, final  List<AskClosest> closest = const <AskClosest>[], this.time}): _used = used,_receipts = receipts,_alsoRead = alsoRead,_closest = closest;
  

 final  String answer;
 final  List<AiNoteRef> _used;
 List<AiNoteRef> get used {
  if (_used is EqualUnmodifiableListView) return _used;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_used);
}

 final  List<AskReceipt> _receipts;
@JsonKey() List<AskReceipt> get receipts {
  if (_receipts is EqualUnmodifiableListView) return _receipts;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_receipts);
}

/// Notes it read but did not cite.
 final  List<AiNoteRef> _alsoRead;
/// Notes it read but did not cite.
@JsonKey() List<AiNoteRef> get alsoRead {
  if (_alsoRead is EqualUnmodifiableListView) return _alsoRead;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_alsoRead);
}

/// The banner over the answer; empty when there is none.
@JsonKey() final  String banner;
/// How many sentences are marked as not found.
@JsonKey() final  int unverified;
/// How many sentences cite nothing in an answer that cites elsewhere.
@JsonKey() final  int noSource;
/// The notes do not hold the answer.
@JsonKey() final  bool neverWrote;
 final  List<AskClosest> _closest;
@JsonKey() List<AskClosest> get closest {
  if (_closest is EqualUnmodifiableListView) return _closest;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_closest);
}

 final  AskTimeNote? time;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskDoneCopyWith<AskDone> get copyWith => _$AskDoneCopyWithImpl<AskDone>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskDone&&(identical(other.answer, answer) || other.answer == answer)&&const DeepCollectionEquality().equals(other._used, _used)&&const DeepCollectionEquality().equals(other._receipts, _receipts)&&const DeepCollectionEquality().equals(other._alsoRead, _alsoRead)&&(identical(other.banner, banner) || other.banner == banner)&&(identical(other.unverified, unverified) || other.unverified == unverified)&&(identical(other.noSource, noSource) || other.noSource == noSource)&&(identical(other.neverWrote, neverWrote) || other.neverWrote == neverWrote)&&const DeepCollectionEquality().equals(other._closest, _closest)&&(identical(other.time, time) || other.time == time));
}


@override
int get hashCode => Object.hash(runtimeType,answer,const DeepCollectionEquality().hash(_used),const DeepCollectionEquality().hash(_receipts),const DeepCollectionEquality().hash(_alsoRead),banner,unverified,noSource,neverWrote,const DeepCollectionEquality().hash(_closest),time);

@override
String toString() {
  return 'AskUpdate.done(answer: $answer, used: $used, receipts: $receipts, alsoRead: $alsoRead, banner: $banner, unverified: $unverified, noSource: $noSource, neverWrote: $neverWrote, closest: $closest, time: $time)';
}


}

/// @nodoc
abstract mixin class $AskDoneCopyWith<$Res> implements $AskUpdateCopyWith<$Res> {
  factory $AskDoneCopyWith(AskDone value, $Res Function(AskDone) _then) = _$AskDoneCopyWithImpl;
@useResult
$Res call({
 String answer, List<AiNoteRef> used, List<AskReceipt> receipts, List<AiNoteRef> alsoRead, String banner, int unverified, int noSource, bool neverWrote, List<AskClosest> closest, AskTimeNote? time
});


$AskTimeNoteCopyWith<$Res>? get time;

}
/// @nodoc
class _$AskDoneCopyWithImpl<$Res>
    implements $AskDoneCopyWith<$Res> {
  _$AskDoneCopyWithImpl(this._self, this._then);

  final AskDone _self;
  final $Res Function(AskDone) _then;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? answer = null,Object? used = null,Object? receipts = null,Object? alsoRead = null,Object? banner = null,Object? unverified = null,Object? noSource = null,Object? neverWrote = null,Object? closest = null,Object? time = freezed,}) {
  return _then(AskDone(
answer: null == answer ? _self.answer : answer // ignore: cast_nullable_to_non_nullable
as String,used: null == used ? _self._used : used // ignore: cast_nullable_to_non_nullable
as List<AiNoteRef>,receipts: null == receipts ? _self._receipts : receipts // ignore: cast_nullable_to_non_nullable
as List<AskReceipt>,alsoRead: null == alsoRead ? _self._alsoRead : alsoRead // ignore: cast_nullable_to_non_nullable
as List<AiNoteRef>,banner: null == banner ? _self.banner : banner // ignore: cast_nullable_to_non_nullable
as String,unverified: null == unverified ? _self.unverified : unverified // ignore: cast_nullable_to_non_nullable
as int,noSource: null == noSource ? _self.noSource : noSource // ignore: cast_nullable_to_non_nullable
as int,neverWrote: null == neverWrote ? _self.neverWrote : neverWrote // ignore: cast_nullable_to_non_nullable
as bool,closest: null == closest ? _self._closest : closest // ignore: cast_nullable_to_non_nullable
as List<AskClosest>,time: freezed == time ? _self.time : time // ignore: cast_nullable_to_non_nullable
as AskTimeNote?,
  ));
}

/// Create a copy of AskUpdate
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

/// @nodoc


class AskNothingMatches implements AskUpdate {
  const AskNothingMatches({this.narrow = false});
  

@JsonKey() final  bool narrow;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskNothingMatchesCopyWith<AskNothingMatches> get copyWith => _$AskNothingMatchesCopyWithImpl<AskNothingMatches>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskNothingMatches&&(identical(other.narrow, narrow) || other.narrow == narrow));
}


@override
int get hashCode => Object.hash(runtimeType,narrow);

@override
String toString() {
  return 'AskUpdate.nothingMatches(narrow: $narrow)';
}


}

/// @nodoc
abstract mixin class $AskNothingMatchesCopyWith<$Res> implements $AskUpdateCopyWith<$Res> {
  factory $AskNothingMatchesCopyWith(AskNothingMatches value, $Res Function(AskNothingMatches) _then) = _$AskNothingMatchesCopyWithImpl;
@useResult
$Res call({
 bool narrow
});




}
/// @nodoc
class _$AskNothingMatchesCopyWithImpl<$Res>
    implements $AskNothingMatchesCopyWith<$Res> {
  _$AskNothingMatchesCopyWithImpl(this._self, this._then);

  final AskNothingMatches _self;
  final $Res Function(AskNothingMatches) _then;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? narrow = null,}) {
  return _then(AskNothingMatches(
narrow: null == narrow ? _self.narrow : narrow // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class AskCutOff implements AskUpdate {
  const AskCutOff();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskCutOff);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskUpdate.cutOff()';
}


}




/// @nodoc


class AskFailed implements AskUpdate {
  const AskFailed({required this.message});
  

 final  String message;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskFailedCopyWith<AskFailed> get copyWith => _$AskFailedCopyWithImpl<AskFailed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskFailed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'AskUpdate.error(message: $message)';
}


}

/// @nodoc
abstract mixin class $AskFailedCopyWith<$Res> implements $AskUpdateCopyWith<$Res> {
  factory $AskFailedCopyWith(AskFailed value, $Res Function(AskFailed) _then) = _$AskFailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$AskFailedCopyWithImpl<$Res>
    implements $AskFailedCopyWith<$Res> {
  _$AskFailedCopyWithImpl(this._self, this._then);

  final AskFailed _self;
  final $Res Function(AskFailed) _then;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(AskFailed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on

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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AskSearching value)?  searching,TResult Function( AskThinking value)?  thinking,TResult Function( AskUsedNotes value)?  usedNotes,TResult Function( AskAnswer value)?  answer,TResult Function( AskDone value)?  done,TResult Function( AskNothingMatches value)?  nothingMatches,TResult Function( AskCutOff value)?  cutOff,TResult Function( AskFailed value)?  error,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AskSearching() when searching != null:
return searching(_that);case AskThinking() when thinking != null:
return thinking(_that);case AskUsedNotes() when usedNotes != null:
return usedNotes(_that);case AskAnswer() when answer != null:
return answer(_that);case AskDone() when done != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AskSearching value)  searching,required TResult Function( AskThinking value)  thinking,required TResult Function( AskUsedNotes value)  usedNotes,required TResult Function( AskAnswer value)  answer,required TResult Function( AskDone value)  done,required TResult Function( AskNothingMatches value)  nothingMatches,required TResult Function( AskCutOff value)  cutOff,required TResult Function( AskFailed value)  error,}){
final _that = this;
switch (_that) {
case AskSearching():
return searching(_that);case AskThinking():
return thinking(_that);case AskUsedNotes():
return usedNotes(_that);case AskAnswer():
return answer(_that);case AskDone():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AskSearching value)?  searching,TResult? Function( AskThinking value)?  thinking,TResult? Function( AskUsedNotes value)?  usedNotes,TResult? Function( AskAnswer value)?  answer,TResult? Function( AskDone value)?  done,TResult? Function( AskNothingMatches value)?  nothingMatches,TResult? Function( AskCutOff value)?  cutOff,TResult? Function( AskFailed value)?  error,}){
final _that = this;
switch (_that) {
case AskSearching() when searching != null:
return searching(_that);case AskThinking() when thinking != null:
return thinking(_that);case AskUsedNotes() when usedNotes != null:
return usedNotes(_that);case AskAnswer() when answer != null:
return answer(_that);case AskDone() when done != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  searching,TResult Function()?  thinking,TResult Function( List<AiNoteRef> notes)?  usedNotes,TResult Function( String text)?  answer,TResult Function( String answer,  List<AiNoteRef> used)?  done,TResult Function()?  nothingMatches,TResult Function()?  cutOff,TResult Function( String message)?  error,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AskSearching() when searching != null:
return searching();case AskThinking() when thinking != null:
return thinking();case AskUsedNotes() when usedNotes != null:
return usedNotes(_that.notes);case AskAnswer() when answer != null:
return answer(_that.text);case AskDone() when done != null:
return done(_that.answer,_that.used);case AskNothingMatches() when nothingMatches != null:
return nothingMatches();case AskCutOff() when cutOff != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  searching,required TResult Function()  thinking,required TResult Function( List<AiNoteRef> notes)  usedNotes,required TResult Function( String text)  answer,required TResult Function( String answer,  List<AiNoteRef> used)  done,required TResult Function()  nothingMatches,required TResult Function()  cutOff,required TResult Function( String message)  error,}) {final _that = this;
switch (_that) {
case AskSearching():
return searching();case AskThinking():
return thinking();case AskUsedNotes():
return usedNotes(_that.notes);case AskAnswer():
return answer(_that.text);case AskDone():
return done(_that.answer,_that.used);case AskNothingMatches():
return nothingMatches();case AskCutOff():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  searching,TResult? Function()?  thinking,TResult? Function( List<AiNoteRef> notes)?  usedNotes,TResult? Function( String text)?  answer,TResult? Function( String answer,  List<AiNoteRef> used)?  done,TResult? Function()?  nothingMatches,TResult? Function()?  cutOff,TResult? Function( String message)?  error,}) {final _that = this;
switch (_that) {
case AskSearching() when searching != null:
return searching();case AskThinking() when thinking != null:
return thinking();case AskUsedNotes() when usedNotes != null:
return usedNotes(_that.notes);case AskAnswer() when answer != null:
return answer(_that.text);case AskDone() when done != null:
return done(_that.answer,_that.used);case AskNothingMatches() when nothingMatches != null:
return nothingMatches();case AskCutOff() when cutOff != null:
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


class AskDone implements AskUpdate {
  const AskDone({required this.answer, required final  List<AiNoteRef> used}): _used = used;
  

 final  String answer;
 final  List<AiNoteRef> _used;
 List<AiNoteRef> get used {
  if (_used is EqualUnmodifiableListView) return _used;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_used);
}


/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskDoneCopyWith<AskDone> get copyWith => _$AskDoneCopyWithImpl<AskDone>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskDone&&(identical(other.answer, answer) || other.answer == answer)&&const DeepCollectionEquality().equals(other._used, _used));
}


@override
int get hashCode => Object.hash(runtimeType,answer,const DeepCollectionEquality().hash(_used));

@override
String toString() {
  return 'AskUpdate.done(answer: $answer, used: $used)';
}


}

/// @nodoc
abstract mixin class $AskDoneCopyWith<$Res> implements $AskUpdateCopyWith<$Res> {
  factory $AskDoneCopyWith(AskDone value, $Res Function(AskDone) _then) = _$AskDoneCopyWithImpl;
@useResult
$Res call({
 String answer, List<AiNoteRef> used
});




}
/// @nodoc
class _$AskDoneCopyWithImpl<$Res>
    implements $AskDoneCopyWith<$Res> {
  _$AskDoneCopyWithImpl(this._self, this._then);

  final AskDone _self;
  final $Res Function(AskDone) _then;

/// Create a copy of AskUpdate
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? answer = null,Object? used = null,}) {
  return _then(AskDone(
answer: null == answer ? _self.answer : answer // ignore: cast_nullable_to_non_nullable
as String,used: null == used ? _self._used : used // ignore: cast_nullable_to_non_nullable
as List<AiNoteRef>,
  ));
}


}

/// @nodoc


class AskNothingMatches implements AskUpdate {
  const AskNothingMatches();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskNothingMatches);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskUpdate.nothingMatches()';
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

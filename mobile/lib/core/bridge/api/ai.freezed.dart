// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'ai.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AskEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskEvent()';
}


}

/// @nodoc
class $AskEventCopyWith<$Res>  {
$AskEventCopyWith(AskEvent _, $Res Function(AskEvent) __);
}


/// Adds pattern-matching-related methods to [AskEvent].
extension AskEventPatterns on AskEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AskEvent_Searching value)?  searching,TResult Function( AskEvent_Used value)?  used,TResult Function( AskEvent_Status value)?  status,TResult Function( AskEvent_Answer value)?  answer,TResult Function( AskEvent_Done value)?  done,TResult Function( AskEvent_NothingMatches value)?  nothingMatches,TResult Function( AskEvent_CutOff value)?  cutOff,TResult Function( AskEvent_Failed value)?  failed,TResult Function( AskEvent_Stopped value)?  stopped,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AskEvent_Searching() when searching != null:
return searching(_that);case AskEvent_Used() when used != null:
return used(_that);case AskEvent_Status() when status != null:
return status(_that);case AskEvent_Answer() when answer != null:
return answer(_that);case AskEvent_Done() when done != null:
return done(_that);case AskEvent_NothingMatches() when nothingMatches != null:
return nothingMatches(_that);case AskEvent_CutOff() when cutOff != null:
return cutOff(_that);case AskEvent_Failed() when failed != null:
return failed(_that);case AskEvent_Stopped() when stopped != null:
return stopped(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AskEvent_Searching value)  searching,required TResult Function( AskEvent_Used value)  used,required TResult Function( AskEvent_Status value)  status,required TResult Function( AskEvent_Answer value)  answer,required TResult Function( AskEvent_Done value)  done,required TResult Function( AskEvent_NothingMatches value)  nothingMatches,required TResult Function( AskEvent_CutOff value)  cutOff,required TResult Function( AskEvent_Failed value)  failed,required TResult Function( AskEvent_Stopped value)  stopped,}){
final _that = this;
switch (_that) {
case AskEvent_Searching():
return searching(_that);case AskEvent_Used():
return used(_that);case AskEvent_Status():
return status(_that);case AskEvent_Answer():
return answer(_that);case AskEvent_Done():
return done(_that);case AskEvent_NothingMatches():
return nothingMatches(_that);case AskEvent_CutOff():
return cutOff(_that);case AskEvent_Failed():
return failed(_that);case AskEvent_Stopped():
return stopped(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AskEvent_Searching value)?  searching,TResult? Function( AskEvent_Used value)?  used,TResult? Function( AskEvent_Status value)?  status,TResult? Function( AskEvent_Answer value)?  answer,TResult? Function( AskEvent_Done value)?  done,TResult? Function( AskEvent_NothingMatches value)?  nothingMatches,TResult? Function( AskEvent_CutOff value)?  cutOff,TResult? Function( AskEvent_Failed value)?  failed,TResult? Function( AskEvent_Stopped value)?  stopped,}){
final _that = this;
switch (_that) {
case AskEvent_Searching() when searching != null:
return searching(_that);case AskEvent_Used() when used != null:
return used(_that);case AskEvent_Status() when status != null:
return status(_that);case AskEvent_Answer() when answer != null:
return answer(_that);case AskEvent_Done() when done != null:
return done(_that);case AskEvent_NothingMatches() when nothingMatches != null:
return nothingMatches(_that);case AskEvent_CutOff() when cutOff != null:
return cutOff(_that);case AskEvent_Failed() when failed != null:
return failed(_that);case AskEvent_Stopped() when stopped != null:
return stopped(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  searching,TResult Function( List<AskNote> notes)?  used,TResult Function( String line)?  status,TResult Function( String markdown)?  answer,TResult Function( String markdown,  List<AskNote> notes)?  done,TResult Function( String message)?  nothingMatches,TResult Function( String message)?  cutOff,TResult Function( String message)?  failed,TResult Function()?  stopped,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AskEvent_Searching() when searching != null:
return searching();case AskEvent_Used() when used != null:
return used(_that.notes);case AskEvent_Status() when status != null:
return status(_that.line);case AskEvent_Answer() when answer != null:
return answer(_that.markdown);case AskEvent_Done() when done != null:
return done(_that.markdown,_that.notes);case AskEvent_NothingMatches() when nothingMatches != null:
return nothingMatches(_that.message);case AskEvent_CutOff() when cutOff != null:
return cutOff(_that.message);case AskEvent_Failed() when failed != null:
return failed(_that.message);case AskEvent_Stopped() when stopped != null:
return stopped();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  searching,required TResult Function( List<AskNote> notes)  used,required TResult Function( String line)  status,required TResult Function( String markdown)  answer,required TResult Function( String markdown,  List<AskNote> notes)  done,required TResult Function( String message)  nothingMatches,required TResult Function( String message)  cutOff,required TResult Function( String message)  failed,required TResult Function()  stopped,}) {final _that = this;
switch (_that) {
case AskEvent_Searching():
return searching();case AskEvent_Used():
return used(_that.notes);case AskEvent_Status():
return status(_that.line);case AskEvent_Answer():
return answer(_that.markdown);case AskEvent_Done():
return done(_that.markdown,_that.notes);case AskEvent_NothingMatches():
return nothingMatches(_that.message);case AskEvent_CutOff():
return cutOff(_that.message);case AskEvent_Failed():
return failed(_that.message);case AskEvent_Stopped():
return stopped();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  searching,TResult? Function( List<AskNote> notes)?  used,TResult? Function( String line)?  status,TResult? Function( String markdown)?  answer,TResult? Function( String markdown,  List<AskNote> notes)?  done,TResult? Function( String message)?  nothingMatches,TResult? Function( String message)?  cutOff,TResult? Function( String message)?  failed,TResult? Function()?  stopped,}) {final _that = this;
switch (_that) {
case AskEvent_Searching() when searching != null:
return searching();case AskEvent_Used() when used != null:
return used(_that.notes);case AskEvent_Status() when status != null:
return status(_that.line);case AskEvent_Answer() when answer != null:
return answer(_that.markdown);case AskEvent_Done() when done != null:
return done(_that.markdown,_that.notes);case AskEvent_NothingMatches() when nothingMatches != null:
return nothingMatches(_that.message);case AskEvent_CutOff() when cutOff != null:
return cutOff(_that.message);case AskEvent_Failed() when failed != null:
return failed(_that.message);case AskEvent_Stopped() when stopped != null:
return stopped();case _:
  return null;

}
}

}

/// @nodoc


class AskEvent_Searching extends AskEvent {
  const AskEvent_Searching(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent_Searching);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskEvent.searching()';
}


}




/// @nodoc


class AskEvent_Used extends AskEvent {
  const AskEvent_Used({required final  List<AskNote> notes}): _notes = notes,super._();
  

 final  List<AskNote> _notes;
 List<AskNote> get notes {
  if (_notes is EqualUnmodifiableListView) return _notes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_notes);
}


/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskEvent_UsedCopyWith<AskEvent_Used> get copyWith => _$AskEvent_UsedCopyWithImpl<AskEvent_Used>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent_Used&&const DeepCollectionEquality().equals(other._notes, _notes));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_notes));

@override
String toString() {
  return 'AskEvent.used(notes: $notes)';
}


}

/// @nodoc
abstract mixin class $AskEvent_UsedCopyWith<$Res> implements $AskEventCopyWith<$Res> {
  factory $AskEvent_UsedCopyWith(AskEvent_Used value, $Res Function(AskEvent_Used) _then) = _$AskEvent_UsedCopyWithImpl;
@useResult
$Res call({
 List<AskNote> notes
});




}
/// @nodoc
class _$AskEvent_UsedCopyWithImpl<$Res>
    implements $AskEvent_UsedCopyWith<$Res> {
  _$AskEvent_UsedCopyWithImpl(this._self, this._then);

  final AskEvent_Used _self;
  final $Res Function(AskEvent_Used) _then;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notes = null,}) {
  return _then(AskEvent_Used(
notes: null == notes ? _self._notes : notes // ignore: cast_nullable_to_non_nullable
as List<AskNote>,
  ));
}


}

/// @nodoc


class AskEvent_Status extends AskEvent {
  const AskEvent_Status({required this.line}): super._();
  

 final  String line;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskEvent_StatusCopyWith<AskEvent_Status> get copyWith => _$AskEvent_StatusCopyWithImpl<AskEvent_Status>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent_Status&&(identical(other.line, line) || other.line == line));
}


@override
int get hashCode => Object.hash(runtimeType,line);

@override
String toString() {
  return 'AskEvent.status(line: $line)';
}


}

/// @nodoc
abstract mixin class $AskEvent_StatusCopyWith<$Res> implements $AskEventCopyWith<$Res> {
  factory $AskEvent_StatusCopyWith(AskEvent_Status value, $Res Function(AskEvent_Status) _then) = _$AskEvent_StatusCopyWithImpl;
@useResult
$Res call({
 String line
});




}
/// @nodoc
class _$AskEvent_StatusCopyWithImpl<$Res>
    implements $AskEvent_StatusCopyWith<$Res> {
  _$AskEvent_StatusCopyWithImpl(this._self, this._then);

  final AskEvent_Status _self;
  final $Res Function(AskEvent_Status) _then;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? line = null,}) {
  return _then(AskEvent_Status(
line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AskEvent_Answer extends AskEvent {
  const AskEvent_Answer({required this.markdown}): super._();
  

 final  String markdown;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskEvent_AnswerCopyWith<AskEvent_Answer> get copyWith => _$AskEvent_AnswerCopyWithImpl<AskEvent_Answer>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent_Answer&&(identical(other.markdown, markdown) || other.markdown == markdown));
}


@override
int get hashCode => Object.hash(runtimeType,markdown);

@override
String toString() {
  return 'AskEvent.answer(markdown: $markdown)';
}


}

/// @nodoc
abstract mixin class $AskEvent_AnswerCopyWith<$Res> implements $AskEventCopyWith<$Res> {
  factory $AskEvent_AnswerCopyWith(AskEvent_Answer value, $Res Function(AskEvent_Answer) _then) = _$AskEvent_AnswerCopyWithImpl;
@useResult
$Res call({
 String markdown
});




}
/// @nodoc
class _$AskEvent_AnswerCopyWithImpl<$Res>
    implements $AskEvent_AnswerCopyWith<$Res> {
  _$AskEvent_AnswerCopyWithImpl(this._self, this._then);

  final AskEvent_Answer _self;
  final $Res Function(AskEvent_Answer) _then;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? markdown = null,}) {
  return _then(AskEvent_Answer(
markdown: null == markdown ? _self.markdown : markdown // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AskEvent_Done extends AskEvent {
  const AskEvent_Done({required this.markdown, required final  List<AskNote> notes}): _notes = notes,super._();
  

 final  String markdown;
 final  List<AskNote> _notes;
 List<AskNote> get notes {
  if (_notes is EqualUnmodifiableListView) return _notes;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_notes);
}


/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskEvent_DoneCopyWith<AskEvent_Done> get copyWith => _$AskEvent_DoneCopyWithImpl<AskEvent_Done>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent_Done&&(identical(other.markdown, markdown) || other.markdown == markdown)&&const DeepCollectionEquality().equals(other._notes, _notes));
}


@override
int get hashCode => Object.hash(runtimeType,markdown,const DeepCollectionEquality().hash(_notes));

@override
String toString() {
  return 'AskEvent.done(markdown: $markdown, notes: $notes)';
}


}

/// @nodoc
abstract mixin class $AskEvent_DoneCopyWith<$Res> implements $AskEventCopyWith<$Res> {
  factory $AskEvent_DoneCopyWith(AskEvent_Done value, $Res Function(AskEvent_Done) _then) = _$AskEvent_DoneCopyWithImpl;
@useResult
$Res call({
 String markdown, List<AskNote> notes
});




}
/// @nodoc
class _$AskEvent_DoneCopyWithImpl<$Res>
    implements $AskEvent_DoneCopyWith<$Res> {
  _$AskEvent_DoneCopyWithImpl(this._self, this._then);

  final AskEvent_Done _self;
  final $Res Function(AskEvent_Done) _then;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? markdown = null,Object? notes = null,}) {
  return _then(AskEvent_Done(
markdown: null == markdown ? _self.markdown : markdown // ignore: cast_nullable_to_non_nullable
as String,notes: null == notes ? _self._notes : notes // ignore: cast_nullable_to_non_nullable
as List<AskNote>,
  ));
}


}

/// @nodoc


class AskEvent_NothingMatches extends AskEvent {
  const AskEvent_NothingMatches({required this.message}): super._();
  

 final  String message;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskEvent_NothingMatchesCopyWith<AskEvent_NothingMatches> get copyWith => _$AskEvent_NothingMatchesCopyWithImpl<AskEvent_NothingMatches>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent_NothingMatches&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'AskEvent.nothingMatches(message: $message)';
}


}

/// @nodoc
abstract mixin class $AskEvent_NothingMatchesCopyWith<$Res> implements $AskEventCopyWith<$Res> {
  factory $AskEvent_NothingMatchesCopyWith(AskEvent_NothingMatches value, $Res Function(AskEvent_NothingMatches) _then) = _$AskEvent_NothingMatchesCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$AskEvent_NothingMatchesCopyWithImpl<$Res>
    implements $AskEvent_NothingMatchesCopyWith<$Res> {
  _$AskEvent_NothingMatchesCopyWithImpl(this._self, this._then);

  final AskEvent_NothingMatches _self;
  final $Res Function(AskEvent_NothingMatches) _then;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(AskEvent_NothingMatches(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AskEvent_CutOff extends AskEvent {
  const AskEvent_CutOff({required this.message}): super._();
  

 final  String message;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskEvent_CutOffCopyWith<AskEvent_CutOff> get copyWith => _$AskEvent_CutOffCopyWithImpl<AskEvent_CutOff>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent_CutOff&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'AskEvent.cutOff(message: $message)';
}


}

/// @nodoc
abstract mixin class $AskEvent_CutOffCopyWith<$Res> implements $AskEventCopyWith<$Res> {
  factory $AskEvent_CutOffCopyWith(AskEvent_CutOff value, $Res Function(AskEvent_CutOff) _then) = _$AskEvent_CutOffCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$AskEvent_CutOffCopyWithImpl<$Res>
    implements $AskEvent_CutOffCopyWith<$Res> {
  _$AskEvent_CutOffCopyWithImpl(this._self, this._then);

  final AskEvent_CutOff _self;
  final $Res Function(AskEvent_CutOff) _then;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(AskEvent_CutOff(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AskEvent_Failed extends AskEvent {
  const AskEvent_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AskEvent_FailedCopyWith<AskEvent_Failed> get copyWith => _$AskEvent_FailedCopyWithImpl<AskEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'AskEvent.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $AskEvent_FailedCopyWith<$Res> implements $AskEventCopyWith<$Res> {
  factory $AskEvent_FailedCopyWith(AskEvent_Failed value, $Res Function(AskEvent_Failed) _then) = _$AskEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$AskEvent_FailedCopyWithImpl<$Res>
    implements $AskEvent_FailedCopyWith<$Res> {
  _$AskEvent_FailedCopyWithImpl(this._self, this._then);

  final AskEvent_Failed _self;
  final $Res Function(AskEvent_Failed) _then;

/// Create a copy of AskEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(AskEvent_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AskEvent_Stopped extends AskEvent {
  const AskEvent_Stopped(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AskEvent_Stopped);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AskEvent.stopped()';
}


}




/// @nodoc
mixin _$AssistEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AssistEvent()';
}


}

/// @nodoc
class $AssistEventCopyWith<$Res>  {
$AssistEventCopyWith(AssistEvent _, $Res Function(AssistEvent) __);
}


/// Adds pattern-matching-related methods to [AssistEvent].
extension AssistEventPatterns on AssistEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AssistEvent_Status value)?  status,TResult Function( AssistEvent_Text value)?  text,TResult Function( AssistEvent_Done value)?  done,TResult Function( AssistEvent_CutOff value)?  cutOff,TResult Function( AssistEvent_Failed value)?  failed,TResult Function( AssistEvent_Stopped value)?  stopped,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AssistEvent_Status() when status != null:
return status(_that);case AssistEvent_Text() when text != null:
return text(_that);case AssistEvent_Done() when done != null:
return done(_that);case AssistEvent_CutOff() when cutOff != null:
return cutOff(_that);case AssistEvent_Failed() when failed != null:
return failed(_that);case AssistEvent_Stopped() when stopped != null:
return stopped(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AssistEvent_Status value)  status,required TResult Function( AssistEvent_Text value)  text,required TResult Function( AssistEvent_Done value)  done,required TResult Function( AssistEvent_CutOff value)  cutOff,required TResult Function( AssistEvent_Failed value)  failed,required TResult Function( AssistEvent_Stopped value)  stopped,}){
final _that = this;
switch (_that) {
case AssistEvent_Status():
return status(_that);case AssistEvent_Text():
return text(_that);case AssistEvent_Done():
return done(_that);case AssistEvent_CutOff():
return cutOff(_that);case AssistEvent_Failed():
return failed(_that);case AssistEvent_Stopped():
return stopped(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AssistEvent_Status value)?  status,TResult? Function( AssistEvent_Text value)?  text,TResult? Function( AssistEvent_Done value)?  done,TResult? Function( AssistEvent_CutOff value)?  cutOff,TResult? Function( AssistEvent_Failed value)?  failed,TResult? Function( AssistEvent_Stopped value)?  stopped,}){
final _that = this;
switch (_that) {
case AssistEvent_Status() when status != null:
return status(_that);case AssistEvent_Text() when text != null:
return text(_that);case AssistEvent_Done() when done != null:
return done(_that);case AssistEvent_CutOff() when cutOff != null:
return cutOff(_that);case AssistEvent_Failed() when failed != null:
return failed(_that);case AssistEvent_Stopped() when stopped != null:
return stopped(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String line)?  status,TResult Function( String text)?  text,TResult Function( String text)?  done,TResult Function( String message)?  cutOff,TResult Function( String message)?  failed,TResult Function()?  stopped,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AssistEvent_Status() when status != null:
return status(_that.line);case AssistEvent_Text() when text != null:
return text(_that.text);case AssistEvent_Done() when done != null:
return done(_that.text);case AssistEvent_CutOff() when cutOff != null:
return cutOff(_that.message);case AssistEvent_Failed() when failed != null:
return failed(_that.message);case AssistEvent_Stopped() when stopped != null:
return stopped();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String line)  status,required TResult Function( String text)  text,required TResult Function( String text)  done,required TResult Function( String message)  cutOff,required TResult Function( String message)  failed,required TResult Function()  stopped,}) {final _that = this;
switch (_that) {
case AssistEvent_Status():
return status(_that.line);case AssistEvent_Text():
return text(_that.text);case AssistEvent_Done():
return done(_that.text);case AssistEvent_CutOff():
return cutOff(_that.message);case AssistEvent_Failed():
return failed(_that.message);case AssistEvent_Stopped():
return stopped();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String line)?  status,TResult? Function( String text)?  text,TResult? Function( String text)?  done,TResult? Function( String message)?  cutOff,TResult? Function( String message)?  failed,TResult? Function()?  stopped,}) {final _that = this;
switch (_that) {
case AssistEvent_Status() when status != null:
return status(_that.line);case AssistEvent_Text() when text != null:
return text(_that.text);case AssistEvent_Done() when done != null:
return done(_that.text);case AssistEvent_CutOff() when cutOff != null:
return cutOff(_that.message);case AssistEvent_Failed() when failed != null:
return failed(_that.message);case AssistEvent_Stopped() when stopped != null:
return stopped();case _:
  return null;

}
}

}

/// @nodoc


class AssistEvent_Status extends AssistEvent {
  const AssistEvent_Status({required this.line}): super._();
  

 final  String line;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistEvent_StatusCopyWith<AssistEvent_Status> get copyWith => _$AssistEvent_StatusCopyWithImpl<AssistEvent_Status>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistEvent_Status&&(identical(other.line, line) || other.line == line));
}


@override
int get hashCode => Object.hash(runtimeType,line);

@override
String toString() {
  return 'AssistEvent.status(line: $line)';
}


}

/// @nodoc
abstract mixin class $AssistEvent_StatusCopyWith<$Res> implements $AssistEventCopyWith<$Res> {
  factory $AssistEvent_StatusCopyWith(AssistEvent_Status value, $Res Function(AssistEvent_Status) _then) = _$AssistEvent_StatusCopyWithImpl;
@useResult
$Res call({
 String line
});




}
/// @nodoc
class _$AssistEvent_StatusCopyWithImpl<$Res>
    implements $AssistEvent_StatusCopyWith<$Res> {
  _$AssistEvent_StatusCopyWithImpl(this._self, this._then);

  final AssistEvent_Status _self;
  final $Res Function(AssistEvent_Status) _then;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? line = null,}) {
  return _then(AssistEvent_Status(
line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AssistEvent_Text extends AssistEvent {
  const AssistEvent_Text({required this.text}): super._();
  

 final  String text;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistEvent_TextCopyWith<AssistEvent_Text> get copyWith => _$AssistEvent_TextCopyWithImpl<AssistEvent_Text>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistEvent_Text&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AssistEvent.text(text: $text)';
}


}

/// @nodoc
abstract mixin class $AssistEvent_TextCopyWith<$Res> implements $AssistEventCopyWith<$Res> {
  factory $AssistEvent_TextCopyWith(AssistEvent_Text value, $Res Function(AssistEvent_Text) _then) = _$AssistEvent_TextCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AssistEvent_TextCopyWithImpl<$Res>
    implements $AssistEvent_TextCopyWith<$Res> {
  _$AssistEvent_TextCopyWithImpl(this._self, this._then);

  final AssistEvent_Text _self;
  final $Res Function(AssistEvent_Text) _then;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AssistEvent_Text(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AssistEvent_Done extends AssistEvent {
  const AssistEvent_Done({required this.text}): super._();
  

 final  String text;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistEvent_DoneCopyWith<AssistEvent_Done> get copyWith => _$AssistEvent_DoneCopyWithImpl<AssistEvent_Done>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistEvent_Done&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AssistEvent.done(text: $text)';
}


}

/// @nodoc
abstract mixin class $AssistEvent_DoneCopyWith<$Res> implements $AssistEventCopyWith<$Res> {
  factory $AssistEvent_DoneCopyWith(AssistEvent_Done value, $Res Function(AssistEvent_Done) _then) = _$AssistEvent_DoneCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AssistEvent_DoneCopyWithImpl<$Res>
    implements $AssistEvent_DoneCopyWith<$Res> {
  _$AssistEvent_DoneCopyWithImpl(this._self, this._then);

  final AssistEvent_Done _self;
  final $Res Function(AssistEvent_Done) _then;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AssistEvent_Done(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AssistEvent_CutOff extends AssistEvent {
  const AssistEvent_CutOff({required this.message}): super._();
  

 final  String message;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistEvent_CutOffCopyWith<AssistEvent_CutOff> get copyWith => _$AssistEvent_CutOffCopyWithImpl<AssistEvent_CutOff>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistEvent_CutOff&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'AssistEvent.cutOff(message: $message)';
}


}

/// @nodoc
abstract mixin class $AssistEvent_CutOffCopyWith<$Res> implements $AssistEventCopyWith<$Res> {
  factory $AssistEvent_CutOffCopyWith(AssistEvent_CutOff value, $Res Function(AssistEvent_CutOff) _then) = _$AssistEvent_CutOffCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$AssistEvent_CutOffCopyWithImpl<$Res>
    implements $AssistEvent_CutOffCopyWith<$Res> {
  _$AssistEvent_CutOffCopyWithImpl(this._self, this._then);

  final AssistEvent_CutOff _self;
  final $Res Function(AssistEvent_CutOff) _then;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(AssistEvent_CutOff(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AssistEvent_Failed extends AssistEvent {
  const AssistEvent_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AssistEvent_FailedCopyWith<AssistEvent_Failed> get copyWith => _$AssistEvent_FailedCopyWithImpl<AssistEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistEvent_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'AssistEvent.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $AssistEvent_FailedCopyWith<$Res> implements $AssistEventCopyWith<$Res> {
  factory $AssistEvent_FailedCopyWith(AssistEvent_Failed value, $Res Function(AssistEvent_Failed) _then) = _$AssistEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$AssistEvent_FailedCopyWithImpl<$Res>
    implements $AssistEvent_FailedCopyWith<$Res> {
  _$AssistEvent_FailedCopyWithImpl(this._self, this._then);

  final AssistEvent_Failed _self;
  final $Res Function(AssistEvent_Failed) _then;

/// Create a copy of AssistEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(AssistEvent_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AssistEvent_Stopped extends AssistEvent {
  const AssistEvent_Stopped(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AssistEvent_Stopped);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AssistEvent.stopped()';
}


}




// dart format on

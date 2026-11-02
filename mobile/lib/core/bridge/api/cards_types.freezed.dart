// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'cards_types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$GradeEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GradeEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'GradeEvent()';
}


}

/// @nodoc
class $GradeEventCopyWith<$Res>  {
$GradeEventCopyWith(GradeEvent _, $Res Function(GradeEvent) __);
}


/// Adds pattern-matching-related methods to [GradeEvent].
extension GradeEventPatterns on GradeEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( GradeEvent_Graded value)?  graded,TResult Function( GradeEvent_Failed value)?  failed,TResult Function( GradeEvent_Stopped value)?  stopped,required TResult orElse(),}){
final _that = this;
switch (_that) {
case GradeEvent_Graded() when graded != null:
return graded(_that);case GradeEvent_Failed() when failed != null:
return failed(_that);case GradeEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( GradeEvent_Graded value)  graded,required TResult Function( GradeEvent_Failed value)  failed,required TResult Function( GradeEvent_Stopped value)  stopped,}){
final _that = this;
switch (_that) {
case GradeEvent_Graded():
return graded(_that);case GradeEvent_Failed():
return failed(_that);case GradeEvent_Stopped():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( GradeEvent_Graded value)?  graded,TResult? Function( GradeEvent_Failed value)?  failed,TResult? Function( GradeEvent_Stopped value)?  stopped,}){
final _that = this;
switch (_that) {
case GradeEvent_Graded() when graded != null:
return graded(_that);case GradeEvent_Failed() when failed != null:
return failed(_that);case GradeEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( QuizVerdict verdict,  String label,  String comment)?  graded,TResult Function( String message)?  failed,TResult Function()?  stopped,required TResult orElse(),}) {final _that = this;
switch (_that) {
case GradeEvent_Graded() when graded != null:
return graded(_that.verdict,_that.label,_that.comment);case GradeEvent_Failed() when failed != null:
return failed(_that.message);case GradeEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( QuizVerdict verdict,  String label,  String comment)  graded,required TResult Function( String message)  failed,required TResult Function()  stopped,}) {final _that = this;
switch (_that) {
case GradeEvent_Graded():
return graded(_that.verdict,_that.label,_that.comment);case GradeEvent_Failed():
return failed(_that.message);case GradeEvent_Stopped():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( QuizVerdict verdict,  String label,  String comment)?  graded,TResult? Function( String message)?  failed,TResult? Function()?  stopped,}) {final _that = this;
switch (_that) {
case GradeEvent_Graded() when graded != null:
return graded(_that.verdict,_that.label,_that.comment);case GradeEvent_Failed() when failed != null:
return failed(_that.message);case GradeEvent_Stopped() when stopped != null:
return stopped();case _:
  return null;

}
}

}

/// @nodoc


class GradeEvent_Graded extends GradeEvent {
  const GradeEvent_Graded({required this.verdict, required this.label, required this.comment}): super._();
  

 final  QuizVerdict verdict;
 final  String label;
 final  String comment;

/// Create a copy of GradeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$GradeEvent_GradedCopyWith<GradeEvent_Graded> get copyWith => _$GradeEvent_GradedCopyWithImpl<GradeEvent_Graded>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GradeEvent_Graded&&(identical(other.verdict, verdict) || other.verdict == verdict)&&(identical(other.label, label) || other.label == label)&&(identical(other.comment, comment) || other.comment == comment));
}


@override
int get hashCode => Object.hash(runtimeType,verdict,label,comment);

@override
String toString() {
  return 'GradeEvent.graded(verdict: $verdict, label: $label, comment: $comment)';
}


}

/// @nodoc
abstract mixin class $GradeEvent_GradedCopyWith<$Res> implements $GradeEventCopyWith<$Res> {
  factory $GradeEvent_GradedCopyWith(GradeEvent_Graded value, $Res Function(GradeEvent_Graded) _then) = _$GradeEvent_GradedCopyWithImpl;
@useResult
$Res call({
 QuizVerdict verdict, String label, String comment
});




}
/// @nodoc
class _$GradeEvent_GradedCopyWithImpl<$Res>
    implements $GradeEvent_GradedCopyWith<$Res> {
  _$GradeEvent_GradedCopyWithImpl(this._self, this._then);

  final GradeEvent_Graded _self;
  final $Res Function(GradeEvent_Graded) _then;

/// Create a copy of GradeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? verdict = null,Object? label = null,Object? comment = null,}) {
  return _then(GradeEvent_Graded(
verdict: null == verdict ? _self.verdict : verdict // ignore: cast_nullable_to_non_nullable
as QuizVerdict,label: null == label ? _self.label : label // ignore: cast_nullable_to_non_nullable
as String,comment: null == comment ? _self.comment : comment // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class GradeEvent_Failed extends GradeEvent {
  const GradeEvent_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of GradeEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$GradeEvent_FailedCopyWith<GradeEvent_Failed> get copyWith => _$GradeEvent_FailedCopyWithImpl<GradeEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GradeEvent_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'GradeEvent.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $GradeEvent_FailedCopyWith<$Res> implements $GradeEventCopyWith<$Res> {
  factory $GradeEvent_FailedCopyWith(GradeEvent_Failed value, $Res Function(GradeEvent_Failed) _then) = _$GradeEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$GradeEvent_FailedCopyWithImpl<$Res>
    implements $GradeEvent_FailedCopyWith<$Res> {
  _$GradeEvent_FailedCopyWithImpl(this._self, this._then);

  final GradeEvent_Failed _self;
  final $Res Function(GradeEvent_Failed) _then;

/// Create a copy of GradeEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(GradeEvent_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class GradeEvent_Stopped extends GradeEvent {
  const GradeEvent_Stopped(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is GradeEvent_Stopped);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'GradeEvent.stopped()';
}


}




/// @nodoc
mixin _$QuizEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuizEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'QuizEvent()';
}


}

/// @nodoc
class $QuizEventCopyWith<$Res>  {
$QuizEventCopyWith(QuizEvent _, $Res Function(QuizEvent) __);
}


/// Adds pattern-matching-related methods to [QuizEvent].
extension QuizEventPatterns on QuizEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( QuizEvent_Status value)?  status,TResult Function( QuizEvent_Questions value)?  questions,TResult Function( QuizEvent_Failed value)?  failed,TResult Function( QuizEvent_Stopped value)?  stopped,required TResult orElse(),}){
final _that = this;
switch (_that) {
case QuizEvent_Status() when status != null:
return status(_that);case QuizEvent_Questions() when questions != null:
return questions(_that);case QuizEvent_Failed() when failed != null:
return failed(_that);case QuizEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( QuizEvent_Status value)  status,required TResult Function( QuizEvent_Questions value)  questions,required TResult Function( QuizEvent_Failed value)  failed,required TResult Function( QuizEvent_Stopped value)  stopped,}){
final _that = this;
switch (_that) {
case QuizEvent_Status():
return status(_that);case QuizEvent_Questions():
return questions(_that);case QuizEvent_Failed():
return failed(_that);case QuizEvent_Stopped():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( QuizEvent_Status value)?  status,TResult? Function( QuizEvent_Questions value)?  questions,TResult? Function( QuizEvent_Failed value)?  failed,TResult? Function( QuizEvent_Stopped value)?  stopped,}){
final _that = this;
switch (_that) {
case QuizEvent_Status() when status != null:
return status(_that);case QuizEvent_Questions() when questions != null:
return questions(_that);case QuizEvent_Failed() when failed != null:
return failed(_that);case QuizEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String line)?  status,TResult Function( List<QuizItem> items)?  questions,TResult Function( String message)?  failed,TResult Function()?  stopped,required TResult orElse(),}) {final _that = this;
switch (_that) {
case QuizEvent_Status() when status != null:
return status(_that.line);case QuizEvent_Questions() when questions != null:
return questions(_that.items);case QuizEvent_Failed() when failed != null:
return failed(_that.message);case QuizEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String line)  status,required TResult Function( List<QuizItem> items)  questions,required TResult Function( String message)  failed,required TResult Function()  stopped,}) {final _that = this;
switch (_that) {
case QuizEvent_Status():
return status(_that.line);case QuizEvent_Questions():
return questions(_that.items);case QuizEvent_Failed():
return failed(_that.message);case QuizEvent_Stopped():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String line)?  status,TResult? Function( List<QuizItem> items)?  questions,TResult? Function( String message)?  failed,TResult? Function()?  stopped,}) {final _that = this;
switch (_that) {
case QuizEvent_Status() when status != null:
return status(_that.line);case QuizEvent_Questions() when questions != null:
return questions(_that.items);case QuizEvent_Failed() when failed != null:
return failed(_that.message);case QuizEvent_Stopped() when stopped != null:
return stopped();case _:
  return null;

}
}

}

/// @nodoc


class QuizEvent_Status extends QuizEvent {
  const QuizEvent_Status({required this.line}): super._();
  

 final  String line;

/// Create a copy of QuizEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QuizEvent_StatusCopyWith<QuizEvent_Status> get copyWith => _$QuizEvent_StatusCopyWithImpl<QuizEvent_Status>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuizEvent_Status&&(identical(other.line, line) || other.line == line));
}


@override
int get hashCode => Object.hash(runtimeType,line);

@override
String toString() {
  return 'QuizEvent.status(line: $line)';
}


}

/// @nodoc
abstract mixin class $QuizEvent_StatusCopyWith<$Res> implements $QuizEventCopyWith<$Res> {
  factory $QuizEvent_StatusCopyWith(QuizEvent_Status value, $Res Function(QuizEvent_Status) _then) = _$QuizEvent_StatusCopyWithImpl;
@useResult
$Res call({
 String line
});




}
/// @nodoc
class _$QuizEvent_StatusCopyWithImpl<$Res>
    implements $QuizEvent_StatusCopyWith<$Res> {
  _$QuizEvent_StatusCopyWithImpl(this._self, this._then);

  final QuizEvent_Status _self;
  final $Res Function(QuizEvent_Status) _then;

/// Create a copy of QuizEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? line = null,}) {
  return _then(QuizEvent_Status(
line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class QuizEvent_Questions extends QuizEvent {
  const QuizEvent_Questions({required final  List<QuizItem> items}): _items = items,super._();
  

 final  List<QuizItem> _items;
 List<QuizItem> get items {
  if (_items is EqualUnmodifiableListView) return _items;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_items);
}


/// Create a copy of QuizEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QuizEvent_QuestionsCopyWith<QuizEvent_Questions> get copyWith => _$QuizEvent_QuestionsCopyWithImpl<QuizEvent_Questions>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuizEvent_Questions&&const DeepCollectionEquality().equals(other._items, _items));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_items));

@override
String toString() {
  return 'QuizEvent.questions(items: $items)';
}


}

/// @nodoc
abstract mixin class $QuizEvent_QuestionsCopyWith<$Res> implements $QuizEventCopyWith<$Res> {
  factory $QuizEvent_QuestionsCopyWith(QuizEvent_Questions value, $Res Function(QuizEvent_Questions) _then) = _$QuizEvent_QuestionsCopyWithImpl;
@useResult
$Res call({
 List<QuizItem> items
});




}
/// @nodoc
class _$QuizEvent_QuestionsCopyWithImpl<$Res>
    implements $QuizEvent_QuestionsCopyWith<$Res> {
  _$QuizEvent_QuestionsCopyWithImpl(this._self, this._then);

  final QuizEvent_Questions _self;
  final $Res Function(QuizEvent_Questions) _then;

/// Create a copy of QuizEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? items = null,}) {
  return _then(QuizEvent_Questions(
items: null == items ? _self._items : items // ignore: cast_nullable_to_non_nullable
as List<QuizItem>,
  ));
}


}

/// @nodoc


class QuizEvent_Failed extends QuizEvent {
  const QuizEvent_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of QuizEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$QuizEvent_FailedCopyWith<QuizEvent_Failed> get copyWith => _$QuizEvent_FailedCopyWithImpl<QuizEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuizEvent_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'QuizEvent.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $QuizEvent_FailedCopyWith<$Res> implements $QuizEventCopyWith<$Res> {
  factory $QuizEvent_FailedCopyWith(QuizEvent_Failed value, $Res Function(QuizEvent_Failed) _then) = _$QuizEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$QuizEvent_FailedCopyWithImpl<$Res>
    implements $QuizEvent_FailedCopyWith<$Res> {
  _$QuizEvent_FailedCopyWithImpl(this._self, this._then);

  final QuizEvent_Failed _self;
  final $Res Function(QuizEvent_Failed) _then;

/// Create a copy of QuizEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(QuizEvent_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class QuizEvent_Stopped extends QuizEvent {
  const QuizEvent_Stopped(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is QuizEvent_Stopped);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'QuizEvent.stopped()';
}


}




/// @nodoc
mixin _$SuggestEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SuggestEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SuggestEvent()';
}


}

/// @nodoc
class $SuggestEventCopyWith<$Res>  {
$SuggestEventCopyWith(SuggestEvent _, $Res Function(SuggestEvent) __);
}


/// Adds pattern-matching-related methods to [SuggestEvent].
extension SuggestEventPatterns on SuggestEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SuggestEvent_Status value)?  status,TResult Function( SuggestEvent_Done value)?  done,TResult Function( SuggestEvent_Failed value)?  failed,TResult Function( SuggestEvent_Stopped value)?  stopped,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SuggestEvent_Status() when status != null:
return status(_that);case SuggestEvent_Done() when done != null:
return done(_that);case SuggestEvent_Failed() when failed != null:
return failed(_that);case SuggestEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SuggestEvent_Status value)  status,required TResult Function( SuggestEvent_Done value)  done,required TResult Function( SuggestEvent_Failed value)  failed,required TResult Function( SuggestEvent_Stopped value)  stopped,}){
final _that = this;
switch (_that) {
case SuggestEvent_Status():
return status(_that);case SuggestEvent_Done():
return done(_that);case SuggestEvent_Failed():
return failed(_that);case SuggestEvent_Stopped():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SuggestEvent_Status value)?  status,TResult? Function( SuggestEvent_Done value)?  done,TResult? Function( SuggestEvent_Failed value)?  failed,TResult? Function( SuggestEvent_Stopped value)?  stopped,}){
final _that = this;
switch (_that) {
case SuggestEvent_Status() when status != null:
return status(_that);case SuggestEvent_Done() when done != null:
return done(_that);case SuggestEvent_Failed() when failed != null:
return failed(_that);case SuggestEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String line)?  status,TResult Function( List<SuggestedCard> cards)?  done,TResult Function( String message)?  failed,TResult Function()?  stopped,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SuggestEvent_Status() when status != null:
return status(_that.line);case SuggestEvent_Done() when done != null:
return done(_that.cards);case SuggestEvent_Failed() when failed != null:
return failed(_that.message);case SuggestEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String line)  status,required TResult Function( List<SuggestedCard> cards)  done,required TResult Function( String message)  failed,required TResult Function()  stopped,}) {final _that = this;
switch (_that) {
case SuggestEvent_Status():
return status(_that.line);case SuggestEvent_Done():
return done(_that.cards);case SuggestEvent_Failed():
return failed(_that.message);case SuggestEvent_Stopped():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String line)?  status,TResult? Function( List<SuggestedCard> cards)?  done,TResult? Function( String message)?  failed,TResult? Function()?  stopped,}) {final _that = this;
switch (_that) {
case SuggestEvent_Status() when status != null:
return status(_that.line);case SuggestEvent_Done() when done != null:
return done(_that.cards);case SuggestEvent_Failed() when failed != null:
return failed(_that.message);case SuggestEvent_Stopped() when stopped != null:
return stopped();case _:
  return null;

}
}

}

/// @nodoc


class SuggestEvent_Status extends SuggestEvent {
  const SuggestEvent_Status({required this.line}): super._();
  

 final  String line;

/// Create a copy of SuggestEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SuggestEvent_StatusCopyWith<SuggestEvent_Status> get copyWith => _$SuggestEvent_StatusCopyWithImpl<SuggestEvent_Status>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SuggestEvent_Status&&(identical(other.line, line) || other.line == line));
}


@override
int get hashCode => Object.hash(runtimeType,line);

@override
String toString() {
  return 'SuggestEvent.status(line: $line)';
}


}

/// @nodoc
abstract mixin class $SuggestEvent_StatusCopyWith<$Res> implements $SuggestEventCopyWith<$Res> {
  factory $SuggestEvent_StatusCopyWith(SuggestEvent_Status value, $Res Function(SuggestEvent_Status) _then) = _$SuggestEvent_StatusCopyWithImpl;
@useResult
$Res call({
 String line
});




}
/// @nodoc
class _$SuggestEvent_StatusCopyWithImpl<$Res>
    implements $SuggestEvent_StatusCopyWith<$Res> {
  _$SuggestEvent_StatusCopyWithImpl(this._self, this._then);

  final SuggestEvent_Status _self;
  final $Res Function(SuggestEvent_Status) _then;

/// Create a copy of SuggestEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? line = null,}) {
  return _then(SuggestEvent_Status(
line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SuggestEvent_Done extends SuggestEvent {
  const SuggestEvent_Done({required final  List<SuggestedCard> cards}): _cards = cards,super._();
  

 final  List<SuggestedCard> _cards;
 List<SuggestedCard> get cards {
  if (_cards is EqualUnmodifiableListView) return _cards;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_cards);
}


/// Create a copy of SuggestEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SuggestEvent_DoneCopyWith<SuggestEvent_Done> get copyWith => _$SuggestEvent_DoneCopyWithImpl<SuggestEvent_Done>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SuggestEvent_Done&&const DeepCollectionEquality().equals(other._cards, _cards));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_cards));

@override
String toString() {
  return 'SuggestEvent.done(cards: $cards)';
}


}

/// @nodoc
abstract mixin class $SuggestEvent_DoneCopyWith<$Res> implements $SuggestEventCopyWith<$Res> {
  factory $SuggestEvent_DoneCopyWith(SuggestEvent_Done value, $Res Function(SuggestEvent_Done) _then) = _$SuggestEvent_DoneCopyWithImpl;
@useResult
$Res call({
 List<SuggestedCard> cards
});




}
/// @nodoc
class _$SuggestEvent_DoneCopyWithImpl<$Res>
    implements $SuggestEvent_DoneCopyWith<$Res> {
  _$SuggestEvent_DoneCopyWithImpl(this._self, this._then);

  final SuggestEvent_Done _self;
  final $Res Function(SuggestEvent_Done) _then;

/// Create a copy of SuggestEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? cards = null,}) {
  return _then(SuggestEvent_Done(
cards: null == cards ? _self._cards : cards // ignore: cast_nullable_to_non_nullable
as List<SuggestedCard>,
  ));
}


}

/// @nodoc


class SuggestEvent_Failed extends SuggestEvent {
  const SuggestEvent_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of SuggestEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SuggestEvent_FailedCopyWith<SuggestEvent_Failed> get copyWith => _$SuggestEvent_FailedCopyWithImpl<SuggestEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SuggestEvent_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'SuggestEvent.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $SuggestEvent_FailedCopyWith<$Res> implements $SuggestEventCopyWith<$Res> {
  factory $SuggestEvent_FailedCopyWith(SuggestEvent_Failed value, $Res Function(SuggestEvent_Failed) _then) = _$SuggestEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$SuggestEvent_FailedCopyWithImpl<$Res>
    implements $SuggestEvent_FailedCopyWith<$Res> {
  _$SuggestEvent_FailedCopyWithImpl(this._self, this._then);

  final SuggestEvent_Failed _self;
  final $Res Function(SuggestEvent_Failed) _then;

/// Create a copy of SuggestEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(SuggestEvent_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class SuggestEvent_Stopped extends SuggestEvent {
  const SuggestEvent_Stopped(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SuggestEvent_Stopped);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SuggestEvent.stopped()';
}


}




// dart format on

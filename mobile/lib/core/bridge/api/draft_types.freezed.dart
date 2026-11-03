// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'draft_types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$DraftEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DraftEvent()';
}


}

/// @nodoc
class $DraftEventCopyWith<$Res>  {
$DraftEventCopyWith(DraftEvent _, $Res Function(DraftEvent) __);
}


/// Adds pattern-matching-related methods to [DraftEvent].
extension DraftEventPatterns on DraftEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( DraftEvent_Status value)?  status,TResult Function( DraftEvent_Text value)?  text,TResult Function( DraftEvent_Done value)?  done,TResult Function( DraftEvent_CutOff value)?  cutOff,TResult Function( DraftEvent_Failed value)?  failed,TResult Function( DraftEvent_Stopped value)?  stopped,required TResult orElse(),}){
final _that = this;
switch (_that) {
case DraftEvent_Status() when status != null:
return status(_that);case DraftEvent_Text() when text != null:
return text(_that);case DraftEvent_Done() when done != null:
return done(_that);case DraftEvent_CutOff() when cutOff != null:
return cutOff(_that);case DraftEvent_Failed() when failed != null:
return failed(_that);case DraftEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( DraftEvent_Status value)  status,required TResult Function( DraftEvent_Text value)  text,required TResult Function( DraftEvent_Done value)  done,required TResult Function( DraftEvent_CutOff value)  cutOff,required TResult Function( DraftEvent_Failed value)  failed,required TResult Function( DraftEvent_Stopped value)  stopped,}){
final _that = this;
switch (_that) {
case DraftEvent_Status():
return status(_that);case DraftEvent_Text():
return text(_that);case DraftEvent_Done():
return done(_that);case DraftEvent_CutOff():
return cutOff(_that);case DraftEvent_Failed():
return failed(_that);case DraftEvent_Stopped():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( DraftEvent_Status value)?  status,TResult? Function( DraftEvent_Text value)?  text,TResult? Function( DraftEvent_Done value)?  done,TResult? Function( DraftEvent_CutOff value)?  cutOff,TResult? Function( DraftEvent_Failed value)?  failed,TResult? Function( DraftEvent_Stopped value)?  stopped,}){
final _that = this;
switch (_that) {
case DraftEvent_Status() when status != null:
return status(_that);case DraftEvent_Text() when text != null:
return text(_that);case DraftEvent_Done() when done != null:
return done(_that);case DraftEvent_CutOff() when cutOff != null:
return cutOff(_that);case DraftEvent_Failed() when failed != null:
return failed(_that);case DraftEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String line)?  status,TResult Function( DraftView draft)?  text,TResult Function( DraftView draft,  List<DraftSourceRef> sources)?  done,TResult Function( String message)?  cutOff,TResult Function( String message)?  failed,TResult Function()?  stopped,required TResult orElse(),}) {final _that = this;
switch (_that) {
case DraftEvent_Status() when status != null:
return status(_that.line);case DraftEvent_Text() when text != null:
return text(_that.draft);case DraftEvent_Done() when done != null:
return done(_that.draft,_that.sources);case DraftEvent_CutOff() when cutOff != null:
return cutOff(_that.message);case DraftEvent_Failed() when failed != null:
return failed(_that.message);case DraftEvent_Stopped() when stopped != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String line)  status,required TResult Function( DraftView draft)  text,required TResult Function( DraftView draft,  List<DraftSourceRef> sources)  done,required TResult Function( String message)  cutOff,required TResult Function( String message)  failed,required TResult Function()  stopped,}) {final _that = this;
switch (_that) {
case DraftEvent_Status():
return status(_that.line);case DraftEvent_Text():
return text(_that.draft);case DraftEvent_Done():
return done(_that.draft,_that.sources);case DraftEvent_CutOff():
return cutOff(_that.message);case DraftEvent_Failed():
return failed(_that.message);case DraftEvent_Stopped():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String line)?  status,TResult? Function( DraftView draft)?  text,TResult? Function( DraftView draft,  List<DraftSourceRef> sources)?  done,TResult? Function( String message)?  cutOff,TResult? Function( String message)?  failed,TResult? Function()?  stopped,}) {final _that = this;
switch (_that) {
case DraftEvent_Status() when status != null:
return status(_that.line);case DraftEvent_Text() when text != null:
return text(_that.draft);case DraftEvent_Done() when done != null:
return done(_that.draft,_that.sources);case DraftEvent_CutOff() when cutOff != null:
return cutOff(_that.message);case DraftEvent_Failed() when failed != null:
return failed(_that.message);case DraftEvent_Stopped() when stopped != null:
return stopped();case _:
  return null;

}
}

}

/// @nodoc


class DraftEvent_Status extends DraftEvent {
  const DraftEvent_Status({required this.line}): super._();
  

 final  String line;

/// Create a copy of DraftEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DraftEvent_StatusCopyWith<DraftEvent_Status> get copyWith => _$DraftEvent_StatusCopyWithImpl<DraftEvent_Status>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftEvent_Status&&(identical(other.line, line) || other.line == line));
}


@override
int get hashCode => Object.hash(runtimeType,line);

@override
String toString() {
  return 'DraftEvent.status(line: $line)';
}


}

/// @nodoc
abstract mixin class $DraftEvent_StatusCopyWith<$Res> implements $DraftEventCopyWith<$Res> {
  factory $DraftEvent_StatusCopyWith(DraftEvent_Status value, $Res Function(DraftEvent_Status) _then) = _$DraftEvent_StatusCopyWithImpl;
@useResult
$Res call({
 String line
});




}
/// @nodoc
class _$DraftEvent_StatusCopyWithImpl<$Res>
    implements $DraftEvent_StatusCopyWith<$Res> {
  _$DraftEvent_StatusCopyWithImpl(this._self, this._then);

  final DraftEvent_Status _self;
  final $Res Function(DraftEvent_Status) _then;

/// Create a copy of DraftEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? line = null,}) {
  return _then(DraftEvent_Status(
line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class DraftEvent_Text extends DraftEvent {
  const DraftEvent_Text({required this.draft}): super._();
  

 final  DraftView draft;

/// Create a copy of DraftEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DraftEvent_TextCopyWith<DraftEvent_Text> get copyWith => _$DraftEvent_TextCopyWithImpl<DraftEvent_Text>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftEvent_Text&&(identical(other.draft, draft) || other.draft == draft));
}


@override
int get hashCode => Object.hash(runtimeType,draft);

@override
String toString() {
  return 'DraftEvent.text(draft: $draft)';
}


}

/// @nodoc
abstract mixin class $DraftEvent_TextCopyWith<$Res> implements $DraftEventCopyWith<$Res> {
  factory $DraftEvent_TextCopyWith(DraftEvent_Text value, $Res Function(DraftEvent_Text) _then) = _$DraftEvent_TextCopyWithImpl;
@useResult
$Res call({
 DraftView draft
});




}
/// @nodoc
class _$DraftEvent_TextCopyWithImpl<$Res>
    implements $DraftEvent_TextCopyWith<$Res> {
  _$DraftEvent_TextCopyWithImpl(this._self, this._then);

  final DraftEvent_Text _self;
  final $Res Function(DraftEvent_Text) _then;

/// Create a copy of DraftEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? draft = null,}) {
  return _then(DraftEvent_Text(
draft: null == draft ? _self.draft : draft // ignore: cast_nullable_to_non_nullable
as DraftView,
  ));
}


}

/// @nodoc


class DraftEvent_Done extends DraftEvent {
  const DraftEvent_Done({required this.draft, required final  List<DraftSourceRef> sources}): _sources = sources,super._();
  

 final  DraftView draft;
 final  List<DraftSourceRef> _sources;
 List<DraftSourceRef> get sources {
  if (_sources is EqualUnmodifiableListView) return _sources;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_sources);
}


/// Create a copy of DraftEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DraftEvent_DoneCopyWith<DraftEvent_Done> get copyWith => _$DraftEvent_DoneCopyWithImpl<DraftEvent_Done>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftEvent_Done&&(identical(other.draft, draft) || other.draft == draft)&&const DeepCollectionEquality().equals(other._sources, _sources));
}


@override
int get hashCode => Object.hash(runtimeType,draft,const DeepCollectionEquality().hash(_sources));

@override
String toString() {
  return 'DraftEvent.done(draft: $draft, sources: $sources)';
}


}

/// @nodoc
abstract mixin class $DraftEvent_DoneCopyWith<$Res> implements $DraftEventCopyWith<$Res> {
  factory $DraftEvent_DoneCopyWith(DraftEvent_Done value, $Res Function(DraftEvent_Done) _then) = _$DraftEvent_DoneCopyWithImpl;
@useResult
$Res call({
 DraftView draft, List<DraftSourceRef> sources
});




}
/// @nodoc
class _$DraftEvent_DoneCopyWithImpl<$Res>
    implements $DraftEvent_DoneCopyWith<$Res> {
  _$DraftEvent_DoneCopyWithImpl(this._self, this._then);

  final DraftEvent_Done _self;
  final $Res Function(DraftEvent_Done) _then;

/// Create a copy of DraftEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? draft = null,Object? sources = null,}) {
  return _then(DraftEvent_Done(
draft: null == draft ? _self.draft : draft // ignore: cast_nullable_to_non_nullable
as DraftView,sources: null == sources ? _self._sources : sources // ignore: cast_nullable_to_non_nullable
as List<DraftSourceRef>,
  ));
}


}

/// @nodoc


class DraftEvent_CutOff extends DraftEvent {
  const DraftEvent_CutOff({required this.message}): super._();
  

 final  String message;

/// Create a copy of DraftEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DraftEvent_CutOffCopyWith<DraftEvent_CutOff> get copyWith => _$DraftEvent_CutOffCopyWithImpl<DraftEvent_CutOff>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftEvent_CutOff&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'DraftEvent.cutOff(message: $message)';
}


}

/// @nodoc
abstract mixin class $DraftEvent_CutOffCopyWith<$Res> implements $DraftEventCopyWith<$Res> {
  factory $DraftEvent_CutOffCopyWith(DraftEvent_CutOff value, $Res Function(DraftEvent_CutOff) _then) = _$DraftEvent_CutOffCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$DraftEvent_CutOffCopyWithImpl<$Res>
    implements $DraftEvent_CutOffCopyWith<$Res> {
  _$DraftEvent_CutOffCopyWithImpl(this._self, this._then);

  final DraftEvent_CutOff _self;
  final $Res Function(DraftEvent_CutOff) _then;

/// Create a copy of DraftEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(DraftEvent_CutOff(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class DraftEvent_Failed extends DraftEvent {
  const DraftEvent_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of DraftEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$DraftEvent_FailedCopyWith<DraftEvent_Failed> get copyWith => _$DraftEvent_FailedCopyWithImpl<DraftEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftEvent_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'DraftEvent.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $DraftEvent_FailedCopyWith<$Res> implements $DraftEventCopyWith<$Res> {
  factory $DraftEvent_FailedCopyWith(DraftEvent_Failed value, $Res Function(DraftEvent_Failed) _then) = _$DraftEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$DraftEvent_FailedCopyWithImpl<$Res>
    implements $DraftEvent_FailedCopyWith<$Res> {
  _$DraftEvent_FailedCopyWithImpl(this._self, this._then);

  final DraftEvent_Failed _self;
  final $Res Function(DraftEvent_Failed) _then;

/// Create a copy of DraftEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(DraftEvent_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class DraftEvent_Stopped extends DraftEvent {
  const DraftEvent_Stopped(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is DraftEvent_Stopped);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'DraftEvent.stopped()';
}


}




// dart format on

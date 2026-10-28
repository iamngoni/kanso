// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'answer_span.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AnswerSpan {

 String get text;
/// Create a copy of AnswerSpan
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AnswerSpanCopyWith<AnswerSpan> get copyWith => _$AnswerSpanCopyWithImpl<AnswerSpan>(this as AnswerSpan, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerSpan&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AnswerSpan(text: $text)';
}


}

/// @nodoc
abstract mixin class $AnswerSpanCopyWith<$Res>  {
  factory $AnswerSpanCopyWith(AnswerSpan value, $Res Function(AnswerSpan) _then) = _$AnswerSpanCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AnswerSpanCopyWithImpl<$Res>
    implements $AnswerSpanCopyWith<$Res> {
  _$AnswerSpanCopyWithImpl(this._self, this._then);

  final AnswerSpan _self;
  final $Res Function(AnswerSpan) _then;

/// Create a copy of AnswerSpan
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? text = null,}) {
  return _then(_self.copyWith(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

}


/// Adds pattern-matching-related methods to [AnswerSpan].
extension AnswerSpanPatterns on AnswerSpan {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AnswerText value)?  text,TResult Function( AnswerNoteLink value)?  note,TResult Function( AnswerWebLink value)?  web,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AnswerText() when text != null:
return text(_that);case AnswerNoteLink() when note != null:
return note(_that);case AnswerWebLink() when web != null:
return web(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AnswerText value)  text,required TResult Function( AnswerNoteLink value)  note,required TResult Function( AnswerWebLink value)  web,}){
final _that = this;
switch (_that) {
case AnswerText():
return text(_that);case AnswerNoteLink():
return note(_that);case AnswerWebLink():
return web(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AnswerText value)?  text,TResult? Function( AnswerNoteLink value)?  note,TResult? Function( AnswerWebLink value)?  web,}){
final _that = this;
switch (_that) {
case AnswerText() when text != null:
return text(_that);case AnswerNoteLink() when note != null:
return note(_that);case AnswerWebLink() when web != null:
return web(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String text,  bool bold,  bool italic,  bool code)?  text,TResult Function( String text,  String noteId)?  note,TResult Function( String text,  String url)?  web,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AnswerText() when text != null:
return text(_that.text,_that.bold,_that.italic,_that.code);case AnswerNoteLink() when note != null:
return note(_that.text,_that.noteId);case AnswerWebLink() when web != null:
return web(_that.text,_that.url);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String text,  bool bold,  bool italic,  bool code)  text,required TResult Function( String text,  String noteId)  note,required TResult Function( String text,  String url)  web,}) {final _that = this;
switch (_that) {
case AnswerText():
return text(_that.text,_that.bold,_that.italic,_that.code);case AnswerNoteLink():
return note(_that.text,_that.noteId);case AnswerWebLink():
return web(_that.text,_that.url);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String text,  bool bold,  bool italic,  bool code)?  text,TResult? Function( String text,  String noteId)?  note,TResult? Function( String text,  String url)?  web,}) {final _that = this;
switch (_that) {
case AnswerText() when text != null:
return text(_that.text,_that.bold,_that.italic,_that.code);case AnswerNoteLink() when note != null:
return note(_that.text,_that.noteId);case AnswerWebLink() when web != null:
return web(_that.text,_that.url);case _:
  return null;

}
}

}

/// @nodoc


class AnswerText implements AnswerSpan {
  const AnswerText(this.text, {this.bold = false, this.italic = false, this.code = false});
  

@override final  String text;
@JsonKey() final  bool bold;
@JsonKey() final  bool italic;
@JsonKey() final  bool code;

/// Create a copy of AnswerSpan
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AnswerTextCopyWith<AnswerText> get copyWith => _$AnswerTextCopyWithImpl<AnswerText>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerText&&(identical(other.text, text) || other.text == text)&&(identical(other.bold, bold) || other.bold == bold)&&(identical(other.italic, italic) || other.italic == italic)&&(identical(other.code, code) || other.code == code));
}


@override
int get hashCode => Object.hash(runtimeType,text,bold,italic,code);

@override
String toString() {
  return 'AnswerSpan.text(text: $text, bold: $bold, italic: $italic, code: $code)';
}


}

/// @nodoc
abstract mixin class $AnswerTextCopyWith<$Res> implements $AnswerSpanCopyWith<$Res> {
  factory $AnswerTextCopyWith(AnswerText value, $Res Function(AnswerText) _then) = _$AnswerTextCopyWithImpl;
@override @useResult
$Res call({
 String text, bool bold, bool italic, bool code
});




}
/// @nodoc
class _$AnswerTextCopyWithImpl<$Res>
    implements $AnswerTextCopyWith<$Res> {
  _$AnswerTextCopyWithImpl(this._self, this._then);

  final AnswerText _self;
  final $Res Function(AnswerText) _then;

/// Create a copy of AnswerSpan
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? text = null,Object? bold = null,Object? italic = null,Object? code = null,}) {
  return _then(AnswerText(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,bold: null == bold ? _self.bold : bold // ignore: cast_nullable_to_non_nullable
as bool,italic: null == italic ? _self.italic : italic // ignore: cast_nullable_to_non_nullable
as bool,code: null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class AnswerNoteLink implements AnswerSpan {
  const AnswerNoteLink({required this.text, required this.noteId});
  

@override final  String text;
 final  String noteId;

/// Create a copy of AnswerSpan
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AnswerNoteLinkCopyWith<AnswerNoteLink> get copyWith => _$AnswerNoteLinkCopyWithImpl<AnswerNoteLink>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerNoteLink&&(identical(other.text, text) || other.text == text)&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,text,noteId);

@override
String toString() {
  return 'AnswerSpan.note(text: $text, noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $AnswerNoteLinkCopyWith<$Res> implements $AnswerSpanCopyWith<$Res> {
  factory $AnswerNoteLinkCopyWith(AnswerNoteLink value, $Res Function(AnswerNoteLink) _then) = _$AnswerNoteLinkCopyWithImpl;
@override @useResult
$Res call({
 String text, String noteId
});




}
/// @nodoc
class _$AnswerNoteLinkCopyWithImpl<$Res>
    implements $AnswerNoteLinkCopyWith<$Res> {
  _$AnswerNoteLinkCopyWithImpl(this._self, this._then);

  final AnswerNoteLink _self;
  final $Res Function(AnswerNoteLink) _then;

/// Create a copy of AnswerSpan
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? text = null,Object? noteId = null,}) {
  return _then(AnswerNoteLink(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AnswerWebLink implements AnswerSpan {
  const AnswerWebLink({required this.text, required this.url});
  

@override final  String text;
 final  String url;

/// Create a copy of AnswerSpan
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AnswerWebLinkCopyWith<AnswerWebLink> get copyWith => _$AnswerWebLinkCopyWithImpl<AnswerWebLink>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerWebLink&&(identical(other.text, text) || other.text == text)&&(identical(other.url, url) || other.url == url));
}


@override
int get hashCode => Object.hash(runtimeType,text,url);

@override
String toString() {
  return 'AnswerSpan.web(text: $text, url: $url)';
}


}

/// @nodoc
abstract mixin class $AnswerWebLinkCopyWith<$Res> implements $AnswerSpanCopyWith<$Res> {
  factory $AnswerWebLinkCopyWith(AnswerWebLink value, $Res Function(AnswerWebLink) _then) = _$AnswerWebLinkCopyWithImpl;
@override @useResult
$Res call({
 String text, String url
});




}
/// @nodoc
class _$AnswerWebLinkCopyWithImpl<$Res>
    implements $AnswerWebLinkCopyWith<$Res> {
  _$AnswerWebLinkCopyWithImpl(this._self, this._then);

  final AnswerWebLink _self;
  final $Res Function(AnswerWebLink) _then;

/// Create a copy of AnswerSpan
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? text = null,Object? url = null,}) {
  return _then(AnswerWebLink(
text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,url: null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on

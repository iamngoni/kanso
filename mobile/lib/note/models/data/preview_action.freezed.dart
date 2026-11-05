// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'preview_action.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PreviewAction {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewAction);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PreviewAction()';
}


}

/// @nodoc
class $PreviewActionCopyWith<$Res>  {
$PreviewActionCopyWith(PreviewAction _, $Res Function(PreviewAction) __);
}


/// Adds pattern-matching-related methods to [PreviewAction].
extension PreviewActionPatterns on PreviewAction {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PreviewOpenNote value)?  openNote,TResult Function( PreviewOpenTag value)?  openTag,TResult Function( PreviewScrollTo value)?  scrollTo,TResult Function( PreviewShowAttachment value)?  showAttachment,TResult Function( PreviewOpenPdf value)?  openPdf,TResult Function( PreviewCopyCode value)?  copyCode,TResult Function( PreviewPlayMoment value)?  playMoment,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PreviewOpenNote() when openNote != null:
return openNote(_that);case PreviewOpenTag() when openTag != null:
return openTag(_that);case PreviewScrollTo() when scrollTo != null:
return scrollTo(_that);case PreviewShowAttachment() when showAttachment != null:
return showAttachment(_that);case PreviewOpenPdf() when openPdf != null:
return openPdf(_that);case PreviewCopyCode() when copyCode != null:
return copyCode(_that);case PreviewPlayMoment() when playMoment != null:
return playMoment(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PreviewOpenNote value)  openNote,required TResult Function( PreviewOpenTag value)  openTag,required TResult Function( PreviewScrollTo value)  scrollTo,required TResult Function( PreviewShowAttachment value)  showAttachment,required TResult Function( PreviewOpenPdf value)  openPdf,required TResult Function( PreviewCopyCode value)  copyCode,required TResult Function( PreviewPlayMoment value)  playMoment,}){
final _that = this;
switch (_that) {
case PreviewOpenNote():
return openNote(_that);case PreviewOpenTag():
return openTag(_that);case PreviewScrollTo():
return scrollTo(_that);case PreviewShowAttachment():
return showAttachment(_that);case PreviewOpenPdf():
return openPdf(_that);case PreviewCopyCode():
return copyCode(_that);case PreviewPlayMoment():
return playMoment(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PreviewOpenNote value)?  openNote,TResult? Function( PreviewOpenTag value)?  openTag,TResult? Function( PreviewScrollTo value)?  scrollTo,TResult? Function( PreviewShowAttachment value)?  showAttachment,TResult? Function( PreviewOpenPdf value)?  openPdf,TResult? Function( PreviewCopyCode value)?  copyCode,TResult? Function( PreviewPlayMoment value)?  playMoment,}){
final _that = this;
switch (_that) {
case PreviewOpenNote() when openNote != null:
return openNote(_that);case PreviewOpenTag() when openTag != null:
return openTag(_that);case PreviewScrollTo() when scrollTo != null:
return scrollTo(_that);case PreviewShowAttachment() when showAttachment != null:
return showAttachment(_that);case PreviewOpenPdf() when openPdf != null:
return openPdf(_that);case PreviewCopyCode() when copyCode != null:
return copyCode(_that);case PreviewPlayMoment() when playMoment != null:
return playMoment(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  openNote,TResult Function( String name)?  openTag,TResult Function( String anchor)?  scrollTo,TResult Function( AttachmentFile file)?  showAttachment,TResult Function( String noteId,  String file,  String name,  int? page,  String? highlight)?  openPdf,TResult Function( int index,  String code)?  copyCode,TResult Function( String fileId,  int seconds)?  playMoment,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PreviewOpenNote() when openNote != null:
return openNote(_that.noteId);case PreviewOpenTag() when openTag != null:
return openTag(_that.name);case PreviewScrollTo() when scrollTo != null:
return scrollTo(_that.anchor);case PreviewShowAttachment() when showAttachment != null:
return showAttachment(_that.file);case PreviewOpenPdf() when openPdf != null:
return openPdf(_that.noteId,_that.file,_that.name,_that.page,_that.highlight);case PreviewCopyCode() when copyCode != null:
return copyCode(_that.index,_that.code);case PreviewPlayMoment() when playMoment != null:
return playMoment(_that.fileId,_that.seconds);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  openNote,required TResult Function( String name)  openTag,required TResult Function( String anchor)  scrollTo,required TResult Function( AttachmentFile file)  showAttachment,required TResult Function( String noteId,  String file,  String name,  int? page,  String? highlight)  openPdf,required TResult Function( int index,  String code)  copyCode,required TResult Function( String fileId,  int seconds)  playMoment,}) {final _that = this;
switch (_that) {
case PreviewOpenNote():
return openNote(_that.noteId);case PreviewOpenTag():
return openTag(_that.name);case PreviewScrollTo():
return scrollTo(_that.anchor);case PreviewShowAttachment():
return showAttachment(_that.file);case PreviewOpenPdf():
return openPdf(_that.noteId,_that.file,_that.name,_that.page,_that.highlight);case PreviewCopyCode():
return copyCode(_that.index,_that.code);case PreviewPlayMoment():
return playMoment(_that.fileId,_that.seconds);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  openNote,TResult? Function( String name)?  openTag,TResult? Function( String anchor)?  scrollTo,TResult? Function( AttachmentFile file)?  showAttachment,TResult? Function( String noteId,  String file,  String name,  int? page,  String? highlight)?  openPdf,TResult? Function( int index,  String code)?  copyCode,TResult? Function( String fileId,  int seconds)?  playMoment,}) {final _that = this;
switch (_that) {
case PreviewOpenNote() when openNote != null:
return openNote(_that.noteId);case PreviewOpenTag() when openTag != null:
return openTag(_that.name);case PreviewScrollTo() when scrollTo != null:
return scrollTo(_that.anchor);case PreviewShowAttachment() when showAttachment != null:
return showAttachment(_that.file);case PreviewOpenPdf() when openPdf != null:
return openPdf(_that.noteId,_that.file,_that.name,_that.page,_that.highlight);case PreviewCopyCode() when copyCode != null:
return copyCode(_that.index,_that.code);case PreviewPlayMoment() when playMoment != null:
return playMoment(_that.fileId,_that.seconds);case _:
  return null;

}
}

}

/// @nodoc


class PreviewOpenNote implements PreviewAction {
  const PreviewOpenNote(this.noteId);
  

 final  String noteId;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewOpenNoteCopyWith<PreviewOpenNote> get copyWith => _$PreviewOpenNoteCopyWithImpl<PreviewOpenNote>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewOpenNote&&(identical(other.noteId, noteId) || other.noteId == noteId));
}


@override
int get hashCode => Object.hash(runtimeType,noteId);

@override
String toString() {
  return 'PreviewAction.openNote(noteId: $noteId)';
}


}

/// @nodoc
abstract mixin class $PreviewOpenNoteCopyWith<$Res> implements $PreviewActionCopyWith<$Res> {
  factory $PreviewOpenNoteCopyWith(PreviewOpenNote value, $Res Function(PreviewOpenNote) _then) = _$PreviewOpenNoteCopyWithImpl;
@useResult
$Res call({
 String noteId
});




}
/// @nodoc
class _$PreviewOpenNoteCopyWithImpl<$Res>
    implements $PreviewOpenNoteCopyWith<$Res> {
  _$PreviewOpenNoteCopyWithImpl(this._self, this._then);

  final PreviewOpenNote _self;
  final $Res Function(PreviewOpenNote) _then;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,}) {
  return _then(PreviewOpenNote(
null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewOpenTag implements PreviewAction {
  const PreviewOpenTag(this.name);
  

 final  String name;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewOpenTagCopyWith<PreviewOpenTag> get copyWith => _$PreviewOpenTagCopyWithImpl<PreviewOpenTag>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewOpenTag&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,name);

@override
String toString() {
  return 'PreviewAction.openTag(name: $name)';
}


}

/// @nodoc
abstract mixin class $PreviewOpenTagCopyWith<$Res> implements $PreviewActionCopyWith<$Res> {
  factory $PreviewOpenTagCopyWith(PreviewOpenTag value, $Res Function(PreviewOpenTag) _then) = _$PreviewOpenTagCopyWithImpl;
@useResult
$Res call({
 String name
});




}
/// @nodoc
class _$PreviewOpenTagCopyWithImpl<$Res>
    implements $PreviewOpenTagCopyWith<$Res> {
  _$PreviewOpenTagCopyWithImpl(this._self, this._then);

  final PreviewOpenTag _self;
  final $Res Function(PreviewOpenTag) _then;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? name = null,}) {
  return _then(PreviewOpenTag(
null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewScrollTo implements PreviewAction {
  const PreviewScrollTo(this.anchor);
  

 final  String anchor;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewScrollToCopyWith<PreviewScrollTo> get copyWith => _$PreviewScrollToCopyWithImpl<PreviewScrollTo>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewScrollTo&&(identical(other.anchor, anchor) || other.anchor == anchor));
}


@override
int get hashCode => Object.hash(runtimeType,anchor);

@override
String toString() {
  return 'PreviewAction.scrollTo(anchor: $anchor)';
}


}

/// @nodoc
abstract mixin class $PreviewScrollToCopyWith<$Res> implements $PreviewActionCopyWith<$Res> {
  factory $PreviewScrollToCopyWith(PreviewScrollTo value, $Res Function(PreviewScrollTo) _then) = _$PreviewScrollToCopyWithImpl;
@useResult
$Res call({
 String anchor
});




}
/// @nodoc
class _$PreviewScrollToCopyWithImpl<$Res>
    implements $PreviewScrollToCopyWith<$Res> {
  _$PreviewScrollToCopyWithImpl(this._self, this._then);

  final PreviewScrollTo _self;
  final $Res Function(PreviewScrollTo) _then;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? anchor = null,}) {
  return _then(PreviewScrollTo(
null == anchor ? _self.anchor : anchor // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewShowAttachment implements PreviewAction {
  const PreviewShowAttachment(this.file);
  

 final  AttachmentFile file;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewShowAttachmentCopyWith<PreviewShowAttachment> get copyWith => _$PreviewShowAttachmentCopyWithImpl<PreviewShowAttachment>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewShowAttachment&&(identical(other.file, file) || other.file == file));
}


@override
int get hashCode => Object.hash(runtimeType,file);

@override
String toString() {
  return 'PreviewAction.showAttachment(file: $file)';
}


}

/// @nodoc
abstract mixin class $PreviewShowAttachmentCopyWith<$Res> implements $PreviewActionCopyWith<$Res> {
  factory $PreviewShowAttachmentCopyWith(PreviewShowAttachment value, $Res Function(PreviewShowAttachment) _then) = _$PreviewShowAttachmentCopyWithImpl;
@useResult
$Res call({
 AttachmentFile file
});




}
/// @nodoc
class _$PreviewShowAttachmentCopyWithImpl<$Res>
    implements $PreviewShowAttachmentCopyWith<$Res> {
  _$PreviewShowAttachmentCopyWithImpl(this._self, this._then);

  final PreviewShowAttachment _self;
  final $Res Function(PreviewShowAttachment) _then;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? file = null,}) {
  return _then(PreviewShowAttachment(
null == file ? _self.file : file // ignore: cast_nullable_to_non_nullable
as AttachmentFile,
  ));
}


}

/// @nodoc


class PreviewOpenPdf implements PreviewAction {
  const PreviewOpenPdf({required this.noteId, required this.file, required this.name, this.page, this.highlight});


 final  String noteId;
 final  String file;
 final  String name;
 final  int? page;
 final  String? highlight;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewOpenPdfCopyWith<PreviewOpenPdf> get copyWith => _$PreviewOpenPdfCopyWithImpl<PreviewOpenPdf>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewOpenPdf&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.file, file) || other.file == file)&&(identical(other.name, name) || other.name == name)&&(identical(other.page, page) || other.page == page)&&(identical(other.highlight, highlight) || other.highlight == highlight));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,file,name,page,highlight);

@override
String toString() {
  return 'PreviewAction.openPdf(noteId: $noteId, file: $file, name: $name, page: $page, highlight: $highlight)';
}


}

/// @nodoc
abstract mixin class $PreviewOpenPdfCopyWith<$Res> implements $PreviewActionCopyWith<$Res> {
  factory $PreviewOpenPdfCopyWith(PreviewOpenPdf value, $Res Function(PreviewOpenPdf) _then) = _$PreviewOpenPdfCopyWithImpl;
@useResult
$Res call({
 String noteId, String file, String name, int? page, String? highlight
});




}
/// @nodoc
class _$PreviewOpenPdfCopyWithImpl<$Res>
    implements $PreviewOpenPdfCopyWith<$Res> {
  _$PreviewOpenPdfCopyWithImpl(this._self, this._then);

  final PreviewOpenPdf _self;
  final $Res Function(PreviewOpenPdf) _then;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? file = null,Object? name = null,Object? page = freezed,Object? highlight = freezed,}) {
  return _then(PreviewOpenPdf(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,file: null == file ? _self.file : file // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,page: freezed == page ? _self.page : page // ignore: cast_nullable_to_non_nullable
as int?,highlight: freezed == highlight ? _self.highlight : highlight // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class PreviewCopyCode implements PreviewAction {
  const PreviewCopyCode(this.index, this.code);
  

 final  int index;
 final  String code;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewCopyCodeCopyWith<PreviewCopyCode> get copyWith => _$PreviewCopyCodeCopyWithImpl<PreviewCopyCode>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewCopyCode&&(identical(other.index, index) || other.index == index)&&(identical(other.code, code) || other.code == code));
}


@override
int get hashCode => Object.hash(runtimeType,index,code);

@override
String toString() {
  return 'PreviewAction.copyCode(index: $index, code: $code)';
}


}

/// @nodoc
abstract mixin class $PreviewCopyCodeCopyWith<$Res> implements $PreviewActionCopyWith<$Res> {
  factory $PreviewCopyCodeCopyWith(PreviewCopyCode value, $Res Function(PreviewCopyCode) _then) = _$PreviewCopyCodeCopyWithImpl;
@useResult
$Res call({
 int index, String code
});




}
/// @nodoc
class _$PreviewCopyCodeCopyWithImpl<$Res>
    implements $PreviewCopyCodeCopyWith<$Res> {
  _$PreviewCopyCodeCopyWithImpl(this._self, this._then);

  final PreviewCopyCode _self;
  final $Res Function(PreviewCopyCode) _then;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? index = null,Object? code = null,}) {
  return _then(PreviewCopyCode(
null == index ? _self.index : index // ignore: cast_nullable_to_non_nullable
as int,null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PreviewPlayMoment implements PreviewAction {
  const PreviewPlayMoment(this.fileId, this.seconds);
  

 final  String fileId;
 final  int seconds;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PreviewPlayMomentCopyWith<PreviewPlayMoment> get copyWith => _$PreviewPlayMomentCopyWithImpl<PreviewPlayMoment>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PreviewPlayMoment&&(identical(other.fileId, fileId) || other.fileId == fileId)&&(identical(other.seconds, seconds) || other.seconds == seconds));
}


@override
int get hashCode => Object.hash(runtimeType,fileId,seconds);

@override
String toString() {
  return 'PreviewAction.playMoment(fileId: $fileId, seconds: $seconds)';
}


}

/// @nodoc
abstract mixin class $PreviewPlayMomentCopyWith<$Res> implements $PreviewActionCopyWith<$Res> {
  factory $PreviewPlayMomentCopyWith(PreviewPlayMoment value, $Res Function(PreviewPlayMoment) _then) = _$PreviewPlayMomentCopyWithImpl;
@useResult
$Res call({
 String fileId, int seconds
});




}
/// @nodoc
class _$PreviewPlayMomentCopyWithImpl<$Res>
    implements $PreviewPlayMomentCopyWith<$Res> {
  _$PreviewPlayMomentCopyWithImpl(this._self, this._then);

  final PreviewPlayMoment _self;
  final $Res Function(PreviewPlayMoment) _then;

/// Create a copy of PreviewAction
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? fileId = null,Object? seconds = null,}) {
  return _then(PreviewPlayMoment(
null == fileId ? _self.fileId : fileId // ignore: cast_nullable_to_non_nullable
as String,null == seconds ? _self.seconds : seconds // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on

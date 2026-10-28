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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PreviewOpenNote value)?  openNote,TResult Function( PreviewOpenTag value)?  openTag,TResult Function( PreviewScrollTo value)?  scrollTo,TResult Function( PreviewShowAttachment value)?  showAttachment,TResult Function( PreviewCopyCode value)?  copyCode,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PreviewOpenNote() when openNote != null:
return openNote(_that);case PreviewOpenTag() when openTag != null:
return openTag(_that);case PreviewScrollTo() when scrollTo != null:
return scrollTo(_that);case PreviewShowAttachment() when showAttachment != null:
return showAttachment(_that);case PreviewCopyCode() when copyCode != null:
return copyCode(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PreviewOpenNote value)  openNote,required TResult Function( PreviewOpenTag value)  openTag,required TResult Function( PreviewScrollTo value)  scrollTo,required TResult Function( PreviewShowAttachment value)  showAttachment,required TResult Function( PreviewCopyCode value)  copyCode,}){
final _that = this;
switch (_that) {
case PreviewOpenNote():
return openNote(_that);case PreviewOpenTag():
return openTag(_that);case PreviewScrollTo():
return scrollTo(_that);case PreviewShowAttachment():
return showAttachment(_that);case PreviewCopyCode():
return copyCode(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PreviewOpenNote value)?  openNote,TResult? Function( PreviewOpenTag value)?  openTag,TResult? Function( PreviewScrollTo value)?  scrollTo,TResult? Function( PreviewShowAttachment value)?  showAttachment,TResult? Function( PreviewCopyCode value)?  copyCode,}){
final _that = this;
switch (_that) {
case PreviewOpenNote() when openNote != null:
return openNote(_that);case PreviewOpenTag() when openTag != null:
return openTag(_that);case PreviewScrollTo() when scrollTo != null:
return scrollTo(_that);case PreviewShowAttachment() when showAttachment != null:
return showAttachment(_that);case PreviewCopyCode() when copyCode != null:
return copyCode(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String noteId)?  openNote,TResult Function( String name)?  openTag,TResult Function( String anchor)?  scrollTo,TResult Function( AttachmentFile file)?  showAttachment,TResult Function( int index,  String code)?  copyCode,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PreviewOpenNote() when openNote != null:
return openNote(_that.noteId);case PreviewOpenTag() when openTag != null:
return openTag(_that.name);case PreviewScrollTo() when scrollTo != null:
return scrollTo(_that.anchor);case PreviewShowAttachment() when showAttachment != null:
return showAttachment(_that.file);case PreviewCopyCode() when copyCode != null:
return copyCode(_that.index,_that.code);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String noteId)  openNote,required TResult Function( String name)  openTag,required TResult Function( String anchor)  scrollTo,required TResult Function( AttachmentFile file)  showAttachment,required TResult Function( int index,  String code)  copyCode,}) {final _that = this;
switch (_that) {
case PreviewOpenNote():
return openNote(_that.noteId);case PreviewOpenTag():
return openTag(_that.name);case PreviewScrollTo():
return scrollTo(_that.anchor);case PreviewShowAttachment():
return showAttachment(_that.file);case PreviewCopyCode():
return copyCode(_that.index,_that.code);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String noteId)?  openNote,TResult? Function( String name)?  openTag,TResult? Function( String anchor)?  scrollTo,TResult? Function( AttachmentFile file)?  showAttachment,TResult? Function( int index,  String code)?  copyCode,}) {final _that = this;
switch (_that) {
case PreviewOpenNote() when openNote != null:
return openNote(_that.noteId);case PreviewOpenTag() when openTag != null:
return openTag(_that.name);case PreviewScrollTo() when scrollTo != null:
return scrollTo(_that.anchor);case PreviewShowAttachment() when showAttachment != null:
return showAttachment(_that.file);case PreviewCopyCode() when copyCode != null:
return copyCode(_that.index,_that.code);case _:
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

// dart format on

// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'answer_block.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AnswerBlock {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerBlock);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AnswerBlock()';
}


}

/// @nodoc
class $AnswerBlockCopyWith<$Res>  {
$AnswerBlockCopyWith(AnswerBlock _, $Res Function(AnswerBlock) __);
}


/// Adds pattern-matching-related methods to [AnswerBlock].
extension AnswerBlockPatterns on AnswerBlock {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AnswerHeading value)?  heading,TResult Function( AnswerParagraph value)?  paragraph,TResult Function( AnswerList value)?  list,TResult Function( AnswerCode value)?  code,TResult Function( AnswerQuote value)?  quote,TResult Function( AnswerRule value)?  rule,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AnswerHeading() when heading != null:
return heading(_that);case AnswerParagraph() when paragraph != null:
return paragraph(_that);case AnswerList() when list != null:
return list(_that);case AnswerCode() when code != null:
return code(_that);case AnswerQuote() when quote != null:
return quote(_that);case AnswerRule() when rule != null:
return rule(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AnswerHeading value)  heading,required TResult Function( AnswerParagraph value)  paragraph,required TResult Function( AnswerList value)  list,required TResult Function( AnswerCode value)  code,required TResult Function( AnswerQuote value)  quote,required TResult Function( AnswerRule value)  rule,}){
final _that = this;
switch (_that) {
case AnswerHeading():
return heading(_that);case AnswerParagraph():
return paragraph(_that);case AnswerList():
return list(_that);case AnswerCode():
return code(_that);case AnswerQuote():
return quote(_that);case AnswerRule():
return rule(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AnswerHeading value)?  heading,TResult? Function( AnswerParagraph value)?  paragraph,TResult? Function( AnswerList value)?  list,TResult? Function( AnswerCode value)?  code,TResult? Function( AnswerQuote value)?  quote,TResult? Function( AnswerRule value)?  rule,}){
final _that = this;
switch (_that) {
case AnswerHeading() when heading != null:
return heading(_that);case AnswerParagraph() when paragraph != null:
return paragraph(_that);case AnswerList() when list != null:
return list(_that);case AnswerCode() when code != null:
return code(_that);case AnswerQuote() when quote != null:
return quote(_that);case AnswerRule() when rule != null:
return rule(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( int level,  String text)?  heading,TResult Function( String text)?  paragraph,TResult Function( List<AnswerListItem> items)?  list,TResult Function( String text)?  code,TResult Function( String text)?  quote,TResult Function()?  rule,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AnswerHeading() when heading != null:
return heading(_that.level,_that.text);case AnswerParagraph() when paragraph != null:
return paragraph(_that.text);case AnswerList() when list != null:
return list(_that.items);case AnswerCode() when code != null:
return code(_that.text);case AnswerQuote() when quote != null:
return quote(_that.text);case AnswerRule() when rule != null:
return rule();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( int level,  String text)  heading,required TResult Function( String text)  paragraph,required TResult Function( List<AnswerListItem> items)  list,required TResult Function( String text)  code,required TResult Function( String text)  quote,required TResult Function()  rule,}) {final _that = this;
switch (_that) {
case AnswerHeading():
return heading(_that.level,_that.text);case AnswerParagraph():
return paragraph(_that.text);case AnswerList():
return list(_that.items);case AnswerCode():
return code(_that.text);case AnswerQuote():
return quote(_that.text);case AnswerRule():
return rule();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( int level,  String text)?  heading,TResult? Function( String text)?  paragraph,TResult? Function( List<AnswerListItem> items)?  list,TResult? Function( String text)?  code,TResult? Function( String text)?  quote,TResult? Function()?  rule,}) {final _that = this;
switch (_that) {
case AnswerHeading() when heading != null:
return heading(_that.level,_that.text);case AnswerParagraph() when paragraph != null:
return paragraph(_that.text);case AnswerList() when list != null:
return list(_that.items);case AnswerCode() when code != null:
return code(_that.text);case AnswerQuote() when quote != null:
return quote(_that.text);case AnswerRule() when rule != null:
return rule();case _:
  return null;

}
}

}

/// @nodoc


class AnswerHeading implements AnswerBlock {
  const AnswerHeading({required this.level, required this.text});
  

 final  int level;
 final  String text;

/// Create a copy of AnswerBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AnswerHeadingCopyWith<AnswerHeading> get copyWith => _$AnswerHeadingCopyWithImpl<AnswerHeading>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerHeading&&(identical(other.level, level) || other.level == level)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,level,text);

@override
String toString() {
  return 'AnswerBlock.heading(level: $level, text: $text)';
}


}

/// @nodoc
abstract mixin class $AnswerHeadingCopyWith<$Res> implements $AnswerBlockCopyWith<$Res> {
  factory $AnswerHeadingCopyWith(AnswerHeading value, $Res Function(AnswerHeading) _then) = _$AnswerHeadingCopyWithImpl;
@useResult
$Res call({
 int level, String text
});




}
/// @nodoc
class _$AnswerHeadingCopyWithImpl<$Res>
    implements $AnswerHeadingCopyWith<$Res> {
  _$AnswerHeadingCopyWithImpl(this._self, this._then);

  final AnswerHeading _self;
  final $Res Function(AnswerHeading) _then;

/// Create a copy of AnswerBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? level = null,Object? text = null,}) {
  return _then(AnswerHeading(
level: null == level ? _self.level : level // ignore: cast_nullable_to_non_nullable
as int,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AnswerParagraph implements AnswerBlock {
  const AnswerParagraph(this.text);
  

 final  String text;

/// Create a copy of AnswerBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AnswerParagraphCopyWith<AnswerParagraph> get copyWith => _$AnswerParagraphCopyWithImpl<AnswerParagraph>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerParagraph&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AnswerBlock.paragraph(text: $text)';
}


}

/// @nodoc
abstract mixin class $AnswerParagraphCopyWith<$Res> implements $AnswerBlockCopyWith<$Res> {
  factory $AnswerParagraphCopyWith(AnswerParagraph value, $Res Function(AnswerParagraph) _then) = _$AnswerParagraphCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AnswerParagraphCopyWithImpl<$Res>
    implements $AnswerParagraphCopyWith<$Res> {
  _$AnswerParagraphCopyWithImpl(this._self, this._then);

  final AnswerParagraph _self;
  final $Res Function(AnswerParagraph) _then;

/// Create a copy of AnswerBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AnswerParagraph(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AnswerList implements AnswerBlock {
  const AnswerList(final  List<AnswerListItem> items): _items = items;
  

 final  List<AnswerListItem> _items;
 List<AnswerListItem> get items {
  if (_items is EqualUnmodifiableListView) return _items;
  // ignore: implicit_dynamic_type
  return EqualUnmodifiableListView(_items);
}


/// Create a copy of AnswerBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AnswerListCopyWith<AnswerList> get copyWith => _$AnswerListCopyWithImpl<AnswerList>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerList&&const DeepCollectionEquality().equals(other._items, _items));
}


@override
int get hashCode => Object.hash(runtimeType,const DeepCollectionEquality().hash(_items));

@override
String toString() {
  return 'AnswerBlock.list(items: $items)';
}


}

/// @nodoc
abstract mixin class $AnswerListCopyWith<$Res> implements $AnswerBlockCopyWith<$Res> {
  factory $AnswerListCopyWith(AnswerList value, $Res Function(AnswerList) _then) = _$AnswerListCopyWithImpl;
@useResult
$Res call({
 List<AnswerListItem> items
});




}
/// @nodoc
class _$AnswerListCopyWithImpl<$Res>
    implements $AnswerListCopyWith<$Res> {
  _$AnswerListCopyWithImpl(this._self, this._then);

  final AnswerList _self;
  final $Res Function(AnswerList) _then;

/// Create a copy of AnswerBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? items = null,}) {
  return _then(AnswerList(
null == items ? _self._items : items // ignore: cast_nullable_to_non_nullable
as List<AnswerListItem>,
  ));
}


}

/// @nodoc


class AnswerCode implements AnswerBlock {
  const AnswerCode(this.text);
  

 final  String text;

/// Create a copy of AnswerBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AnswerCodeCopyWith<AnswerCode> get copyWith => _$AnswerCodeCopyWithImpl<AnswerCode>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerCode&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AnswerBlock.code(text: $text)';
}


}

/// @nodoc
abstract mixin class $AnswerCodeCopyWith<$Res> implements $AnswerBlockCopyWith<$Res> {
  factory $AnswerCodeCopyWith(AnswerCode value, $Res Function(AnswerCode) _then) = _$AnswerCodeCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AnswerCodeCopyWithImpl<$Res>
    implements $AnswerCodeCopyWith<$Res> {
  _$AnswerCodeCopyWithImpl(this._self, this._then);

  final AnswerCode _self;
  final $Res Function(AnswerCode) _then;

/// Create a copy of AnswerBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AnswerCode(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AnswerQuote implements AnswerBlock {
  const AnswerQuote(this.text);
  

 final  String text;

/// Create a copy of AnswerBlock
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AnswerQuoteCopyWith<AnswerQuote> get copyWith => _$AnswerQuoteCopyWithImpl<AnswerQuote>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerQuote&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,text);

@override
String toString() {
  return 'AnswerBlock.quote(text: $text)';
}


}

/// @nodoc
abstract mixin class $AnswerQuoteCopyWith<$Res> implements $AnswerBlockCopyWith<$Res> {
  factory $AnswerQuoteCopyWith(AnswerQuote value, $Res Function(AnswerQuote) _then) = _$AnswerQuoteCopyWithImpl;
@useResult
$Res call({
 String text
});




}
/// @nodoc
class _$AnswerQuoteCopyWithImpl<$Res>
    implements $AnswerQuoteCopyWith<$Res> {
  _$AnswerQuoteCopyWithImpl(this._self, this._then);

  final AnswerQuote _self;
  final $Res Function(AnswerQuote) _then;

/// Create a copy of AnswerBlock
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? text = null,}) {
  return _then(AnswerQuote(
null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AnswerRule implements AnswerBlock {
  const AnswerRule();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerRule);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AnswerBlock.rule()';
}


}




/// @nodoc
mixin _$AnswerListItem {

/// What leads the item: a bullet, or its number and a full stop.
 String get marker;/// The item's inline Markdown.
 String get text;/// How deep the item is nested, from 0.
 int get depth;
/// Create a copy of AnswerListItem
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AnswerListItemCopyWith<AnswerListItem> get copyWith => _$AnswerListItemCopyWithImpl<AnswerListItem>(this as AnswerListItem, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AnswerListItem&&(identical(other.marker, marker) || other.marker == marker)&&(identical(other.text, text) || other.text == text)&&(identical(other.depth, depth) || other.depth == depth));
}


@override
int get hashCode => Object.hash(runtimeType,marker,text,depth);

@override
String toString() {
  return 'AnswerListItem(marker: $marker, text: $text, depth: $depth)';
}


}

/// @nodoc
abstract mixin class $AnswerListItemCopyWith<$Res>  {
  factory $AnswerListItemCopyWith(AnswerListItem value, $Res Function(AnswerListItem) _then) = _$AnswerListItemCopyWithImpl;
@useResult
$Res call({
 String marker, String text, int depth
});




}
/// @nodoc
class _$AnswerListItemCopyWithImpl<$Res>
    implements $AnswerListItemCopyWith<$Res> {
  _$AnswerListItemCopyWithImpl(this._self, this._then);

  final AnswerListItem _self;
  final $Res Function(AnswerListItem) _then;

/// Create a copy of AnswerListItem
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? marker = null,Object? text = null,Object? depth = null,}) {
  return _then(_self.copyWith(
marker: null == marker ? _self.marker : marker // ignore: cast_nullable_to_non_nullable
as String,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,depth: null == depth ? _self.depth : depth // ignore: cast_nullable_to_non_nullable
as int,
  ));
}

}


/// Adds pattern-matching-related methods to [AnswerListItem].
extension AnswerListItemPatterns on AnswerListItem {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AnswerListItem value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AnswerListItem() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AnswerListItem value)  $default,){
final _that = this;
switch (_that) {
case _AnswerListItem():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AnswerListItem value)?  $default,){
final _that = this;
switch (_that) {
case _AnswerListItem() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String marker,  String text,  int depth)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AnswerListItem() when $default != null:
return $default(_that.marker,_that.text,_that.depth);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String marker,  String text,  int depth)  $default,) {final _that = this;
switch (_that) {
case _AnswerListItem():
return $default(_that.marker,_that.text,_that.depth);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String marker,  String text,  int depth)?  $default,) {final _that = this;
switch (_that) {
case _AnswerListItem() when $default != null:
return $default(_that.marker,_that.text,_that.depth);case _:
  return null;

}
}

}

/// @nodoc


class _AnswerListItem implements AnswerListItem {
  const _AnswerListItem({required this.marker, required this.text, this.depth = 0});
  

/// What leads the item: a bullet, or its number and a full stop.
@override final  String marker;
/// The item's inline Markdown.
@override final  String text;
/// How deep the item is nested, from 0.
@override@JsonKey() final  int depth;

/// Create a copy of AnswerListItem
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AnswerListItemCopyWith<_AnswerListItem> get copyWith => __$AnswerListItemCopyWithImpl<_AnswerListItem>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AnswerListItem&&(identical(other.marker, marker) || other.marker == marker)&&(identical(other.text, text) || other.text == text)&&(identical(other.depth, depth) || other.depth == depth));
}


@override
int get hashCode => Object.hash(runtimeType,marker,text,depth);

@override
String toString() {
  return 'AnswerListItem(marker: $marker, text: $text, depth: $depth)';
}


}

/// @nodoc
abstract mixin class _$AnswerListItemCopyWith<$Res> implements $AnswerListItemCopyWith<$Res> {
  factory _$AnswerListItemCopyWith(_AnswerListItem value, $Res Function(_AnswerListItem) _then) = __$AnswerListItemCopyWithImpl;
@override @useResult
$Res call({
 String marker, String text, int depth
});




}
/// @nodoc
class __$AnswerListItemCopyWithImpl<$Res>
    implements _$AnswerListItemCopyWith<$Res> {
  __$AnswerListItemCopyWithImpl(this._self, this._then);

  final _AnswerListItem _self;
  final $Res Function(_AnswerListItem) _then;

/// Create a copy of AnswerListItem
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? marker = null,Object? text = null,Object? depth = null,}) {
  return _then(_AnswerListItem(
marker: null == marker ? _self.marker : marker // ignore: cast_nullable_to_non_nullable
as String,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,depth: null == depth ? _self.depth : depth // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on

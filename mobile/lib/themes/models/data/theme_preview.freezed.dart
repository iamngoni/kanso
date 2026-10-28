// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'theme_preview.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$ThemePreview {

/// The theme file's name without `.json`: `nord`.
 String get id; String get name;/// Whether the theme is for a dark interface.
 bool get dark; Color get editor; Color get text; Color get accent; Color get heading; Color get marker; Color get quote; Color get link; Color get url; Color get code; Color get strong; Color get emphasis;/// The theme as the app would draw it, for the pictures of the editor
/// syntax and the preview.
 KansoPalette get palette;/// Where the theme came from.
 InstalledThemeOrigin get origin;/// Who made the theme; empty when it does not say.
 String get author;
/// Create a copy of ThemePreview
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ThemePreviewCopyWith<ThemePreview> get copyWith => _$ThemePreviewCopyWithImpl<ThemePreview>(this as ThemePreview, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ThemePreview&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.dark, dark) || other.dark == dark)&&(identical(other.editor, editor) || other.editor == editor)&&(identical(other.text, text) || other.text == text)&&(identical(other.accent, accent) || other.accent == accent)&&(identical(other.heading, heading) || other.heading == heading)&&(identical(other.marker, marker) || other.marker == marker)&&(identical(other.quote, quote) || other.quote == quote)&&(identical(other.link, link) || other.link == link)&&(identical(other.url, url) || other.url == url)&&(identical(other.code, code) || other.code == code)&&(identical(other.strong, strong) || other.strong == strong)&&(identical(other.emphasis, emphasis) || other.emphasis == emphasis)&&(identical(other.palette, palette) || other.palette == palette)&&(identical(other.origin, origin) || other.origin == origin)&&(identical(other.author, author) || other.author == author));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,dark,editor,text,accent,heading,marker,quote,link,url,code,strong,emphasis,palette,origin,author);

@override
String toString() {
  return 'ThemePreview(id: $id, name: $name, dark: $dark, editor: $editor, text: $text, accent: $accent, heading: $heading, marker: $marker, quote: $quote, link: $link, url: $url, code: $code, strong: $strong, emphasis: $emphasis, palette: $palette, origin: $origin, author: $author)';
}


}

/// @nodoc
abstract mixin class $ThemePreviewCopyWith<$Res>  {
  factory $ThemePreviewCopyWith(ThemePreview value, $Res Function(ThemePreview) _then) = _$ThemePreviewCopyWithImpl;
@useResult
$Res call({
 String id, String name, bool dark, Color editor, Color text, Color accent, Color heading, Color marker, Color quote, Color link, Color url, Color code, Color strong, Color emphasis, KansoPalette palette, InstalledThemeOrigin origin, String author
});


$KansoPaletteCopyWith<$Res> get palette;

}
/// @nodoc
class _$ThemePreviewCopyWithImpl<$Res>
    implements $ThemePreviewCopyWith<$Res> {
  _$ThemePreviewCopyWithImpl(this._self, this._then);

  final ThemePreview _self;
  final $Res Function(ThemePreview) _then;

/// Create a copy of ThemePreview
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? id = null,Object? name = null,Object? dark = null,Object? editor = null,Object? text = null,Object? accent = null,Object? heading = null,Object? marker = null,Object? quote = null,Object? link = null,Object? url = null,Object? code = null,Object? strong = null,Object? emphasis = null,Object? palette = null,Object? origin = null,Object? author = null,}) {
  return _then(_self.copyWith(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as bool,editor: null == editor ? _self.editor : editor // ignore: cast_nullable_to_non_nullable
as Color,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as Color,accent: null == accent ? _self.accent : accent // ignore: cast_nullable_to_non_nullable
as Color,heading: null == heading ? _self.heading : heading // ignore: cast_nullable_to_non_nullable
as Color,marker: null == marker ? _self.marker : marker // ignore: cast_nullable_to_non_nullable
as Color,quote: null == quote ? _self.quote : quote // ignore: cast_nullable_to_non_nullable
as Color,link: null == link ? _self.link : link // ignore: cast_nullable_to_non_nullable
as Color,url: null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as Color,code: null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as Color,strong: null == strong ? _self.strong : strong // ignore: cast_nullable_to_non_nullable
as Color,emphasis: null == emphasis ? _self.emphasis : emphasis // ignore: cast_nullable_to_non_nullable
as Color,palette: null == palette ? _self.palette : palette // ignore: cast_nullable_to_non_nullable
as KansoPalette,origin: null == origin ? _self.origin : origin // ignore: cast_nullable_to_non_nullable
as InstalledThemeOrigin,author: null == author ? _self.author : author // ignore: cast_nullable_to_non_nullable
as String,
  ));
}
/// Create a copy of ThemePreview
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoPaletteCopyWith<$Res> get palette {
  
  return $KansoPaletteCopyWith<$Res>(_self.palette, (value) {
    return _then(_self.copyWith(palette: value));
  });
}
}


/// Adds pattern-matching-related methods to [ThemePreview].
extension ThemePreviewPatterns on ThemePreview {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _ThemePreview value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _ThemePreview() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _ThemePreview value)  $default,){
final _that = this;
switch (_that) {
case _ThemePreview():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _ThemePreview value)?  $default,){
final _that = this;
switch (_that) {
case _ThemePreview() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( String id,  String name,  bool dark,  Color editor,  Color text,  Color accent,  Color heading,  Color marker,  Color quote,  Color link,  Color url,  Color code,  Color strong,  Color emphasis,  KansoPalette palette,  InstalledThemeOrigin origin,  String author)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _ThemePreview() when $default != null:
return $default(_that.id,_that.name,_that.dark,_that.editor,_that.text,_that.accent,_that.heading,_that.marker,_that.quote,_that.link,_that.url,_that.code,_that.strong,_that.emphasis,_that.palette,_that.origin,_that.author);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( String id,  String name,  bool dark,  Color editor,  Color text,  Color accent,  Color heading,  Color marker,  Color quote,  Color link,  Color url,  Color code,  Color strong,  Color emphasis,  KansoPalette palette,  InstalledThemeOrigin origin,  String author)  $default,) {final _that = this;
switch (_that) {
case _ThemePreview():
return $default(_that.id,_that.name,_that.dark,_that.editor,_that.text,_that.accent,_that.heading,_that.marker,_that.quote,_that.link,_that.url,_that.code,_that.strong,_that.emphasis,_that.palette,_that.origin,_that.author);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( String id,  String name,  bool dark,  Color editor,  Color text,  Color accent,  Color heading,  Color marker,  Color quote,  Color link,  Color url,  Color code,  Color strong,  Color emphasis,  KansoPalette palette,  InstalledThemeOrigin origin,  String author)?  $default,) {final _that = this;
switch (_that) {
case _ThemePreview() when $default != null:
return $default(_that.id,_that.name,_that.dark,_that.editor,_that.text,_that.accent,_that.heading,_that.marker,_that.quote,_that.link,_that.url,_that.code,_that.strong,_that.emphasis,_that.palette,_that.origin,_that.author);case _:
  return null;

}
}

}

/// @nodoc


class _ThemePreview extends ThemePreview {
  const _ThemePreview({required this.id, required this.name, required this.dark, required this.editor, required this.text, required this.accent, required this.heading, required this.marker, required this.quote, required this.link, required this.url, required this.code, required this.strong, required this.emphasis, required this.palette, this.origin = InstalledThemeOrigin.builtin, this.author = ''}): super._();
  

/// The theme file's name without `.json`: `nord`.
@override final  String id;
@override final  String name;
/// Whether the theme is for a dark interface.
@override final  bool dark;
@override final  Color editor;
@override final  Color text;
@override final  Color accent;
@override final  Color heading;
@override final  Color marker;
@override final  Color quote;
@override final  Color link;
@override final  Color url;
@override final  Color code;
@override final  Color strong;
@override final  Color emphasis;
/// The theme as the app would draw it, for the pictures of the editor
/// syntax and the preview.
@override final  KansoPalette palette;
/// Where the theme came from.
@override@JsonKey() final  InstalledThemeOrigin origin;
/// Who made the theme; empty when it does not say.
@override@JsonKey() final  String author;

/// Create a copy of ThemePreview
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$ThemePreviewCopyWith<_ThemePreview> get copyWith => __$ThemePreviewCopyWithImpl<_ThemePreview>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _ThemePreview&&(identical(other.id, id) || other.id == id)&&(identical(other.name, name) || other.name == name)&&(identical(other.dark, dark) || other.dark == dark)&&(identical(other.editor, editor) || other.editor == editor)&&(identical(other.text, text) || other.text == text)&&(identical(other.accent, accent) || other.accent == accent)&&(identical(other.heading, heading) || other.heading == heading)&&(identical(other.marker, marker) || other.marker == marker)&&(identical(other.quote, quote) || other.quote == quote)&&(identical(other.link, link) || other.link == link)&&(identical(other.url, url) || other.url == url)&&(identical(other.code, code) || other.code == code)&&(identical(other.strong, strong) || other.strong == strong)&&(identical(other.emphasis, emphasis) || other.emphasis == emphasis)&&(identical(other.palette, palette) || other.palette == palette)&&(identical(other.origin, origin) || other.origin == origin)&&(identical(other.author, author) || other.author == author));
}


@override
int get hashCode => Object.hash(runtimeType,id,name,dark,editor,text,accent,heading,marker,quote,link,url,code,strong,emphasis,palette,origin,author);

@override
String toString() {
  return 'ThemePreview(id: $id, name: $name, dark: $dark, editor: $editor, text: $text, accent: $accent, heading: $heading, marker: $marker, quote: $quote, link: $link, url: $url, code: $code, strong: $strong, emphasis: $emphasis, palette: $palette, origin: $origin, author: $author)';
}


}

/// @nodoc
abstract mixin class _$ThemePreviewCopyWith<$Res> implements $ThemePreviewCopyWith<$Res> {
  factory _$ThemePreviewCopyWith(_ThemePreview value, $Res Function(_ThemePreview) _then) = __$ThemePreviewCopyWithImpl;
@override @useResult
$Res call({
 String id, String name, bool dark, Color editor, Color text, Color accent, Color heading, Color marker, Color quote, Color link, Color url, Color code, Color strong, Color emphasis, KansoPalette palette, InstalledThemeOrigin origin, String author
});


@override $KansoPaletteCopyWith<$Res> get palette;

}
/// @nodoc
class __$ThemePreviewCopyWithImpl<$Res>
    implements _$ThemePreviewCopyWith<$Res> {
  __$ThemePreviewCopyWithImpl(this._self, this._then);

  final _ThemePreview _self;
  final $Res Function(_ThemePreview) _then;

/// Create a copy of ThemePreview
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? id = null,Object? name = null,Object? dark = null,Object? editor = null,Object? text = null,Object? accent = null,Object? heading = null,Object? marker = null,Object? quote = null,Object? link = null,Object? url = null,Object? code = null,Object? strong = null,Object? emphasis = null,Object? palette = null,Object? origin = null,Object? author = null,}) {
  return _then(_ThemePreview(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as bool,editor: null == editor ? _self.editor : editor // ignore: cast_nullable_to_non_nullable
as Color,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as Color,accent: null == accent ? _self.accent : accent // ignore: cast_nullable_to_non_nullable
as Color,heading: null == heading ? _self.heading : heading // ignore: cast_nullable_to_non_nullable
as Color,marker: null == marker ? _self.marker : marker // ignore: cast_nullable_to_non_nullable
as Color,quote: null == quote ? _self.quote : quote // ignore: cast_nullable_to_non_nullable
as Color,link: null == link ? _self.link : link // ignore: cast_nullable_to_non_nullable
as Color,url: null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as Color,code: null == code ? _self.code : code // ignore: cast_nullable_to_non_nullable
as Color,strong: null == strong ? _self.strong : strong // ignore: cast_nullable_to_non_nullable
as Color,emphasis: null == emphasis ? _self.emphasis : emphasis // ignore: cast_nullable_to_non_nullable
as Color,palette: null == palette ? _self.palette : palette // ignore: cast_nullable_to_non_nullable
as KansoPalette,origin: null == origin ? _self.origin : origin // ignore: cast_nullable_to_non_nullable
as InstalledThemeOrigin,author: null == author ? _self.author : author // ignore: cast_nullable_to_non_nullable
as String,
  ));
}

/// Create a copy of ThemePreview
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoPaletteCopyWith<$Res> get palette {
  
  return $KansoPaletteCopyWith<$Res>(_self.palette, (value) {
    return _then(_self.copyWith(palette: value));
  });
}
}

// dart format on

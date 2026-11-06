// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'appearance_bloc.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$AppearanceEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AppearanceEvent()';
}


}

/// @nodoc
class $AppearanceEventCopyWith<$Res>  {
$AppearanceEventCopyWith(AppearanceEvent _, $Res Function(AppearanceEvent) __);
}


/// Adds pattern-matching-related methods to [AppearanceEvent].
extension AppearanceEventPatterns on AppearanceEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( AppearanceStartedEvent value)?  started,TResult Function( AppearanceThemePickedEvent value)?  themePicked,TResult Function( AppearanceSyntaxPickedEvent value)?  syntaxPicked,TResult Function( AppearancePreviewPickedEvent value)?  previewPicked,TResult Function( AppearanceFollowSystemChangedEvent value)?  followSystemChanged,TResult Function( AppearanceIconThemePickedEvent value)?  iconThemePicked,TResult Function( AppearanceInterfaceFontChangedEvent value)?  interfaceFontChanged,TResult Function( AppearanceEditorFontChangedEvent value)?  editorFontChanged,TResult Function( AppearancePreviewFontChangedEvent value)?  previewFontChanged,TResult Function( AppearanceCodeFontChangedEvent value)?  codeFontChanged,TResult Function( AppearanceTextSizeChangedEvent value)?  textSizeChanged,TResult Function( AppearancePreviewTextSizeChangedEvent value)?  previewTextSizeChanged,TResult Function( AppearanceInterfaceTextSizeChangedEvent value)?  interfaceTextSizeChanged,TResult Function( AppearanceGoogleFontPickedEvent value)?  googleFontPicked,TResult Function( AppearanceGoogleFontChosenEvent value)?  googleFontChosen,TResult Function( AppearanceDiagramRendererChangedEvent value)?  diagramRendererChanged,TResult Function( AppearanceScrollbarVisibilityChangedEvent value)?  scrollbarVisibilityChanged,TResult Function( AppearanceThemeChangedEvent value)?  themeChanged,required TResult orElse(),}){
final _that = this;
switch (_that) {
case AppearanceStartedEvent() when started != null:
return started(_that);case AppearanceThemePickedEvent() when themePicked != null:
return themePicked(_that);case AppearanceSyntaxPickedEvent() when syntaxPicked != null:
return syntaxPicked(_that);case AppearancePreviewPickedEvent() when previewPicked != null:
return previewPicked(_that);case AppearanceFollowSystemChangedEvent() when followSystemChanged != null:
return followSystemChanged(_that);case AppearanceIconThemePickedEvent() when iconThemePicked != null:
return iconThemePicked(_that);case AppearanceInterfaceFontChangedEvent() when interfaceFontChanged != null:
return interfaceFontChanged(_that);case AppearanceEditorFontChangedEvent() when editorFontChanged != null:
return editorFontChanged(_that);case AppearancePreviewFontChangedEvent() when previewFontChanged != null:
return previewFontChanged(_that);case AppearanceCodeFontChangedEvent() when codeFontChanged != null:
return codeFontChanged(_that);case AppearanceTextSizeChangedEvent() when textSizeChanged != null:
return textSizeChanged(_that);case AppearancePreviewTextSizeChangedEvent() when previewTextSizeChanged != null:
return previewTextSizeChanged(_that);case AppearanceInterfaceTextSizeChangedEvent() when interfaceTextSizeChanged != null:
return interfaceTextSizeChanged(_that);case AppearanceGoogleFontPickedEvent() when googleFontPicked != null:
return googleFontPicked(_that);case AppearanceGoogleFontChosenEvent() when googleFontChosen != null:
return googleFontChosen(_that);case AppearanceDiagramRendererChangedEvent() when diagramRendererChanged != null:
return diagramRendererChanged(_that);case AppearanceScrollbarVisibilityChangedEvent() when scrollbarVisibilityChanged != null:
return scrollbarVisibilityChanged(_that);case AppearanceThemeChangedEvent() when themeChanged != null:
return themeChanged(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( AppearanceStartedEvent value)  started,required TResult Function( AppearanceThemePickedEvent value)  themePicked,required TResult Function( AppearanceSyntaxPickedEvent value)  syntaxPicked,required TResult Function( AppearancePreviewPickedEvent value)  previewPicked,required TResult Function( AppearanceFollowSystemChangedEvent value)  followSystemChanged,required TResult Function( AppearanceIconThemePickedEvent value)  iconThemePicked,required TResult Function( AppearanceInterfaceFontChangedEvent value)  interfaceFontChanged,required TResult Function( AppearanceEditorFontChangedEvent value)  editorFontChanged,required TResult Function( AppearancePreviewFontChangedEvent value)  previewFontChanged,required TResult Function( AppearanceCodeFontChangedEvent value)  codeFontChanged,required TResult Function( AppearanceTextSizeChangedEvent value)  textSizeChanged,required TResult Function( AppearancePreviewTextSizeChangedEvent value)  previewTextSizeChanged,required TResult Function( AppearanceInterfaceTextSizeChangedEvent value)  interfaceTextSizeChanged,required TResult Function( AppearanceGoogleFontPickedEvent value)  googleFontPicked,required TResult Function( AppearanceGoogleFontChosenEvent value)  googleFontChosen,required TResult Function( AppearanceDiagramRendererChangedEvent value)  diagramRendererChanged,required TResult Function( AppearanceScrollbarVisibilityChangedEvent value)  scrollbarVisibilityChanged,required TResult Function( AppearanceThemeChangedEvent value)  themeChanged,}){
final _that = this;
switch (_that) {
case AppearanceStartedEvent():
return started(_that);case AppearanceThemePickedEvent():
return themePicked(_that);case AppearanceSyntaxPickedEvent():
return syntaxPicked(_that);case AppearancePreviewPickedEvent():
return previewPicked(_that);case AppearanceFollowSystemChangedEvent():
return followSystemChanged(_that);case AppearanceIconThemePickedEvent():
return iconThemePicked(_that);case AppearanceInterfaceFontChangedEvent():
return interfaceFontChanged(_that);case AppearanceEditorFontChangedEvent():
return editorFontChanged(_that);case AppearancePreviewFontChangedEvent():
return previewFontChanged(_that);case AppearanceCodeFontChangedEvent():
return codeFontChanged(_that);case AppearanceTextSizeChangedEvent():
return textSizeChanged(_that);case AppearancePreviewTextSizeChangedEvent():
return previewTextSizeChanged(_that);case AppearanceInterfaceTextSizeChangedEvent():
return interfaceTextSizeChanged(_that);case AppearanceGoogleFontPickedEvent():
return googleFontPicked(_that);case AppearanceGoogleFontChosenEvent():
return googleFontChosen(_that);case AppearanceDiagramRendererChangedEvent():
return diagramRendererChanged(_that);case AppearanceScrollbarVisibilityChangedEvent():
return scrollbarVisibilityChanged(_that);case AppearanceThemeChangedEvent():
return themeChanged(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( AppearanceStartedEvent value)?  started,TResult? Function( AppearanceThemePickedEvent value)?  themePicked,TResult? Function( AppearanceSyntaxPickedEvent value)?  syntaxPicked,TResult? Function( AppearancePreviewPickedEvent value)?  previewPicked,TResult? Function( AppearanceFollowSystemChangedEvent value)?  followSystemChanged,TResult? Function( AppearanceIconThemePickedEvent value)?  iconThemePicked,TResult? Function( AppearanceInterfaceFontChangedEvent value)?  interfaceFontChanged,TResult? Function( AppearanceEditorFontChangedEvent value)?  editorFontChanged,TResult? Function( AppearancePreviewFontChangedEvent value)?  previewFontChanged,TResult? Function( AppearanceCodeFontChangedEvent value)?  codeFontChanged,TResult? Function( AppearanceTextSizeChangedEvent value)?  textSizeChanged,TResult? Function( AppearancePreviewTextSizeChangedEvent value)?  previewTextSizeChanged,TResult? Function( AppearanceInterfaceTextSizeChangedEvent value)?  interfaceTextSizeChanged,TResult? Function( AppearanceGoogleFontPickedEvent value)?  googleFontPicked,TResult? Function( AppearanceGoogleFontChosenEvent value)?  googleFontChosen,TResult? Function( AppearanceDiagramRendererChangedEvent value)?  diagramRendererChanged,TResult? Function( AppearanceScrollbarVisibilityChangedEvent value)?  scrollbarVisibilityChanged,TResult? Function( AppearanceThemeChangedEvent value)?  themeChanged,}){
final _that = this;
switch (_that) {
case AppearanceStartedEvent() when started != null:
return started(_that);case AppearanceThemePickedEvent() when themePicked != null:
return themePicked(_that);case AppearanceSyntaxPickedEvent() when syntaxPicked != null:
return syntaxPicked(_that);case AppearancePreviewPickedEvent() when previewPicked != null:
return previewPicked(_that);case AppearanceFollowSystemChangedEvent() when followSystemChanged != null:
return followSystemChanged(_that);case AppearanceIconThemePickedEvent() when iconThemePicked != null:
return iconThemePicked(_that);case AppearanceInterfaceFontChangedEvent() when interfaceFontChanged != null:
return interfaceFontChanged(_that);case AppearanceEditorFontChangedEvent() when editorFontChanged != null:
return editorFontChanged(_that);case AppearancePreviewFontChangedEvent() when previewFontChanged != null:
return previewFontChanged(_that);case AppearanceCodeFontChangedEvent() when codeFontChanged != null:
return codeFontChanged(_that);case AppearanceTextSizeChangedEvent() when textSizeChanged != null:
return textSizeChanged(_that);case AppearancePreviewTextSizeChangedEvent() when previewTextSizeChanged != null:
return previewTextSizeChanged(_that);case AppearanceInterfaceTextSizeChangedEvent() when interfaceTextSizeChanged != null:
return interfaceTextSizeChanged(_that);case AppearanceGoogleFontPickedEvent() when googleFontPicked != null:
return googleFontPicked(_that);case AppearanceGoogleFontChosenEvent() when googleFontChosen != null:
return googleFontChosen(_that);case AppearanceDiagramRendererChangedEvent() when diagramRendererChanged != null:
return diagramRendererChanged(_that);case AppearanceScrollbarVisibilityChangedEvent() when scrollbarVisibilityChanged != null:
return scrollbarVisibilityChanged(_that);case AppearanceThemeChangedEvent() when themeChanged != null:
return themeChanged(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  started,TResult Function( String id,  bool dark)?  themePicked,TResult Function( String? id)?  syntaxPicked,TResult Function( String? id)?  previewPicked,TResult Function( bool follow)?  followSystemChanged,TResult Function( String id)?  iconThemePicked,TResult Function( KansoFontFamily font)?  interfaceFontChanged,TResult Function( KansoFontFamily font)?  editorFontChanged,TResult Function( KansoFontFamily font)?  previewFontChanged,TResult Function( KansoFontFamily font)?  codeFontChanged,TResult Function( double scale)?  textSizeChanged,TResult Function( double scale)?  previewTextSizeChanged,TResult Function( double scale)?  interfaceTextSizeChanged,TResult Function( String family)?  googleFontPicked,TResult Function( KansoFontSlot slot,  String family)?  googleFontChosen,TResult Function( DiagramRenderer renderer)?  diagramRendererChanged,TResult Function( bool always)?  scrollbarVisibilityChanged,TResult Function( String id)?  themeChanged,required TResult orElse(),}) {final _that = this;
switch (_that) {
case AppearanceStartedEvent() when started != null:
return started();case AppearanceThemePickedEvent() when themePicked != null:
return themePicked(_that.id,_that.dark);case AppearanceSyntaxPickedEvent() when syntaxPicked != null:
return syntaxPicked(_that.id);case AppearancePreviewPickedEvent() when previewPicked != null:
return previewPicked(_that.id);case AppearanceFollowSystemChangedEvent() when followSystemChanged != null:
return followSystemChanged(_that.follow);case AppearanceIconThemePickedEvent() when iconThemePicked != null:
return iconThemePicked(_that.id);case AppearanceInterfaceFontChangedEvent() when interfaceFontChanged != null:
return interfaceFontChanged(_that.font);case AppearanceEditorFontChangedEvent() when editorFontChanged != null:
return editorFontChanged(_that.font);case AppearancePreviewFontChangedEvent() when previewFontChanged != null:
return previewFontChanged(_that.font);case AppearanceCodeFontChangedEvent() when codeFontChanged != null:
return codeFontChanged(_that.font);case AppearanceTextSizeChangedEvent() when textSizeChanged != null:
return textSizeChanged(_that.scale);case AppearancePreviewTextSizeChangedEvent() when previewTextSizeChanged != null:
return previewTextSizeChanged(_that.scale);case AppearanceInterfaceTextSizeChangedEvent() when interfaceTextSizeChanged != null:
return interfaceTextSizeChanged(_that.scale);case AppearanceGoogleFontPickedEvent() when googleFontPicked != null:
return googleFontPicked(_that.family);case AppearanceGoogleFontChosenEvent() when googleFontChosen != null:
return googleFontChosen(_that.slot,_that.family);case AppearanceDiagramRendererChangedEvent() when diagramRendererChanged != null:
return diagramRendererChanged(_that.renderer);case AppearanceScrollbarVisibilityChangedEvent() when scrollbarVisibilityChanged != null:
return scrollbarVisibilityChanged(_that.always);case AppearanceThemeChangedEvent() when themeChanged != null:
return themeChanged(_that.id);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  started,required TResult Function( String id,  bool dark)  themePicked,required TResult Function( String? id)  syntaxPicked,required TResult Function( String? id)  previewPicked,required TResult Function( bool follow)  followSystemChanged,required TResult Function( String id)  iconThemePicked,required TResult Function( KansoFontFamily font)  interfaceFontChanged,required TResult Function( KansoFontFamily font)  editorFontChanged,required TResult Function( KansoFontFamily font)  previewFontChanged,required TResult Function( KansoFontFamily font)  codeFontChanged,required TResult Function( double scale)  textSizeChanged,required TResult Function( double scale)  previewTextSizeChanged,required TResult Function( double scale)  interfaceTextSizeChanged,required TResult Function( String family)  googleFontPicked,required TResult Function( KansoFontSlot slot,  String family)  googleFontChosen,required TResult Function( DiagramRenderer renderer)  diagramRendererChanged,required TResult Function( bool always)  scrollbarVisibilityChanged,required TResult Function( String id)  themeChanged,}) {final _that = this;
switch (_that) {
case AppearanceStartedEvent():
return started();case AppearanceThemePickedEvent():
return themePicked(_that.id,_that.dark);case AppearanceSyntaxPickedEvent():
return syntaxPicked(_that.id);case AppearancePreviewPickedEvent():
return previewPicked(_that.id);case AppearanceFollowSystemChangedEvent():
return followSystemChanged(_that.follow);case AppearanceIconThemePickedEvent():
return iconThemePicked(_that.id);case AppearanceInterfaceFontChangedEvent():
return interfaceFontChanged(_that.font);case AppearanceEditorFontChangedEvent():
return editorFontChanged(_that.font);case AppearancePreviewFontChangedEvent():
return previewFontChanged(_that.font);case AppearanceCodeFontChangedEvent():
return codeFontChanged(_that.font);case AppearanceTextSizeChangedEvent():
return textSizeChanged(_that.scale);case AppearancePreviewTextSizeChangedEvent():
return previewTextSizeChanged(_that.scale);case AppearanceInterfaceTextSizeChangedEvent():
return interfaceTextSizeChanged(_that.scale);case AppearanceGoogleFontPickedEvent():
return googleFontPicked(_that.family);case AppearanceGoogleFontChosenEvent():
return googleFontChosen(_that.slot,_that.family);case AppearanceDiagramRendererChangedEvent():
return diagramRendererChanged(_that.renderer);case AppearanceScrollbarVisibilityChangedEvent():
return scrollbarVisibilityChanged(_that.always);case AppearanceThemeChangedEvent():
return themeChanged(_that.id);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  started,TResult? Function( String id,  bool dark)?  themePicked,TResult? Function( String? id)?  syntaxPicked,TResult? Function( String? id)?  previewPicked,TResult? Function( bool follow)?  followSystemChanged,TResult? Function( String id)?  iconThemePicked,TResult? Function( KansoFontFamily font)?  interfaceFontChanged,TResult? Function( KansoFontFamily font)?  editorFontChanged,TResult? Function( KansoFontFamily font)?  previewFontChanged,TResult? Function( KansoFontFamily font)?  codeFontChanged,TResult? Function( double scale)?  textSizeChanged,TResult? Function( double scale)?  previewTextSizeChanged,TResult? Function( double scale)?  interfaceTextSizeChanged,TResult? Function( String family)?  googleFontPicked,TResult? Function( KansoFontSlot slot,  String family)?  googleFontChosen,TResult? Function( DiagramRenderer renderer)?  diagramRendererChanged,TResult? Function( bool always)?  scrollbarVisibilityChanged,TResult? Function( String id)?  themeChanged,}) {final _that = this;
switch (_that) {
case AppearanceStartedEvent() when started != null:
return started();case AppearanceThemePickedEvent() when themePicked != null:
return themePicked(_that.id,_that.dark);case AppearanceSyntaxPickedEvent() when syntaxPicked != null:
return syntaxPicked(_that.id);case AppearancePreviewPickedEvent() when previewPicked != null:
return previewPicked(_that.id);case AppearanceFollowSystemChangedEvent() when followSystemChanged != null:
return followSystemChanged(_that.follow);case AppearanceIconThemePickedEvent() when iconThemePicked != null:
return iconThemePicked(_that.id);case AppearanceInterfaceFontChangedEvent() when interfaceFontChanged != null:
return interfaceFontChanged(_that.font);case AppearanceEditorFontChangedEvent() when editorFontChanged != null:
return editorFontChanged(_that.font);case AppearancePreviewFontChangedEvent() when previewFontChanged != null:
return previewFontChanged(_that.font);case AppearanceCodeFontChangedEvent() when codeFontChanged != null:
return codeFontChanged(_that.font);case AppearanceTextSizeChangedEvent() when textSizeChanged != null:
return textSizeChanged(_that.scale);case AppearancePreviewTextSizeChangedEvent() when previewTextSizeChanged != null:
return previewTextSizeChanged(_that.scale);case AppearanceInterfaceTextSizeChangedEvent() when interfaceTextSizeChanged != null:
return interfaceTextSizeChanged(_that.scale);case AppearanceGoogleFontPickedEvent() when googleFontPicked != null:
return googleFontPicked(_that.family);case AppearanceGoogleFontChosenEvent() when googleFontChosen != null:
return googleFontChosen(_that.slot,_that.family);case AppearanceDiagramRendererChangedEvent() when diagramRendererChanged != null:
return diagramRendererChanged(_that.renderer);case AppearanceScrollbarVisibilityChangedEvent() when scrollbarVisibilityChanged != null:
return scrollbarVisibilityChanged(_that.always);case AppearanceThemeChangedEvent() when themeChanged != null:
return themeChanged(_that.id);case _:
  return null;

}
}

}

/// @nodoc


class AppearanceStartedEvent implements AppearanceEvent {
  const AppearanceStartedEvent();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceStartedEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'AppearanceEvent.started()';
}


}




/// @nodoc


class AppearanceThemePickedEvent implements AppearanceEvent {
  const AppearanceThemePickedEvent(this.id, {required this.dark});
  

 final  String id;
 final  bool dark;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceThemePickedEventCopyWith<AppearanceThemePickedEvent> get copyWith => _$AppearanceThemePickedEventCopyWithImpl<AppearanceThemePickedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceThemePickedEvent&&(identical(other.id, id) || other.id == id)&&(identical(other.dark, dark) || other.dark == dark));
}


@override
int get hashCode => Object.hash(runtimeType,id,dark);

@override
String toString() {
  return 'AppearanceEvent.themePicked(id: $id, dark: $dark)';
}


}

/// @nodoc
abstract mixin class $AppearanceThemePickedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceThemePickedEventCopyWith(AppearanceThemePickedEvent value, $Res Function(AppearanceThemePickedEvent) _then) = _$AppearanceThemePickedEventCopyWithImpl;
@useResult
$Res call({
 String id, bool dark
});




}
/// @nodoc
class _$AppearanceThemePickedEventCopyWithImpl<$Res>
    implements $AppearanceThemePickedEventCopyWith<$Res> {
  _$AppearanceThemePickedEventCopyWithImpl(this._self, this._then);

  final AppearanceThemePickedEvent _self;
  final $Res Function(AppearanceThemePickedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? dark = null,}) {
  return _then(AppearanceThemePickedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class AppearanceSyntaxPickedEvent implements AppearanceEvent {
  const AppearanceSyntaxPickedEvent(this.id);
  

 final  String? id;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceSyntaxPickedEventCopyWith<AppearanceSyntaxPickedEvent> get copyWith => _$AppearanceSyntaxPickedEventCopyWithImpl<AppearanceSyntaxPickedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceSyntaxPickedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'AppearanceEvent.syntaxPicked(id: $id)';
}


}

/// @nodoc
abstract mixin class $AppearanceSyntaxPickedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceSyntaxPickedEventCopyWith(AppearanceSyntaxPickedEvent value, $Res Function(AppearanceSyntaxPickedEvent) _then) = _$AppearanceSyntaxPickedEventCopyWithImpl;
@useResult
$Res call({
 String? id
});




}
/// @nodoc
class _$AppearanceSyntaxPickedEventCopyWithImpl<$Res>
    implements $AppearanceSyntaxPickedEventCopyWith<$Res> {
  _$AppearanceSyntaxPickedEventCopyWithImpl(this._self, this._then);

  final AppearanceSyntaxPickedEvent _self;
  final $Res Function(AppearanceSyntaxPickedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = freezed,}) {
  return _then(AppearanceSyntaxPickedEvent(
freezed == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class AppearancePreviewPickedEvent implements AppearanceEvent {
  const AppearancePreviewPickedEvent(this.id);
  

 final  String? id;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearancePreviewPickedEventCopyWith<AppearancePreviewPickedEvent> get copyWith => _$AppearancePreviewPickedEventCopyWithImpl<AppearancePreviewPickedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearancePreviewPickedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'AppearanceEvent.previewPicked(id: $id)';
}


}

/// @nodoc
abstract mixin class $AppearancePreviewPickedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearancePreviewPickedEventCopyWith(AppearancePreviewPickedEvent value, $Res Function(AppearancePreviewPickedEvent) _then) = _$AppearancePreviewPickedEventCopyWithImpl;
@useResult
$Res call({
 String? id
});




}
/// @nodoc
class _$AppearancePreviewPickedEventCopyWithImpl<$Res>
    implements $AppearancePreviewPickedEventCopyWith<$Res> {
  _$AppearancePreviewPickedEventCopyWithImpl(this._self, this._then);

  final AppearancePreviewPickedEvent _self;
  final $Res Function(AppearancePreviewPickedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = freezed,}) {
  return _then(AppearancePreviewPickedEvent(
freezed == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc


class AppearanceFollowSystemChangedEvent implements AppearanceEvent {
  const AppearanceFollowSystemChangedEvent({required this.follow});
  

 final  bool follow;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceFollowSystemChangedEventCopyWith<AppearanceFollowSystemChangedEvent> get copyWith => _$AppearanceFollowSystemChangedEventCopyWithImpl<AppearanceFollowSystemChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceFollowSystemChangedEvent&&(identical(other.follow, follow) || other.follow == follow));
}


@override
int get hashCode => Object.hash(runtimeType,follow);

@override
String toString() {
  return 'AppearanceEvent.followSystemChanged(follow: $follow)';
}


}

/// @nodoc
abstract mixin class $AppearanceFollowSystemChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceFollowSystemChangedEventCopyWith(AppearanceFollowSystemChangedEvent value, $Res Function(AppearanceFollowSystemChangedEvent) _then) = _$AppearanceFollowSystemChangedEventCopyWithImpl;
@useResult
$Res call({
 bool follow
});




}
/// @nodoc
class _$AppearanceFollowSystemChangedEventCopyWithImpl<$Res>
    implements $AppearanceFollowSystemChangedEventCopyWith<$Res> {
  _$AppearanceFollowSystemChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceFollowSystemChangedEvent _self;
  final $Res Function(AppearanceFollowSystemChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? follow = null,}) {
  return _then(AppearanceFollowSystemChangedEvent(
follow: null == follow ? _self.follow : follow // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class AppearanceIconThemePickedEvent implements AppearanceEvent {
  const AppearanceIconThemePickedEvent(this.id);
  

 final  String id;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceIconThemePickedEventCopyWith<AppearanceIconThemePickedEvent> get copyWith => _$AppearanceIconThemePickedEventCopyWithImpl<AppearanceIconThemePickedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceIconThemePickedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'AppearanceEvent.iconThemePicked(id: $id)';
}


}

/// @nodoc
abstract mixin class $AppearanceIconThemePickedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceIconThemePickedEventCopyWith(AppearanceIconThemePickedEvent value, $Res Function(AppearanceIconThemePickedEvent) _then) = _$AppearanceIconThemePickedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$AppearanceIconThemePickedEventCopyWithImpl<$Res>
    implements $AppearanceIconThemePickedEventCopyWith<$Res> {
  _$AppearanceIconThemePickedEventCopyWithImpl(this._self, this._then);

  final AppearanceIconThemePickedEvent _self;
  final $Res Function(AppearanceIconThemePickedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(AppearanceIconThemePickedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AppearanceInterfaceFontChangedEvent implements AppearanceEvent {
  const AppearanceInterfaceFontChangedEvent(this.font);
  

 final  KansoFontFamily font;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceInterfaceFontChangedEventCopyWith<AppearanceInterfaceFontChangedEvent> get copyWith => _$AppearanceInterfaceFontChangedEventCopyWithImpl<AppearanceInterfaceFontChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceInterfaceFontChangedEvent&&(identical(other.font, font) || other.font == font));
}


@override
int get hashCode => Object.hash(runtimeType,font);

@override
String toString() {
  return 'AppearanceEvent.interfaceFontChanged(font: $font)';
}


}

/// @nodoc
abstract mixin class $AppearanceInterfaceFontChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceInterfaceFontChangedEventCopyWith(AppearanceInterfaceFontChangedEvent value, $Res Function(AppearanceInterfaceFontChangedEvent) _then) = _$AppearanceInterfaceFontChangedEventCopyWithImpl;
@useResult
$Res call({
 KansoFontFamily font
});




}
/// @nodoc
class _$AppearanceInterfaceFontChangedEventCopyWithImpl<$Res>
    implements $AppearanceInterfaceFontChangedEventCopyWith<$Res> {
  _$AppearanceInterfaceFontChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceInterfaceFontChangedEvent _self;
  final $Res Function(AppearanceInterfaceFontChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? font = null,}) {
  return _then(AppearanceInterfaceFontChangedEvent(
null == font ? _self.font : font // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,
  ));
}


}

/// @nodoc


class AppearanceEditorFontChangedEvent implements AppearanceEvent {
  const AppearanceEditorFontChangedEvent(this.font);
  

 final  KansoFontFamily font;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceEditorFontChangedEventCopyWith<AppearanceEditorFontChangedEvent> get copyWith => _$AppearanceEditorFontChangedEventCopyWithImpl<AppearanceEditorFontChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceEditorFontChangedEvent&&(identical(other.font, font) || other.font == font));
}


@override
int get hashCode => Object.hash(runtimeType,font);

@override
String toString() {
  return 'AppearanceEvent.editorFontChanged(font: $font)';
}


}

/// @nodoc
abstract mixin class $AppearanceEditorFontChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceEditorFontChangedEventCopyWith(AppearanceEditorFontChangedEvent value, $Res Function(AppearanceEditorFontChangedEvent) _then) = _$AppearanceEditorFontChangedEventCopyWithImpl;
@useResult
$Res call({
 KansoFontFamily font
});




}
/// @nodoc
class _$AppearanceEditorFontChangedEventCopyWithImpl<$Res>
    implements $AppearanceEditorFontChangedEventCopyWith<$Res> {
  _$AppearanceEditorFontChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceEditorFontChangedEvent _self;
  final $Res Function(AppearanceEditorFontChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? font = null,}) {
  return _then(AppearanceEditorFontChangedEvent(
null == font ? _self.font : font // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,
  ));
}


}

/// @nodoc


class AppearancePreviewFontChangedEvent implements AppearanceEvent {
  const AppearancePreviewFontChangedEvent(this.font);
  

 final  KansoFontFamily font;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearancePreviewFontChangedEventCopyWith<AppearancePreviewFontChangedEvent> get copyWith => _$AppearancePreviewFontChangedEventCopyWithImpl<AppearancePreviewFontChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearancePreviewFontChangedEvent&&(identical(other.font, font) || other.font == font));
}


@override
int get hashCode => Object.hash(runtimeType,font);

@override
String toString() {
  return 'AppearanceEvent.previewFontChanged(font: $font)';
}


}

/// @nodoc
abstract mixin class $AppearancePreviewFontChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearancePreviewFontChangedEventCopyWith(AppearancePreviewFontChangedEvent value, $Res Function(AppearancePreviewFontChangedEvent) _then) = _$AppearancePreviewFontChangedEventCopyWithImpl;
@useResult
$Res call({
 KansoFontFamily font
});




}
/// @nodoc
class _$AppearancePreviewFontChangedEventCopyWithImpl<$Res>
    implements $AppearancePreviewFontChangedEventCopyWith<$Res> {
  _$AppearancePreviewFontChangedEventCopyWithImpl(this._self, this._then);

  final AppearancePreviewFontChangedEvent _self;
  final $Res Function(AppearancePreviewFontChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? font = null,}) {
  return _then(AppearancePreviewFontChangedEvent(
null == font ? _self.font : font // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,
  ));
}


}

/// @nodoc


class AppearanceCodeFontChangedEvent implements AppearanceEvent {
  const AppearanceCodeFontChangedEvent(this.font);
  

 final  KansoFontFamily font;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceCodeFontChangedEventCopyWith<AppearanceCodeFontChangedEvent> get copyWith => _$AppearanceCodeFontChangedEventCopyWithImpl<AppearanceCodeFontChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceCodeFontChangedEvent&&(identical(other.font, font) || other.font == font));
}


@override
int get hashCode => Object.hash(runtimeType,font);

@override
String toString() {
  return 'AppearanceEvent.codeFontChanged(font: $font)';
}


}

/// @nodoc
abstract mixin class $AppearanceCodeFontChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceCodeFontChangedEventCopyWith(AppearanceCodeFontChangedEvent value, $Res Function(AppearanceCodeFontChangedEvent) _then) = _$AppearanceCodeFontChangedEventCopyWithImpl;
@useResult
$Res call({
 KansoFontFamily font
});




}
/// @nodoc
class _$AppearanceCodeFontChangedEventCopyWithImpl<$Res>
    implements $AppearanceCodeFontChangedEventCopyWith<$Res> {
  _$AppearanceCodeFontChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceCodeFontChangedEvent _self;
  final $Res Function(AppearanceCodeFontChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? font = null,}) {
  return _then(AppearanceCodeFontChangedEvent(
null == font ? _self.font : font // ignore: cast_nullable_to_non_nullable
as KansoFontFamily,
  ));
}


}

/// @nodoc


class AppearanceTextSizeChangedEvent implements AppearanceEvent {
  const AppearanceTextSizeChangedEvent(this.scale);
  

 final  double scale;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceTextSizeChangedEventCopyWith<AppearanceTextSizeChangedEvent> get copyWith => _$AppearanceTextSizeChangedEventCopyWithImpl<AppearanceTextSizeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceTextSizeChangedEvent&&(identical(other.scale, scale) || other.scale == scale));
}


@override
int get hashCode => Object.hash(runtimeType,scale);

@override
String toString() {
  return 'AppearanceEvent.textSizeChanged(scale: $scale)';
}


}

/// @nodoc
abstract mixin class $AppearanceTextSizeChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceTextSizeChangedEventCopyWith(AppearanceTextSizeChangedEvent value, $Res Function(AppearanceTextSizeChangedEvent) _then) = _$AppearanceTextSizeChangedEventCopyWithImpl;
@useResult
$Res call({
 double scale
});




}
/// @nodoc
class _$AppearanceTextSizeChangedEventCopyWithImpl<$Res>
    implements $AppearanceTextSizeChangedEventCopyWith<$Res> {
  _$AppearanceTextSizeChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceTextSizeChangedEvent _self;
  final $Res Function(AppearanceTextSizeChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? scale = null,}) {
  return _then(AppearanceTextSizeChangedEvent(
null == scale ? _self.scale : scale // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class AppearancePreviewTextSizeChangedEvent implements AppearanceEvent {
  const AppearancePreviewTextSizeChangedEvent(this.scale);
  

 final  double scale;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearancePreviewTextSizeChangedEventCopyWith<AppearancePreviewTextSizeChangedEvent> get copyWith => _$AppearancePreviewTextSizeChangedEventCopyWithImpl<AppearancePreviewTextSizeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearancePreviewTextSizeChangedEvent&&(identical(other.scale, scale) || other.scale == scale));
}


@override
int get hashCode => Object.hash(runtimeType,scale);

@override
String toString() {
  return 'AppearanceEvent.previewTextSizeChanged(scale: $scale)';
}


}

/// @nodoc
abstract mixin class $AppearancePreviewTextSizeChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearancePreviewTextSizeChangedEventCopyWith(AppearancePreviewTextSizeChangedEvent value, $Res Function(AppearancePreviewTextSizeChangedEvent) _then) = _$AppearancePreviewTextSizeChangedEventCopyWithImpl;
@useResult
$Res call({
 double scale
});




}
/// @nodoc
class _$AppearancePreviewTextSizeChangedEventCopyWithImpl<$Res>
    implements $AppearancePreviewTextSizeChangedEventCopyWith<$Res> {
  _$AppearancePreviewTextSizeChangedEventCopyWithImpl(this._self, this._then);

  final AppearancePreviewTextSizeChangedEvent _self;
  final $Res Function(AppearancePreviewTextSizeChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? scale = null,}) {
  return _then(AppearancePreviewTextSizeChangedEvent(
null == scale ? _self.scale : scale // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class AppearanceInterfaceTextSizeChangedEvent implements AppearanceEvent {
  const AppearanceInterfaceTextSizeChangedEvent(this.scale);
  

 final  double scale;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceInterfaceTextSizeChangedEventCopyWith<AppearanceInterfaceTextSizeChangedEvent> get copyWith => _$AppearanceInterfaceTextSizeChangedEventCopyWithImpl<AppearanceInterfaceTextSizeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceInterfaceTextSizeChangedEvent&&(identical(other.scale, scale) || other.scale == scale));
}


@override
int get hashCode => Object.hash(runtimeType,scale);

@override
String toString() {
  return 'AppearanceEvent.interfaceTextSizeChanged(scale: $scale)';
}


}

/// @nodoc
abstract mixin class $AppearanceInterfaceTextSizeChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceInterfaceTextSizeChangedEventCopyWith(AppearanceInterfaceTextSizeChangedEvent value, $Res Function(AppearanceInterfaceTextSizeChangedEvent) _then) = _$AppearanceInterfaceTextSizeChangedEventCopyWithImpl;
@useResult
$Res call({
 double scale
});




}
/// @nodoc
class _$AppearanceInterfaceTextSizeChangedEventCopyWithImpl<$Res>
    implements $AppearanceInterfaceTextSizeChangedEventCopyWith<$Res> {
  _$AppearanceInterfaceTextSizeChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceInterfaceTextSizeChangedEvent _self;
  final $Res Function(AppearanceInterfaceTextSizeChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? scale = null,}) {
  return _then(AppearanceInterfaceTextSizeChangedEvent(
null == scale ? _self.scale : scale // ignore: cast_nullable_to_non_nullable
as double,
  ));
}


}

/// @nodoc


class AppearanceGoogleFontPickedEvent implements AppearanceEvent {
  const AppearanceGoogleFontPickedEvent(this.family);
  

 final  String family;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceGoogleFontPickedEventCopyWith<AppearanceGoogleFontPickedEvent> get copyWith => _$AppearanceGoogleFontPickedEventCopyWithImpl<AppearanceGoogleFontPickedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceGoogleFontPickedEvent&&(identical(other.family, family) || other.family == family));
}


@override
int get hashCode => Object.hash(runtimeType,family);

@override
String toString() {
  return 'AppearanceEvent.googleFontPicked(family: $family)';
}


}

/// @nodoc
abstract mixin class $AppearanceGoogleFontPickedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceGoogleFontPickedEventCopyWith(AppearanceGoogleFontPickedEvent value, $Res Function(AppearanceGoogleFontPickedEvent) _then) = _$AppearanceGoogleFontPickedEventCopyWithImpl;
@useResult
$Res call({
 String family
});




}
/// @nodoc
class _$AppearanceGoogleFontPickedEventCopyWithImpl<$Res>
    implements $AppearanceGoogleFontPickedEventCopyWith<$Res> {
  _$AppearanceGoogleFontPickedEventCopyWithImpl(this._self, this._then);

  final AppearanceGoogleFontPickedEvent _self;
  final $Res Function(AppearanceGoogleFontPickedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? family = null,}) {
  return _then(AppearanceGoogleFontPickedEvent(
null == family ? _self.family : family // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AppearanceGoogleFontChosenEvent implements AppearanceEvent {
  const AppearanceGoogleFontChosenEvent(this.slot, this.family);
  

 final  KansoFontSlot slot;
 final  String family;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceGoogleFontChosenEventCopyWith<AppearanceGoogleFontChosenEvent> get copyWith => _$AppearanceGoogleFontChosenEventCopyWithImpl<AppearanceGoogleFontChosenEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceGoogleFontChosenEvent&&(identical(other.slot, slot) || other.slot == slot)&&(identical(other.family, family) || other.family == family));
}


@override
int get hashCode => Object.hash(runtimeType,slot,family);

@override
String toString() {
  return 'AppearanceEvent.googleFontChosen(slot: $slot, family: $family)';
}


}

/// @nodoc
abstract mixin class $AppearanceGoogleFontChosenEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceGoogleFontChosenEventCopyWith(AppearanceGoogleFontChosenEvent value, $Res Function(AppearanceGoogleFontChosenEvent) _then) = _$AppearanceGoogleFontChosenEventCopyWithImpl;
@useResult
$Res call({
 KansoFontSlot slot, String family
});




}
/// @nodoc
class _$AppearanceGoogleFontChosenEventCopyWithImpl<$Res>
    implements $AppearanceGoogleFontChosenEventCopyWith<$Res> {
  _$AppearanceGoogleFontChosenEventCopyWithImpl(this._self, this._then);

  final AppearanceGoogleFontChosenEvent _self;
  final $Res Function(AppearanceGoogleFontChosenEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? slot = null,Object? family = null,}) {
  return _then(AppearanceGoogleFontChosenEvent(
null == slot ? _self.slot : slot // ignore: cast_nullable_to_non_nullable
as KansoFontSlot,null == family ? _self.family : family // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class AppearanceDiagramRendererChangedEvent implements AppearanceEvent {
  const AppearanceDiagramRendererChangedEvent(this.renderer);
  

 final  DiagramRenderer renderer;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceDiagramRendererChangedEventCopyWith<AppearanceDiagramRendererChangedEvent> get copyWith => _$AppearanceDiagramRendererChangedEventCopyWithImpl<AppearanceDiagramRendererChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceDiagramRendererChangedEvent&&(identical(other.renderer, renderer) || other.renderer == renderer));
}


@override
int get hashCode => Object.hash(runtimeType,renderer);

@override
String toString() {
  return 'AppearanceEvent.diagramRendererChanged(renderer: $renderer)';
}


}

/// @nodoc
abstract mixin class $AppearanceDiagramRendererChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceDiagramRendererChangedEventCopyWith(AppearanceDiagramRendererChangedEvent value, $Res Function(AppearanceDiagramRendererChangedEvent) _then) = _$AppearanceDiagramRendererChangedEventCopyWithImpl;
@useResult
$Res call({
 DiagramRenderer renderer
});




}
/// @nodoc
class _$AppearanceDiagramRendererChangedEventCopyWithImpl<$Res>
    implements $AppearanceDiagramRendererChangedEventCopyWith<$Res> {
  _$AppearanceDiagramRendererChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceDiagramRendererChangedEvent _self;
  final $Res Function(AppearanceDiagramRendererChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? renderer = null,}) {
  return _then(AppearanceDiagramRendererChangedEvent(
null == renderer ? _self.renderer : renderer // ignore: cast_nullable_to_non_nullable
as DiagramRenderer,
  ));
}


}

/// @nodoc


class AppearanceScrollbarVisibilityChangedEvent implements AppearanceEvent {
  const AppearanceScrollbarVisibilityChangedEvent({required this.always});
  

 final  bool always;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceScrollbarVisibilityChangedEventCopyWith<AppearanceScrollbarVisibilityChangedEvent> get copyWith => _$AppearanceScrollbarVisibilityChangedEventCopyWithImpl<AppearanceScrollbarVisibilityChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceScrollbarVisibilityChangedEvent&&(identical(other.always, always) || other.always == always));
}


@override
int get hashCode => Object.hash(runtimeType,always);

@override
String toString() {
  return 'AppearanceEvent.scrollbarVisibilityChanged(always: $always)';
}


}

/// @nodoc
abstract mixin class $AppearanceScrollbarVisibilityChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceScrollbarVisibilityChangedEventCopyWith(AppearanceScrollbarVisibilityChangedEvent value, $Res Function(AppearanceScrollbarVisibilityChangedEvent) _then) = _$AppearanceScrollbarVisibilityChangedEventCopyWithImpl;
@useResult
$Res call({
 bool always
});




}
/// @nodoc
class _$AppearanceScrollbarVisibilityChangedEventCopyWithImpl<$Res>
    implements $AppearanceScrollbarVisibilityChangedEventCopyWith<$Res> {
  _$AppearanceScrollbarVisibilityChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceScrollbarVisibilityChangedEvent _self;
  final $Res Function(AppearanceScrollbarVisibilityChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? always = null,}) {
  return _then(AppearanceScrollbarVisibilityChangedEvent(
always: null == always ? _self.always : always // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class AppearanceThemeChangedEvent implements AppearanceEvent {
  const AppearanceThemeChangedEvent(this.id);
  

 final  String id;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceThemeChangedEventCopyWith<AppearanceThemeChangedEvent> get copyWith => _$AppearanceThemeChangedEventCopyWithImpl<AppearanceThemeChangedEvent>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceThemeChangedEvent&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'AppearanceEvent.themeChanged(id: $id)';
}


}

/// @nodoc
abstract mixin class $AppearanceThemeChangedEventCopyWith<$Res> implements $AppearanceEventCopyWith<$Res> {
  factory $AppearanceThemeChangedEventCopyWith(AppearanceThemeChangedEvent value, $Res Function(AppearanceThemeChangedEvent) _then) = _$AppearanceThemeChangedEventCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$AppearanceThemeChangedEventCopyWithImpl<$Res>
    implements $AppearanceThemeChangedEventCopyWith<$Res> {
  _$AppearanceThemeChangedEventCopyWithImpl(this._self, this._then);

  final AppearanceThemeChangedEvent _self;
  final $Res Function(AppearanceThemeChangedEvent) _then;

/// Create a copy of AppearanceEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(AppearanceThemeChangedEvent(
null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$AppearanceState {

/// The themes chosen.
 ThemeSelection get selection;/// The palette of the app's light theme: the light slot while the system
/// is followed, the fixed theme otherwise. It is drawn from the tokens
/// the core fitted for the interface, syntax and preview themes of the
/// slot together, so [KansoPalette.syntax] is the syntax theme's and
/// [KansoPalette.preview] the preview theme's.
 KansoPalette get light;/// The palette of the app's dark theme: the dark slot while the system
/// is followed, the fixed theme otherwise (so the same as [light]).
 KansoPalette get dark;/// The icon theme every icon is drawn in.
 KansoIconSet get iconSet; KansoFonts get fonts;/// Whether the selected Google Font is downloading or loading from cache.
 bool get googleFontLoading;/// A Google Fonts failure detail used by the settings feedback.
 String? get googleFontError;/// Whether the last Google Fonts operation failed because the network
/// could not be reached.
 bool get googleFontOffline;/// The editor's note text scale.
 double get textSize;/// The rendered preview's note text scale. The editor does not use it.
 double get previewTextSize;/// The interface's label and text scale. Note text has its own scales.
 double get interfaceTextSize;/// What draws Mermaid diagrams.
 DiagramRenderer get diagramRenderer;/// Keep overflowing list scrollbars visible when idle.
 bool get alwaysShowScrollbars; AppearanceStatus get status;
/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$AppearanceStateCopyWith<AppearanceState> get copyWith => _$AppearanceStateCopyWithImpl<AppearanceState>(this as AppearanceState, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is AppearanceState&&(identical(other.selection, selection) || other.selection == selection)&&(identical(other.light, light) || other.light == light)&&(identical(other.dark, dark) || other.dark == dark)&&(identical(other.iconSet, iconSet) || other.iconSet == iconSet)&&(identical(other.fonts, fonts) || other.fonts == fonts)&&(identical(other.googleFontLoading, googleFontLoading) || other.googleFontLoading == googleFontLoading)&&(identical(other.googleFontError, googleFontError) || other.googleFontError == googleFontError)&&(identical(other.googleFontOffline, googleFontOffline) || other.googleFontOffline == googleFontOffline)&&(identical(other.textSize, textSize) || other.textSize == textSize)&&(identical(other.previewTextSize, previewTextSize) || other.previewTextSize == previewTextSize)&&(identical(other.interfaceTextSize, interfaceTextSize) || other.interfaceTextSize == interfaceTextSize)&&(identical(other.diagramRenderer, diagramRenderer) || other.diagramRenderer == diagramRenderer)&&(identical(other.alwaysShowScrollbars, alwaysShowScrollbars) || other.alwaysShowScrollbars == alwaysShowScrollbars)&&(identical(other.status, status) || other.status == status));
}


@override
int get hashCode => Object.hash(runtimeType,selection,light,dark,iconSet,fonts,googleFontLoading,googleFontError,googleFontOffline,textSize,previewTextSize,interfaceTextSize,diagramRenderer,alwaysShowScrollbars,status);

@override
String toString() {
  return 'AppearanceState(selection: $selection, light: $light, dark: $dark, iconSet: $iconSet, fonts: $fonts, googleFontLoading: $googleFontLoading, googleFontError: $googleFontError, googleFontOffline: $googleFontOffline, textSize: $textSize, previewTextSize: $previewTextSize, interfaceTextSize: $interfaceTextSize, diagramRenderer: $diagramRenderer, alwaysShowScrollbars: $alwaysShowScrollbars, status: $status)';
}


}

/// @nodoc
abstract mixin class $AppearanceStateCopyWith<$Res>  {
  factory $AppearanceStateCopyWith(AppearanceState value, $Res Function(AppearanceState) _then) = _$AppearanceStateCopyWithImpl;
@useResult
$Res call({
 ThemeSelection selection, KansoPalette light, KansoPalette dark, KansoIconSet iconSet, KansoFonts fonts, bool googleFontLoading, String? googleFontError, bool googleFontOffline, double textSize, double previewTextSize, double interfaceTextSize, DiagramRenderer diagramRenderer, bool alwaysShowScrollbars, AppearanceStatus status
});


$ThemeSelectionCopyWith<$Res> get selection;$KansoPaletteCopyWith<$Res> get light;$KansoPaletteCopyWith<$Res> get dark;$KansoIconSetCopyWith<$Res> get iconSet;$KansoFontsCopyWith<$Res> get fonts;

}
/// @nodoc
class _$AppearanceStateCopyWithImpl<$Res>
    implements $AppearanceStateCopyWith<$Res> {
  _$AppearanceStateCopyWithImpl(this._self, this._then);

  final AppearanceState _self;
  final $Res Function(AppearanceState) _then;

/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? selection = null,Object? light = null,Object? dark = null,Object? iconSet = null,Object? fonts = null,Object? googleFontLoading = null,Object? googleFontError = freezed,Object? googleFontOffline = null,Object? textSize = null,Object? previewTextSize = null,Object? interfaceTextSize = null,Object? diagramRenderer = null,Object? alwaysShowScrollbars = null,Object? status = null,}) {
  return _then(_self.copyWith(
selection: null == selection ? _self.selection : selection // ignore: cast_nullable_to_non_nullable
as ThemeSelection,light: null == light ? _self.light : light // ignore: cast_nullable_to_non_nullable
as KansoPalette,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as KansoPalette,iconSet: null == iconSet ? _self.iconSet : iconSet // ignore: cast_nullable_to_non_nullable
as KansoIconSet,fonts: null == fonts ? _self.fonts : fonts // ignore: cast_nullable_to_non_nullable
as KansoFonts,googleFontLoading: null == googleFontLoading ? _self.googleFontLoading : googleFontLoading // ignore: cast_nullable_to_non_nullable
as bool,googleFontError: freezed == googleFontError ? _self.googleFontError : googleFontError // ignore: cast_nullable_to_non_nullable
as String?,googleFontOffline: null == googleFontOffline ? _self.googleFontOffline : googleFontOffline // ignore: cast_nullable_to_non_nullable
as bool,textSize: null == textSize ? _self.textSize : textSize // ignore: cast_nullable_to_non_nullable
as double,previewTextSize: null == previewTextSize ? _self.previewTextSize : previewTextSize // ignore: cast_nullable_to_non_nullable
as double,interfaceTextSize: null == interfaceTextSize ? _self.interfaceTextSize : interfaceTextSize // ignore: cast_nullable_to_non_nullable
as double,diagramRenderer: null == diagramRenderer ? _self.diagramRenderer : diagramRenderer // ignore: cast_nullable_to_non_nullable
as DiagramRenderer,alwaysShowScrollbars: null == alwaysShowScrollbars ? _self.alwaysShowScrollbars : alwaysShowScrollbars // ignore: cast_nullable_to_non_nullable
as bool,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as AppearanceStatus,
  ));
}
/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ThemeSelectionCopyWith<$Res> get selection {
  
  return $ThemeSelectionCopyWith<$Res>(_self.selection, (value) {
    return _then(_self.copyWith(selection: value));
  });
}/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoPaletteCopyWith<$Res> get light {
  
  return $KansoPaletteCopyWith<$Res>(_self.light, (value) {
    return _then(_self.copyWith(light: value));
  });
}/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoPaletteCopyWith<$Res> get dark {
  
  return $KansoPaletteCopyWith<$Res>(_self.dark, (value) {
    return _then(_self.copyWith(dark: value));
  });
}/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoIconSetCopyWith<$Res> get iconSet {
  
  return $KansoIconSetCopyWith<$Res>(_self.iconSet, (value) {
    return _then(_self.copyWith(iconSet: value));
  });
}/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoFontsCopyWith<$Res> get fonts {
  
  return $KansoFontsCopyWith<$Res>(_self.fonts, (value) {
    return _then(_self.copyWith(fonts: value));
  });
}
}


/// Adds pattern-matching-related methods to [AppearanceState].
extension AppearanceStatePatterns on AppearanceState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _AppearanceState value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _AppearanceState() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _AppearanceState value)  $default,){
final _that = this;
switch (_that) {
case _AppearanceState():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _AppearanceState value)?  $default,){
final _that = this;
switch (_that) {
case _AppearanceState() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( ThemeSelection selection,  KansoPalette light,  KansoPalette dark,  KansoIconSet iconSet,  KansoFonts fonts,  bool googleFontLoading,  String? googleFontError,  bool googleFontOffline,  double textSize,  double previewTextSize,  double interfaceTextSize,  DiagramRenderer diagramRenderer,  bool alwaysShowScrollbars,  AppearanceStatus status)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _AppearanceState() when $default != null:
return $default(_that.selection,_that.light,_that.dark,_that.iconSet,_that.fonts,_that.googleFontLoading,_that.googleFontError,_that.googleFontOffline,_that.textSize,_that.previewTextSize,_that.interfaceTextSize,_that.diagramRenderer,_that.alwaysShowScrollbars,_that.status);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( ThemeSelection selection,  KansoPalette light,  KansoPalette dark,  KansoIconSet iconSet,  KansoFonts fonts,  bool googleFontLoading,  String? googleFontError,  bool googleFontOffline,  double textSize,  double previewTextSize,  double interfaceTextSize,  DiagramRenderer diagramRenderer,  bool alwaysShowScrollbars,  AppearanceStatus status)  $default,) {final _that = this;
switch (_that) {
case _AppearanceState():
return $default(_that.selection,_that.light,_that.dark,_that.iconSet,_that.fonts,_that.googleFontLoading,_that.googleFontError,_that.googleFontOffline,_that.textSize,_that.previewTextSize,_that.interfaceTextSize,_that.diagramRenderer,_that.alwaysShowScrollbars,_that.status);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( ThemeSelection selection,  KansoPalette light,  KansoPalette dark,  KansoIconSet iconSet,  KansoFonts fonts,  bool googleFontLoading,  String? googleFontError,  bool googleFontOffline,  double textSize,  double previewTextSize,  double interfaceTextSize,  DiagramRenderer diagramRenderer,  bool alwaysShowScrollbars,  AppearanceStatus status)?  $default,) {final _that = this;
switch (_that) {
case _AppearanceState() when $default != null:
return $default(_that.selection,_that.light,_that.dark,_that.iconSet,_that.fonts,_that.googleFontLoading,_that.googleFontError,_that.googleFontOffline,_that.textSize,_that.previewTextSize,_that.interfaceTextSize,_that.diagramRenderer,_that.alwaysShowScrollbars,_that.status);case _:
  return null;

}
}

}

/// @nodoc


class _AppearanceState extends AppearanceState {
  const _AppearanceState({this.selection = const ThemeSelection(), this.light = KansoPalette.kansoLight, this.dark = KansoPalette.kansoDark, this.iconSet = KansoIconSet.lucide, this.fonts = const KansoFonts(), this.googleFontLoading = false, this.googleFontError, this.googleFontOffline = false, this.textSize = 1.0, this.previewTextSize = 1.0, this.interfaceTextSize = 1.0, this.diagramRenderer = DiagramRenderer.pele, this.alwaysShowScrollbars = false, this.status = AppearanceStatus.loading}): super._();
  

/// The themes chosen.
@override@JsonKey() final  ThemeSelection selection;
/// The palette of the app's light theme: the light slot while the system
/// is followed, the fixed theme otherwise. It is drawn from the tokens
/// the core fitted for the interface, syntax and preview themes of the
/// slot together, so [KansoPalette.syntax] is the syntax theme's and
/// [KansoPalette.preview] the preview theme's.
@override@JsonKey() final  KansoPalette light;
/// The palette of the app's dark theme: the dark slot while the system
/// is followed, the fixed theme otherwise (so the same as [light]).
@override@JsonKey() final  KansoPalette dark;
/// The icon theme every icon is drawn in.
@override@JsonKey() final  KansoIconSet iconSet;
@override@JsonKey() final  KansoFonts fonts;
/// Whether the selected Google Font is downloading or loading from cache.
@override@JsonKey() final  bool googleFontLoading;
/// A Google Fonts failure detail used by the settings feedback.
@override final  String? googleFontError;
/// Whether the last Google Fonts operation failed because the network
/// could not be reached.
@override@JsonKey() final  bool googleFontOffline;
/// The editor's note text scale.
@override@JsonKey() final  double textSize;
/// The rendered preview's note text scale. The editor does not use it.
@override@JsonKey() final  double previewTextSize;
/// The interface's label and text scale. Note text has its own scales.
@override@JsonKey() final  double interfaceTextSize;
/// What draws Mermaid diagrams.
@override@JsonKey() final  DiagramRenderer diagramRenderer;
/// Keep overflowing list scrollbars visible when idle.
@override@JsonKey() final  bool alwaysShowScrollbars;
@override@JsonKey() final  AppearanceStatus status;

/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$AppearanceStateCopyWith<_AppearanceState> get copyWith => __$AppearanceStateCopyWithImpl<_AppearanceState>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _AppearanceState&&(identical(other.selection, selection) || other.selection == selection)&&(identical(other.light, light) || other.light == light)&&(identical(other.dark, dark) || other.dark == dark)&&(identical(other.iconSet, iconSet) || other.iconSet == iconSet)&&(identical(other.fonts, fonts) || other.fonts == fonts)&&(identical(other.googleFontLoading, googleFontLoading) || other.googleFontLoading == googleFontLoading)&&(identical(other.googleFontError, googleFontError) || other.googleFontError == googleFontError)&&(identical(other.googleFontOffline, googleFontOffline) || other.googleFontOffline == googleFontOffline)&&(identical(other.textSize, textSize) || other.textSize == textSize)&&(identical(other.previewTextSize, previewTextSize) || other.previewTextSize == previewTextSize)&&(identical(other.interfaceTextSize, interfaceTextSize) || other.interfaceTextSize == interfaceTextSize)&&(identical(other.diagramRenderer, diagramRenderer) || other.diagramRenderer == diagramRenderer)&&(identical(other.alwaysShowScrollbars, alwaysShowScrollbars) || other.alwaysShowScrollbars == alwaysShowScrollbars)&&(identical(other.status, status) || other.status == status));
}


@override
int get hashCode => Object.hash(runtimeType,selection,light,dark,iconSet,fonts,googleFontLoading,googleFontError,googleFontOffline,textSize,previewTextSize,interfaceTextSize,diagramRenderer,alwaysShowScrollbars,status);

@override
String toString() {
  return 'AppearanceState(selection: $selection, light: $light, dark: $dark, iconSet: $iconSet, fonts: $fonts, googleFontLoading: $googleFontLoading, googleFontError: $googleFontError, googleFontOffline: $googleFontOffline, textSize: $textSize, previewTextSize: $previewTextSize, interfaceTextSize: $interfaceTextSize, diagramRenderer: $diagramRenderer, alwaysShowScrollbars: $alwaysShowScrollbars, status: $status)';
}


}

/// @nodoc
abstract mixin class _$AppearanceStateCopyWith<$Res> implements $AppearanceStateCopyWith<$Res> {
  factory _$AppearanceStateCopyWith(_AppearanceState value, $Res Function(_AppearanceState) _then) = __$AppearanceStateCopyWithImpl;
@override @useResult
$Res call({
 ThemeSelection selection, KansoPalette light, KansoPalette dark, KansoIconSet iconSet, KansoFonts fonts, bool googleFontLoading, String? googleFontError, bool googleFontOffline, double textSize, double previewTextSize, double interfaceTextSize, DiagramRenderer diagramRenderer, bool alwaysShowScrollbars, AppearanceStatus status
});


@override $ThemeSelectionCopyWith<$Res> get selection;@override $KansoPaletteCopyWith<$Res> get light;@override $KansoPaletteCopyWith<$Res> get dark;@override $KansoIconSetCopyWith<$Res> get iconSet;@override $KansoFontsCopyWith<$Res> get fonts;

}
/// @nodoc
class __$AppearanceStateCopyWithImpl<$Res>
    implements _$AppearanceStateCopyWith<$Res> {
  __$AppearanceStateCopyWithImpl(this._self, this._then);

  final _AppearanceState _self;
  final $Res Function(_AppearanceState) _then;

/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? selection = null,Object? light = null,Object? dark = null,Object? iconSet = null,Object? fonts = null,Object? googleFontLoading = null,Object? googleFontError = freezed,Object? googleFontOffline = null,Object? textSize = null,Object? previewTextSize = null,Object? interfaceTextSize = null,Object? diagramRenderer = null,Object? alwaysShowScrollbars = null,Object? status = null,}) {
  return _then(_AppearanceState(
selection: null == selection ? _self.selection : selection // ignore: cast_nullable_to_non_nullable
as ThemeSelection,light: null == light ? _self.light : light // ignore: cast_nullable_to_non_nullable
as KansoPalette,dark: null == dark ? _self.dark : dark // ignore: cast_nullable_to_non_nullable
as KansoPalette,iconSet: null == iconSet ? _self.iconSet : iconSet // ignore: cast_nullable_to_non_nullable
as KansoIconSet,fonts: null == fonts ? _self.fonts : fonts // ignore: cast_nullable_to_non_nullable
as KansoFonts,googleFontLoading: null == googleFontLoading ? _self.googleFontLoading : googleFontLoading // ignore: cast_nullable_to_non_nullable
as bool,googleFontError: freezed == googleFontError ? _self.googleFontError : googleFontError // ignore: cast_nullable_to_non_nullable
as String?,googleFontOffline: null == googleFontOffline ? _self.googleFontOffline : googleFontOffline // ignore: cast_nullable_to_non_nullable
as bool,textSize: null == textSize ? _self.textSize : textSize // ignore: cast_nullable_to_non_nullable
as double,previewTextSize: null == previewTextSize ? _self.previewTextSize : previewTextSize // ignore: cast_nullable_to_non_nullable
as double,interfaceTextSize: null == interfaceTextSize ? _self.interfaceTextSize : interfaceTextSize // ignore: cast_nullable_to_non_nullable
as double,diagramRenderer: null == diagramRenderer ? _self.diagramRenderer : diagramRenderer // ignore: cast_nullable_to_non_nullable
as DiagramRenderer,alwaysShowScrollbars: null == alwaysShowScrollbars ? _self.alwaysShowScrollbars : alwaysShowScrollbars // ignore: cast_nullable_to_non_nullable
as bool,status: null == status ? _self.status : status // ignore: cast_nullable_to_non_nullable
as AppearanceStatus,
  ));
}

/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$ThemeSelectionCopyWith<$Res> get selection {
  
  return $ThemeSelectionCopyWith<$Res>(_self.selection, (value) {
    return _then(_self.copyWith(selection: value));
  });
}/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoPaletteCopyWith<$Res> get light {
  
  return $KansoPaletteCopyWith<$Res>(_self.light, (value) {
    return _then(_self.copyWith(light: value));
  });
}/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoPaletteCopyWith<$Res> get dark {
  
  return $KansoPaletteCopyWith<$Res>(_self.dark, (value) {
    return _then(_self.copyWith(dark: value));
  });
}/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoIconSetCopyWith<$Res> get iconSet {
  
  return $KansoIconSetCopyWith<$Res>(_self.iconSet, (value) {
    return _then(_self.copyWith(iconSet: value));
  });
}/// Create a copy of AppearanceState
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$KansoFontsCopyWith<$Res> get fonts {
  
  return $KansoFontsCopyWith<$Res>(_self.fonts, (value) {
    return _then(_self.copyWith(fonts: value));
  });
}
}

// dart format on

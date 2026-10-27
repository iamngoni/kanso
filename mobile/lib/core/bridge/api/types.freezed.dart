// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$EditCommand {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditCommand()';
}


}

/// @nodoc
class $EditCommandCopyWith<$Res>  {
$EditCommandCopyWith(EditCommand _, $Res Function(EditCommand) __);
}


/// Adds pattern-matching-related methods to [EditCommand].
extension EditCommandPatterns on EditCommand {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( EditCommand_Enter value)?  enter,TResult Function( EditCommand_Tab value)?  tab,TResult Function( EditCommand_Format value)?  format,TResult Function( EditCommand_CycleHeading value)?  cycleHeading,TResult Function( EditCommand_WikiLink value)?  wikiLink,TResult Function( EditCommand_Insert value)?  insert,required TResult orElse(),}){
final _that = this;
switch (_that) {
case EditCommand_Enter() when enter != null:
return enter(_that);case EditCommand_Tab() when tab != null:
return tab(_that);case EditCommand_Format() when format != null:
return format(_that);case EditCommand_CycleHeading() when cycleHeading != null:
return cycleHeading(_that);case EditCommand_WikiLink() when wikiLink != null:
return wikiLink(_that);case EditCommand_Insert() when insert != null:
return insert(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( EditCommand_Enter value)  enter,required TResult Function( EditCommand_Tab value)  tab,required TResult Function( EditCommand_Format value)  format,required TResult Function( EditCommand_CycleHeading value)  cycleHeading,required TResult Function( EditCommand_WikiLink value)  wikiLink,required TResult Function( EditCommand_Insert value)  insert,}){
final _that = this;
switch (_that) {
case EditCommand_Enter():
return enter(_that);case EditCommand_Tab():
return tab(_that);case EditCommand_Format():
return format(_that);case EditCommand_CycleHeading():
return cycleHeading(_that);case EditCommand_WikiLink():
return wikiLink(_that);case EditCommand_Insert():
return insert(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( EditCommand_Enter value)?  enter,TResult? Function( EditCommand_Tab value)?  tab,TResult? Function( EditCommand_Format value)?  format,TResult? Function( EditCommand_CycleHeading value)?  cycleHeading,TResult? Function( EditCommand_WikiLink value)?  wikiLink,TResult? Function( EditCommand_Insert value)?  insert,}){
final _that = this;
switch (_that) {
case EditCommand_Enter() when enter != null:
return enter(_that);case EditCommand_Tab() when tab != null:
return tab(_that);case EditCommand_Format() when format != null:
return format(_that);case EditCommand_CycleHeading() when cycleHeading != null:
return cycleHeading(_that);case EditCommand_WikiLink() when wikiLink != null:
return wikiLink(_that);case EditCommand_Insert() when insert != null:
return insert(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( bool plain)?  enter,TResult Function( bool out)?  tab,TResult Function( FormatKind format)?  format,TResult Function()?  cycleHeading,TResult Function()?  wikiLink,TResult Function( Range16 range,  String text)?  insert,required TResult orElse(),}) {final _that = this;
switch (_that) {
case EditCommand_Enter() when enter != null:
return enter(_that.plain);case EditCommand_Tab() when tab != null:
return tab(_that.out);case EditCommand_Format() when format != null:
return format(_that.format);case EditCommand_CycleHeading() when cycleHeading != null:
return cycleHeading();case EditCommand_WikiLink() when wikiLink != null:
return wikiLink();case EditCommand_Insert() when insert != null:
return insert(_that.range,_that.text);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( bool plain)  enter,required TResult Function( bool out)  tab,required TResult Function( FormatKind format)  format,required TResult Function()  cycleHeading,required TResult Function()  wikiLink,required TResult Function( Range16 range,  String text)  insert,}) {final _that = this;
switch (_that) {
case EditCommand_Enter():
return enter(_that.plain);case EditCommand_Tab():
return tab(_that.out);case EditCommand_Format():
return format(_that.format);case EditCommand_CycleHeading():
return cycleHeading();case EditCommand_WikiLink():
return wikiLink();case EditCommand_Insert():
return insert(_that.range,_that.text);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( bool plain)?  enter,TResult? Function( bool out)?  tab,TResult? Function( FormatKind format)?  format,TResult? Function()?  cycleHeading,TResult? Function()?  wikiLink,TResult? Function( Range16 range,  String text)?  insert,}) {final _that = this;
switch (_that) {
case EditCommand_Enter() when enter != null:
return enter(_that.plain);case EditCommand_Tab() when tab != null:
return tab(_that.out);case EditCommand_Format() when format != null:
return format(_that.format);case EditCommand_CycleHeading() when cycleHeading != null:
return cycleHeading();case EditCommand_WikiLink() when wikiLink != null:
return wikiLink();case EditCommand_Insert() when insert != null:
return insert(_that.range,_that.text);case _:
  return null;

}
}

}

/// @nodoc


class EditCommand_Enter extends EditCommand {
  const EditCommand_Enter({required this.plain}): super._();
  

 final  bool plain;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditCommand_EnterCopyWith<EditCommand_Enter> get copyWith => _$EditCommand_EnterCopyWithImpl<EditCommand_Enter>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_Enter&&(identical(other.plain, plain) || other.plain == plain));
}


@override
int get hashCode => Object.hash(runtimeType,plain);

@override
String toString() {
  return 'EditCommand.enter(plain: $plain)';
}


}

/// @nodoc
abstract mixin class $EditCommand_EnterCopyWith<$Res> implements $EditCommandCopyWith<$Res> {
  factory $EditCommand_EnterCopyWith(EditCommand_Enter value, $Res Function(EditCommand_Enter) _then) = _$EditCommand_EnterCopyWithImpl;
@useResult
$Res call({
 bool plain
});




}
/// @nodoc
class _$EditCommand_EnterCopyWithImpl<$Res>
    implements $EditCommand_EnterCopyWith<$Res> {
  _$EditCommand_EnterCopyWithImpl(this._self, this._then);

  final EditCommand_Enter _self;
  final $Res Function(EditCommand_Enter) _then;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? plain = null,}) {
  return _then(EditCommand_Enter(
plain: null == plain ? _self.plain : plain // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class EditCommand_Tab extends EditCommand {
  const EditCommand_Tab({required this.out}): super._();
  

 final  bool out;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditCommand_TabCopyWith<EditCommand_Tab> get copyWith => _$EditCommand_TabCopyWithImpl<EditCommand_Tab>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_Tab&&(identical(other.out, out) || other.out == out));
}


@override
int get hashCode => Object.hash(runtimeType,out);

@override
String toString() {
  return 'EditCommand.tab(out: $out)';
}


}

/// @nodoc
abstract mixin class $EditCommand_TabCopyWith<$Res> implements $EditCommandCopyWith<$Res> {
  factory $EditCommand_TabCopyWith(EditCommand_Tab value, $Res Function(EditCommand_Tab) _then) = _$EditCommand_TabCopyWithImpl;
@useResult
$Res call({
 bool out
});




}
/// @nodoc
class _$EditCommand_TabCopyWithImpl<$Res>
    implements $EditCommand_TabCopyWith<$Res> {
  _$EditCommand_TabCopyWithImpl(this._self, this._then);

  final EditCommand_Tab _self;
  final $Res Function(EditCommand_Tab) _then;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? out = null,}) {
  return _then(EditCommand_Tab(
out: null == out ? _self.out : out // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class EditCommand_Format extends EditCommand {
  const EditCommand_Format({required this.format}): super._();
  

 final  FormatKind format;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditCommand_FormatCopyWith<EditCommand_Format> get copyWith => _$EditCommand_FormatCopyWithImpl<EditCommand_Format>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_Format&&(identical(other.format, format) || other.format == format));
}


@override
int get hashCode => Object.hash(runtimeType,format);

@override
String toString() {
  return 'EditCommand.format(format: $format)';
}


}

/// @nodoc
abstract mixin class $EditCommand_FormatCopyWith<$Res> implements $EditCommandCopyWith<$Res> {
  factory $EditCommand_FormatCopyWith(EditCommand_Format value, $Res Function(EditCommand_Format) _then) = _$EditCommand_FormatCopyWithImpl;
@useResult
$Res call({
 FormatKind format
});




}
/// @nodoc
class _$EditCommand_FormatCopyWithImpl<$Res>
    implements $EditCommand_FormatCopyWith<$Res> {
  _$EditCommand_FormatCopyWithImpl(this._self, this._then);

  final EditCommand_Format _self;
  final $Res Function(EditCommand_Format) _then;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? format = null,}) {
  return _then(EditCommand_Format(
format: null == format ? _self.format : format // ignore: cast_nullable_to_non_nullable
as FormatKind,
  ));
}


}

/// @nodoc


class EditCommand_CycleHeading extends EditCommand {
  const EditCommand_CycleHeading(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_CycleHeading);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditCommand.cycleHeading()';
}


}




/// @nodoc


class EditCommand_WikiLink extends EditCommand {
  const EditCommand_WikiLink(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_WikiLink);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditCommand.wikiLink()';
}


}




/// @nodoc


class EditCommand_Insert extends EditCommand {
  const EditCommand_Insert({required this.range, required this.text}): super._();
  

 final  Range16 range;
 final  String text;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditCommand_InsertCopyWith<EditCommand_Insert> get copyWith => _$EditCommand_InsertCopyWithImpl<EditCommand_Insert>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditCommand_Insert&&(identical(other.range, range) || other.range == range)&&(identical(other.text, text) || other.text == text));
}


@override
int get hashCode => Object.hash(runtimeType,range,text);

@override
String toString() {
  return 'EditCommand.insert(range: $range, text: $text)';
}


}

/// @nodoc
abstract mixin class $EditCommand_InsertCopyWith<$Res> implements $EditCommandCopyWith<$Res> {
  factory $EditCommand_InsertCopyWith(EditCommand_Insert value, $Res Function(EditCommand_Insert) _then) = _$EditCommand_InsertCopyWithImpl;
@useResult
$Res call({
 Range16 range, String text
});




}
/// @nodoc
class _$EditCommand_InsertCopyWithImpl<$Res>
    implements $EditCommand_InsertCopyWith<$Res> {
  _$EditCommand_InsertCopyWithImpl(this._self, this._then);

  final EditCommand_Insert _self;
  final $Res Function(EditCommand_Insert) _then;

/// Create a copy of EditCommand
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? range = null,Object? text = null,}) {
  return _then(EditCommand_Insert(
range: null == range ? _self.range : range // ignore: cast_nullable_to_non_nullable
as Range16,text: null == text ? _self.text : text // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$EditorEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent()';
}


}

/// @nodoc
class $EditorEventCopyWith<$Res>  {
$EditorEventCopyWith(EditorEvent _, $Res Function(EditorEvent) __);
}


/// Adds pattern-matching-related methods to [EditorEvent].
extension EditorEventPatterns on EditorEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( EditorEvent_Saved value)?  saved,TResult Function( EditorEvent_Incoming value)?  incoming,TResult Function( EditorEvent_Reloaded value)?  reloaded,TResult Function( EditorEvent_Closed value)?  closed,TResult Function( EditorEvent_Failed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case EditorEvent_Saved() when saved != null:
return saved(_that);case EditorEvent_Incoming() when incoming != null:
return incoming(_that);case EditorEvent_Reloaded() when reloaded != null:
return reloaded(_that);case EditorEvent_Closed() when closed != null:
return closed(_that);case EditorEvent_Failed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( EditorEvent_Saved value)  saved,required TResult Function( EditorEvent_Incoming value)  incoming,required TResult Function( EditorEvent_Reloaded value)  reloaded,required TResult Function( EditorEvent_Closed value)  closed,required TResult Function( EditorEvent_Failed value)  failed,}){
final _that = this;
switch (_that) {
case EditorEvent_Saved():
return saved(_that);case EditorEvent_Incoming():
return incoming(_that);case EditorEvent_Reloaded():
return reloaded(_that);case EditorEvent_Closed():
return closed(_that);case EditorEvent_Failed():
return failed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( EditorEvent_Saved value)?  saved,TResult? Function( EditorEvent_Incoming value)?  incoming,TResult? Function( EditorEvent_Reloaded value)?  reloaded,TResult? Function( EditorEvent_Closed value)?  closed,TResult? Function( EditorEvent_Failed value)?  failed,}){
final _that = this;
switch (_that) {
case EditorEvent_Saved() when saved != null:
return saved(_that);case EditorEvent_Incoming() when incoming != null:
return incoming(_that);case EditorEvent_Reloaded() when reloaded != null:
return reloaded(_that);case EditorEvent_Closed() when closed != null:
return closed(_that);case EditorEvent_Failed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String rev,  int atMs)?  saved,TResult Function( bool deleted)?  incoming,TResult Function( EditorSnapshot snapshot)?  reloaded,TResult Function()?  closed,TResult Function( String message)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case EditorEvent_Saved() when saved != null:
return saved(_that.rev,_that.atMs);case EditorEvent_Incoming() when incoming != null:
return incoming(_that.deleted);case EditorEvent_Reloaded() when reloaded != null:
return reloaded(_that.snapshot);case EditorEvent_Closed() when closed != null:
return closed();case EditorEvent_Failed() when failed != null:
return failed(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String rev,  int atMs)  saved,required TResult Function( bool deleted)  incoming,required TResult Function( EditorSnapshot snapshot)  reloaded,required TResult Function()  closed,required TResult Function( String message)  failed,}) {final _that = this;
switch (_that) {
case EditorEvent_Saved():
return saved(_that.rev,_that.atMs);case EditorEvent_Incoming():
return incoming(_that.deleted);case EditorEvent_Reloaded():
return reloaded(_that.snapshot);case EditorEvent_Closed():
return closed();case EditorEvent_Failed():
return failed(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String rev,  int atMs)?  saved,TResult? Function( bool deleted)?  incoming,TResult? Function( EditorSnapshot snapshot)?  reloaded,TResult? Function()?  closed,TResult? Function( String message)?  failed,}) {final _that = this;
switch (_that) {
case EditorEvent_Saved() when saved != null:
return saved(_that.rev,_that.atMs);case EditorEvent_Incoming() when incoming != null:
return incoming(_that.deleted);case EditorEvent_Reloaded() when reloaded != null:
return reloaded(_that.snapshot);case EditorEvent_Closed() when closed != null:
return closed();case EditorEvent_Failed() when failed != null:
return failed(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class EditorEvent_Saved extends EditorEvent {
  const EditorEvent_Saved({required this.rev, required this.atMs}): super._();
  

 final  String rev;
 final  int atMs;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditorEvent_SavedCopyWith<EditorEvent_Saved> get copyWith => _$EditorEvent_SavedCopyWithImpl<EditorEvent_Saved>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent_Saved&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.atMs, atMs) || other.atMs == atMs));
}


@override
int get hashCode => Object.hash(runtimeType,rev,atMs);

@override
String toString() {
  return 'EditorEvent.saved(rev: $rev, atMs: $atMs)';
}


}

/// @nodoc
abstract mixin class $EditorEvent_SavedCopyWith<$Res> implements $EditorEventCopyWith<$Res> {
  factory $EditorEvent_SavedCopyWith(EditorEvent_Saved value, $Res Function(EditorEvent_Saved) _then) = _$EditorEvent_SavedCopyWithImpl;
@useResult
$Res call({
 String rev, int atMs
});




}
/// @nodoc
class _$EditorEvent_SavedCopyWithImpl<$Res>
    implements $EditorEvent_SavedCopyWith<$Res> {
  _$EditorEvent_SavedCopyWithImpl(this._self, this._then);

  final EditorEvent_Saved _self;
  final $Res Function(EditorEvent_Saved) _then;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? rev = null,Object? atMs = null,}) {
  return _then(EditorEvent_Saved(
rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,atMs: null == atMs ? _self.atMs : atMs // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class EditorEvent_Incoming extends EditorEvent {
  const EditorEvent_Incoming({required this.deleted}): super._();
  

 final  bool deleted;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditorEvent_IncomingCopyWith<EditorEvent_Incoming> get copyWith => _$EditorEvent_IncomingCopyWithImpl<EditorEvent_Incoming>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent_Incoming&&(identical(other.deleted, deleted) || other.deleted == deleted));
}


@override
int get hashCode => Object.hash(runtimeType,deleted);

@override
String toString() {
  return 'EditorEvent.incoming(deleted: $deleted)';
}


}

/// @nodoc
abstract mixin class $EditorEvent_IncomingCopyWith<$Res> implements $EditorEventCopyWith<$Res> {
  factory $EditorEvent_IncomingCopyWith(EditorEvent_Incoming value, $Res Function(EditorEvent_Incoming) _then) = _$EditorEvent_IncomingCopyWithImpl;
@useResult
$Res call({
 bool deleted
});




}
/// @nodoc
class _$EditorEvent_IncomingCopyWithImpl<$Res>
    implements $EditorEvent_IncomingCopyWith<$Res> {
  _$EditorEvent_IncomingCopyWithImpl(this._self, this._then);

  final EditorEvent_Incoming _self;
  final $Res Function(EditorEvent_Incoming) _then;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? deleted = null,}) {
  return _then(EditorEvent_Incoming(
deleted: null == deleted ? _self.deleted : deleted // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class EditorEvent_Reloaded extends EditorEvent {
  const EditorEvent_Reloaded({required this.snapshot}): super._();
  

 final  EditorSnapshot snapshot;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditorEvent_ReloadedCopyWith<EditorEvent_Reloaded> get copyWith => _$EditorEvent_ReloadedCopyWithImpl<EditorEvent_Reloaded>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent_Reloaded&&(identical(other.snapshot, snapshot) || other.snapshot == snapshot));
}


@override
int get hashCode => Object.hash(runtimeType,snapshot);

@override
String toString() {
  return 'EditorEvent.reloaded(snapshot: $snapshot)';
}


}

/// @nodoc
abstract mixin class $EditorEvent_ReloadedCopyWith<$Res> implements $EditorEventCopyWith<$Res> {
  factory $EditorEvent_ReloadedCopyWith(EditorEvent_Reloaded value, $Res Function(EditorEvent_Reloaded) _then) = _$EditorEvent_ReloadedCopyWithImpl;
@useResult
$Res call({
 EditorSnapshot snapshot
});




}
/// @nodoc
class _$EditorEvent_ReloadedCopyWithImpl<$Res>
    implements $EditorEvent_ReloadedCopyWith<$Res> {
  _$EditorEvent_ReloadedCopyWithImpl(this._self, this._then);

  final EditorEvent_Reloaded _self;
  final $Res Function(EditorEvent_Reloaded) _then;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? snapshot = null,}) {
  return _then(EditorEvent_Reloaded(
snapshot: null == snapshot ? _self.snapshot : snapshot // ignore: cast_nullable_to_non_nullable
as EditorSnapshot,
  ));
}


}

/// @nodoc


class EditorEvent_Closed extends EditorEvent {
  const EditorEvent_Closed(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent_Closed);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'EditorEvent.closed()';
}


}




/// @nodoc


class EditorEvent_Failed extends EditorEvent {
  const EditorEvent_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$EditorEvent_FailedCopyWith<EditorEvent_Failed> get copyWith => _$EditorEvent_FailedCopyWithImpl<EditorEvent_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is EditorEvent_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'EditorEvent.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $EditorEvent_FailedCopyWith<$Res> implements $EditorEventCopyWith<$Res> {
  factory $EditorEvent_FailedCopyWith(EditorEvent_Failed value, $Res Function(EditorEvent_Failed) _then) = _$EditorEvent_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$EditorEvent_FailedCopyWithImpl<$Res>
    implements $EditorEvent_FailedCopyWith<$Res> {
  _$EditorEvent_FailedCopyWithImpl(this._self, this._then);

  final EditorEvent_Failed _self;
  final $Res Function(EditorEvent_Failed) _then;

/// Create a copy of EditorEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(EditorEvent_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$KansoError {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError()';
}


}

/// @nodoc
class $KansoErrorCopyWith<$Res>  {
$KansoErrorCopyWith(KansoError _, $Res Function(KansoError) __);
}


/// Adds pattern-matching-related methods to [KansoError].
extension KansoErrorPatterns on KansoError {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( KansoError_NotFound value)?  notFound,TResult Function( KansoError_Locked value)?  locked,TResult Function( KansoError_Conflict value)?  conflict,TResult Function( KansoError_Invalid value)?  invalid,TResult Function( KansoError_Storage value)?  storage,TResult Function( KansoError_Network value)?  network,TResult Function( KansoError_SignedOut value)?  signedOut,TResult Function( KansoError_Auth value)?  auth,TResult Function( KansoError_Internal value)?  internal,required TResult orElse(),}){
final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound(_that);case KansoError_Locked() when locked != null:
return locked(_that);case KansoError_Conflict() when conflict != null:
return conflict(_that);case KansoError_Invalid() when invalid != null:
return invalid(_that);case KansoError_Storage() when storage != null:
return storage(_that);case KansoError_Network() when network != null:
return network(_that);case KansoError_SignedOut() when signedOut != null:
return signedOut(_that);case KansoError_Auth() when auth != null:
return auth(_that);case KansoError_Internal() when internal != null:
return internal(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( KansoError_NotFound value)  notFound,required TResult Function( KansoError_Locked value)  locked,required TResult Function( KansoError_Conflict value)  conflict,required TResult Function( KansoError_Invalid value)  invalid,required TResult Function( KansoError_Storage value)  storage,required TResult Function( KansoError_Network value)  network,required TResult Function( KansoError_SignedOut value)  signedOut,required TResult Function( KansoError_Auth value)  auth,required TResult Function( KansoError_Internal value)  internal,}){
final _that = this;
switch (_that) {
case KansoError_NotFound():
return notFound(_that);case KansoError_Locked():
return locked(_that);case KansoError_Conflict():
return conflict(_that);case KansoError_Invalid():
return invalid(_that);case KansoError_Storage():
return storage(_that);case KansoError_Network():
return network(_that);case KansoError_SignedOut():
return signedOut(_that);case KansoError_Auth():
return auth(_that);case KansoError_Internal():
return internal(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( KansoError_NotFound value)?  notFound,TResult? Function( KansoError_Locked value)?  locked,TResult? Function( KansoError_Conflict value)?  conflict,TResult? Function( KansoError_Invalid value)?  invalid,TResult? Function( KansoError_Storage value)?  storage,TResult? Function( KansoError_Network value)?  network,TResult? Function( KansoError_SignedOut value)?  signedOut,TResult? Function( KansoError_Auth value)?  auth,TResult? Function( KansoError_Internal value)?  internal,}){
final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound(_that);case KansoError_Locked() when locked != null:
return locked(_that);case KansoError_Conflict() when conflict != null:
return conflict(_that);case KansoError_Invalid() when invalid != null:
return invalid(_that);case KansoError_Storage() when storage != null:
return storage(_that);case KansoError_Network() when network != null:
return network(_that);case KansoError_SignedOut() when signedOut != null:
return signedOut(_that);case KansoError_Auth() when auth != null:
return auth(_that);case KansoError_Internal() when internal != null:
return internal(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  notFound,TResult Function()?  locked,TResult Function( String currentRev)?  conflict,TResult Function( String message)?  invalid,TResult Function( String message)?  storage,TResult Function( String message)?  network,TResult Function()?  signedOut,TResult Function( String message)?  auth,TResult Function( String message)?  internal,required TResult orElse(),}) {final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound();case KansoError_Locked() when locked != null:
return locked();case KansoError_Conflict() when conflict != null:
return conflict(_that.currentRev);case KansoError_Invalid() when invalid != null:
return invalid(_that.message);case KansoError_Storage() when storage != null:
return storage(_that.message);case KansoError_Network() when network != null:
return network(_that.message);case KansoError_SignedOut() when signedOut != null:
return signedOut();case KansoError_Auth() when auth != null:
return auth(_that.message);case KansoError_Internal() when internal != null:
return internal(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  notFound,required TResult Function()  locked,required TResult Function( String currentRev)  conflict,required TResult Function( String message)  invalid,required TResult Function( String message)  storage,required TResult Function( String message)  network,required TResult Function()  signedOut,required TResult Function( String message)  auth,required TResult Function( String message)  internal,}) {final _that = this;
switch (_that) {
case KansoError_NotFound():
return notFound();case KansoError_Locked():
return locked();case KansoError_Conflict():
return conflict(_that.currentRev);case KansoError_Invalid():
return invalid(_that.message);case KansoError_Storage():
return storage(_that.message);case KansoError_Network():
return network(_that.message);case KansoError_SignedOut():
return signedOut();case KansoError_Auth():
return auth(_that.message);case KansoError_Internal():
return internal(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  notFound,TResult? Function()?  locked,TResult? Function( String currentRev)?  conflict,TResult? Function( String message)?  invalid,TResult? Function( String message)?  storage,TResult? Function( String message)?  network,TResult? Function()?  signedOut,TResult? Function( String message)?  auth,TResult? Function( String message)?  internal,}) {final _that = this;
switch (_that) {
case KansoError_NotFound() when notFound != null:
return notFound();case KansoError_Locked() when locked != null:
return locked();case KansoError_Conflict() when conflict != null:
return conflict(_that.currentRev);case KansoError_Invalid() when invalid != null:
return invalid(_that.message);case KansoError_Storage() when storage != null:
return storage(_that.message);case KansoError_Network() when network != null:
return network(_that.message);case KansoError_SignedOut() when signedOut != null:
return signedOut();case KansoError_Auth() when auth != null:
return auth(_that.message);case KansoError_Internal() when internal != null:
return internal(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class KansoError_NotFound extends KansoError {
  const KansoError_NotFound(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_NotFound);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError.notFound()';
}


}




/// @nodoc


class KansoError_Locked extends KansoError {
  const KansoError_Locked(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Locked);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError.locked()';
}


}




/// @nodoc


class KansoError_Conflict extends KansoError {
  const KansoError_Conflict({required this.currentRev}): super._();
  

 final  String currentRev;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_ConflictCopyWith<KansoError_Conflict> get copyWith => _$KansoError_ConflictCopyWithImpl<KansoError_Conflict>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Conflict&&(identical(other.currentRev, currentRev) || other.currentRev == currentRev));
}


@override
int get hashCode => Object.hash(runtimeType,currentRev);

@override
String toString() {
  return 'KansoError.conflict(currentRev: $currentRev)';
}


}

/// @nodoc
abstract mixin class $KansoError_ConflictCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_ConflictCopyWith(KansoError_Conflict value, $Res Function(KansoError_Conflict) _then) = _$KansoError_ConflictCopyWithImpl;
@useResult
$Res call({
 String currentRev
});




}
/// @nodoc
class _$KansoError_ConflictCopyWithImpl<$Res>
    implements $KansoError_ConflictCopyWith<$Res> {
  _$KansoError_ConflictCopyWithImpl(this._self, this._then);

  final KansoError_Conflict _self;
  final $Res Function(KansoError_Conflict) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? currentRev = null,}) {
  return _then(KansoError_Conflict(
currentRev: null == currentRev ? _self.currentRev : currentRev // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Invalid extends KansoError {
  const KansoError_Invalid({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_InvalidCopyWith<KansoError_Invalid> get copyWith => _$KansoError_InvalidCopyWithImpl<KansoError_Invalid>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Invalid&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.invalid(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_InvalidCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_InvalidCopyWith(KansoError_Invalid value, $Res Function(KansoError_Invalid) _then) = _$KansoError_InvalidCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_InvalidCopyWithImpl<$Res>
    implements $KansoError_InvalidCopyWith<$Res> {
  _$KansoError_InvalidCopyWithImpl(this._self, this._then);

  final KansoError_Invalid _self;
  final $Res Function(KansoError_Invalid) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Invalid(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Storage extends KansoError {
  const KansoError_Storage({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_StorageCopyWith<KansoError_Storage> get copyWith => _$KansoError_StorageCopyWithImpl<KansoError_Storage>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Storage&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.storage(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_StorageCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_StorageCopyWith(KansoError_Storage value, $Res Function(KansoError_Storage) _then) = _$KansoError_StorageCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_StorageCopyWithImpl<$Res>
    implements $KansoError_StorageCopyWith<$Res> {
  _$KansoError_StorageCopyWithImpl(this._self, this._then);

  final KansoError_Storage _self;
  final $Res Function(KansoError_Storage) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Storage(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Network extends KansoError {
  const KansoError_Network({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_NetworkCopyWith<KansoError_Network> get copyWith => _$KansoError_NetworkCopyWithImpl<KansoError_Network>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Network&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.network(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_NetworkCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_NetworkCopyWith(KansoError_Network value, $Res Function(KansoError_Network) _then) = _$KansoError_NetworkCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_NetworkCopyWithImpl<$Res>
    implements $KansoError_NetworkCopyWith<$Res> {
  _$KansoError_NetworkCopyWithImpl(this._self, this._then);

  final KansoError_Network _self;
  final $Res Function(KansoError_Network) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Network(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_SignedOut extends KansoError {
  const KansoError_SignedOut(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_SignedOut);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'KansoError.signedOut()';
}


}




/// @nodoc


class KansoError_Auth extends KansoError {
  const KansoError_Auth({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_AuthCopyWith<KansoError_Auth> get copyWith => _$KansoError_AuthCopyWithImpl<KansoError_Auth>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Auth&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.auth(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_AuthCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_AuthCopyWith(KansoError_Auth value, $Res Function(KansoError_Auth) _then) = _$KansoError_AuthCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_AuthCopyWithImpl<$Res>
    implements $KansoError_AuthCopyWith<$Res> {
  _$KansoError_AuthCopyWithImpl(this._self, this._then);

  final KansoError_Auth _self;
  final $Res Function(KansoError_Auth) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Auth(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class KansoError_Internal extends KansoError {
  const KansoError_Internal({required this.message}): super._();
  

 final  String message;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$KansoError_InternalCopyWith<KansoError_Internal> get copyWith => _$KansoError_InternalCopyWithImpl<KansoError_Internal>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is KansoError_Internal&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'KansoError.internal(message: $message)';
}


}

/// @nodoc
abstract mixin class $KansoError_InternalCopyWith<$Res> implements $KansoErrorCopyWith<$Res> {
  factory $KansoError_InternalCopyWith(KansoError_Internal value, $Res Function(KansoError_Internal) _then) = _$KansoError_InternalCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$KansoError_InternalCopyWithImpl<$Res>
    implements $KansoError_InternalCopyWith<$Res> {
  _$KansoError_InternalCopyWithImpl(this._self, this._then);

  final KansoError_Internal _self;
  final $Res Function(KansoError_Internal) _then;

/// Create a copy of KansoError
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(KansoError_Internal(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$LinkTarget {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkTarget);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LinkTarget()';
}


}

/// @nodoc
class $LinkTargetCopyWith<$Res>  {
$LinkTargetCopyWith(LinkTarget _, $Res Function(LinkTarget) __);
}


/// Adds pattern-matching-related methods to [LinkTarget].
extension LinkTargetPatterns on LinkTarget {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LinkTarget_Note value)?  note,TResult Function( LinkTarget_NewNote value)?  newNote,TResult Function( LinkTarget_Web value)?  web,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LinkTarget_Note() when note != null:
return note(_that);case LinkTarget_NewNote() when newNote != null:
return newNote(_that);case LinkTarget_Web() when web != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LinkTarget_Note value)  note,required TResult Function( LinkTarget_NewNote value)  newNote,required TResult Function( LinkTarget_Web value)  web,}){
final _that = this;
switch (_that) {
case LinkTarget_Note():
return note(_that);case LinkTarget_NewNote():
return newNote(_that);case LinkTarget_Web():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LinkTarget_Note value)?  note,TResult? Function( LinkTarget_NewNote value)?  newNote,TResult? Function( LinkTarget_Web value)?  web,}){
final _that = this;
switch (_that) {
case LinkTarget_Note() when note != null:
return note(_that);case LinkTarget_NewNote() when newNote != null:
return newNote(_that);case LinkTarget_Web() when web != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String id)?  note,TResult Function( String title)?  newNote,TResult Function( String url)?  web,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LinkTarget_Note() when note != null:
return note(_that.id);case LinkTarget_NewNote() when newNote != null:
return newNote(_that.title);case LinkTarget_Web() when web != null:
return web(_that.url);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String id)  note,required TResult Function( String title)  newNote,required TResult Function( String url)  web,}) {final _that = this;
switch (_that) {
case LinkTarget_Note():
return note(_that.id);case LinkTarget_NewNote():
return newNote(_that.title);case LinkTarget_Web():
return web(_that.url);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String id)?  note,TResult? Function( String title)?  newNote,TResult? Function( String url)?  web,}) {final _that = this;
switch (_that) {
case LinkTarget_Note() when note != null:
return note(_that.id);case LinkTarget_NewNote() when newNote != null:
return newNote(_that.title);case LinkTarget_Web() when web != null:
return web(_that.url);case _:
  return null;

}
}

}

/// @nodoc


class LinkTarget_Note extends LinkTarget {
  const LinkTarget_Note({required this.id}): super._();
  

 final  String id;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkTarget_NoteCopyWith<LinkTarget_Note> get copyWith => _$LinkTarget_NoteCopyWithImpl<LinkTarget_Note>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkTarget_Note&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'LinkTarget.note(id: $id)';
}


}

/// @nodoc
abstract mixin class $LinkTarget_NoteCopyWith<$Res> implements $LinkTargetCopyWith<$Res> {
  factory $LinkTarget_NoteCopyWith(LinkTarget_Note value, $Res Function(LinkTarget_Note) _then) = _$LinkTarget_NoteCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$LinkTarget_NoteCopyWithImpl<$Res>
    implements $LinkTarget_NoteCopyWith<$Res> {
  _$LinkTarget_NoteCopyWithImpl(this._self, this._then);

  final LinkTarget_Note _self;
  final $Res Function(LinkTarget_Note) _then;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(LinkTarget_Note(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class LinkTarget_NewNote extends LinkTarget {
  const LinkTarget_NewNote({required this.title}): super._();
  

 final  String title;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkTarget_NewNoteCopyWith<LinkTarget_NewNote> get copyWith => _$LinkTarget_NewNoteCopyWithImpl<LinkTarget_NewNote>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkTarget_NewNote&&(identical(other.title, title) || other.title == title));
}


@override
int get hashCode => Object.hash(runtimeType,title);

@override
String toString() {
  return 'LinkTarget.newNote(title: $title)';
}


}

/// @nodoc
abstract mixin class $LinkTarget_NewNoteCopyWith<$Res> implements $LinkTargetCopyWith<$Res> {
  factory $LinkTarget_NewNoteCopyWith(LinkTarget_NewNote value, $Res Function(LinkTarget_NewNote) _then) = _$LinkTarget_NewNoteCopyWithImpl;
@useResult
$Res call({
 String title
});




}
/// @nodoc
class _$LinkTarget_NewNoteCopyWithImpl<$Res>
    implements $LinkTarget_NewNoteCopyWith<$Res> {
  _$LinkTarget_NewNoteCopyWithImpl(this._self, this._then);

  final LinkTarget_NewNote _self;
  final $Res Function(LinkTarget_NewNote) _then;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? title = null,}) {
  return _then(LinkTarget_NewNote(
title: null == title ? _self.title : title // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class LinkTarget_Web extends LinkTarget {
  const LinkTarget_Web({required this.url}): super._();
  

 final  String url;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LinkTarget_WebCopyWith<LinkTarget_Web> get copyWith => _$LinkTarget_WebCopyWithImpl<LinkTarget_Web>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LinkTarget_Web&&(identical(other.url, url) || other.url == url));
}


@override
int get hashCode => Object.hash(runtimeType,url);

@override
String toString() {
  return 'LinkTarget.web(url: $url)';
}


}

/// @nodoc
abstract mixin class $LinkTarget_WebCopyWith<$Res> implements $LinkTargetCopyWith<$Res> {
  factory $LinkTarget_WebCopyWith(LinkTarget_Web value, $Res Function(LinkTarget_Web) _then) = _$LinkTarget_WebCopyWithImpl;
@useResult
$Res call({
 String url
});




}
/// @nodoc
class _$LinkTarget_WebCopyWithImpl<$Res>
    implements $LinkTarget_WebCopyWith<$Res> {
  _$LinkTarget_WebCopyWithImpl(this._self, this._then);

  final LinkTarget_Web _self;
  final $Res Function(LinkTarget_Web) _then;

/// Create a copy of LinkTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? url = null,}) {
  return _then(LinkTarget_Web(
url: null == url ? _self.url : url // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$ListPlace {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace()';
}


}

/// @nodoc
class $ListPlaceCopyWith<$Res>  {
$ListPlaceCopyWith(ListPlace _, $Res Function(ListPlace) __);
}


/// Adds pattern-matching-related methods to [ListPlace].
extension ListPlacePatterns on ListPlace {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( ListPlace_All value)?  all,TResult Function( ListPlace_Pinned value)?  pinned,TResult Function( ListPlace_Recent value)?  recent,TResult Function( ListPlace_Notebook value)?  notebook,TResult Function( ListPlace_Tag value)?  tag,TResult Function( ListPlace_Trash value)?  trash,required TResult orElse(),}){
final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all(_that);case ListPlace_Pinned() when pinned != null:
return pinned(_that);case ListPlace_Recent() when recent != null:
return recent(_that);case ListPlace_Notebook() when notebook != null:
return notebook(_that);case ListPlace_Tag() when tag != null:
return tag(_that);case ListPlace_Trash() when trash != null:
return trash(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( ListPlace_All value)  all,required TResult Function( ListPlace_Pinned value)  pinned,required TResult Function( ListPlace_Recent value)  recent,required TResult Function( ListPlace_Notebook value)  notebook,required TResult Function( ListPlace_Tag value)  tag,required TResult Function( ListPlace_Trash value)  trash,}){
final _that = this;
switch (_that) {
case ListPlace_All():
return all(_that);case ListPlace_Pinned():
return pinned(_that);case ListPlace_Recent():
return recent(_that);case ListPlace_Notebook():
return notebook(_that);case ListPlace_Tag():
return tag(_that);case ListPlace_Trash():
return trash(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( ListPlace_All value)?  all,TResult? Function( ListPlace_Pinned value)?  pinned,TResult? Function( ListPlace_Recent value)?  recent,TResult? Function( ListPlace_Notebook value)?  notebook,TResult? Function( ListPlace_Tag value)?  tag,TResult? Function( ListPlace_Trash value)?  trash,}){
final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all(_that);case ListPlace_Pinned() when pinned != null:
return pinned(_that);case ListPlace_Recent() when recent != null:
return recent(_that);case ListPlace_Notebook() when notebook != null:
return notebook(_that);case ListPlace_Tag() when tag != null:
return tag(_that);case ListPlace_Trash() when trash != null:
return trash(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  all,TResult Function()?  pinned,TResult Function()?  recent,TResult Function( String id)?  notebook,TResult Function( String id)?  tag,TResult Function()?  trash,required TResult orElse(),}) {final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all();case ListPlace_Pinned() when pinned != null:
return pinned();case ListPlace_Recent() when recent != null:
return recent();case ListPlace_Notebook() when notebook != null:
return notebook(_that.id);case ListPlace_Tag() when tag != null:
return tag(_that.id);case ListPlace_Trash() when trash != null:
return trash();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  all,required TResult Function()  pinned,required TResult Function()  recent,required TResult Function( String id)  notebook,required TResult Function( String id)  tag,required TResult Function()  trash,}) {final _that = this;
switch (_that) {
case ListPlace_All():
return all();case ListPlace_Pinned():
return pinned();case ListPlace_Recent():
return recent();case ListPlace_Notebook():
return notebook(_that.id);case ListPlace_Tag():
return tag(_that.id);case ListPlace_Trash():
return trash();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  all,TResult? Function()?  pinned,TResult? Function()?  recent,TResult? Function( String id)?  notebook,TResult? Function( String id)?  tag,TResult? Function()?  trash,}) {final _that = this;
switch (_that) {
case ListPlace_All() when all != null:
return all();case ListPlace_Pinned() when pinned != null:
return pinned();case ListPlace_Recent() when recent != null:
return recent();case ListPlace_Notebook() when notebook != null:
return notebook(_that.id);case ListPlace_Tag() when tag != null:
return tag(_that.id);case ListPlace_Trash() when trash != null:
return trash();case _:
  return null;

}
}

}

/// @nodoc


class ListPlace_All extends ListPlace {
  const ListPlace_All(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_All);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.all()';
}


}




/// @nodoc


class ListPlace_Pinned extends ListPlace {
  const ListPlace_Pinned(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Pinned);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.pinned()';
}


}




/// @nodoc


class ListPlace_Recent extends ListPlace {
  const ListPlace_Recent(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Recent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.recent()';
}


}




/// @nodoc


class ListPlace_Notebook extends ListPlace {
  const ListPlace_Notebook({required this.id}): super._();
  

 final  String id;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListPlace_NotebookCopyWith<ListPlace_Notebook> get copyWith => _$ListPlace_NotebookCopyWithImpl<ListPlace_Notebook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Notebook&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'ListPlace.notebook(id: $id)';
}


}

/// @nodoc
abstract mixin class $ListPlace_NotebookCopyWith<$Res> implements $ListPlaceCopyWith<$Res> {
  factory $ListPlace_NotebookCopyWith(ListPlace_Notebook value, $Res Function(ListPlace_Notebook) _then) = _$ListPlace_NotebookCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$ListPlace_NotebookCopyWithImpl<$Res>
    implements $ListPlace_NotebookCopyWith<$Res> {
  _$ListPlace_NotebookCopyWithImpl(this._self, this._then);

  final ListPlace_Notebook _self;
  final $Res Function(ListPlace_Notebook) _then;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(ListPlace_Notebook(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ListPlace_Tag extends ListPlace {
  const ListPlace_Tag({required this.id}): super._();
  

 final  String id;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$ListPlace_TagCopyWith<ListPlace_Tag> get copyWith => _$ListPlace_TagCopyWithImpl<ListPlace_Tag>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Tag&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'ListPlace.tag(id: $id)';
}


}

/// @nodoc
abstract mixin class $ListPlace_TagCopyWith<$Res> implements $ListPlaceCopyWith<$Res> {
  factory $ListPlace_TagCopyWith(ListPlace_Tag value, $Res Function(ListPlace_Tag) _then) = _$ListPlace_TagCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$ListPlace_TagCopyWithImpl<$Res>
    implements $ListPlace_TagCopyWith<$Res> {
  _$ListPlace_TagCopyWithImpl(this._self, this._then);

  final ListPlace_Tag _self;
  final $Res Function(ListPlace_Tag) _then;

/// Create a copy of ListPlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(ListPlace_Tag(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class ListPlace_Trash extends ListPlace {
  const ListPlace_Trash(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is ListPlace_Trash);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'ListPlace.trash()';
}


}




/// @nodoc
mixin _$LockEvent {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockEvent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockEvent()';
}


}

/// @nodoc
class $LockEventCopyWith<$Res>  {
$LockEventCopyWith(LockEvent _, $Res Function(LockEvent) __);
}


/// Adds pattern-matching-related methods to [LockEvent].
extension LockEventPatterns on LockEvent {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LockEvent_Unlocked value)?  unlocked,TResult Function( LockEvent_Locked value)?  locked,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked(_that);case LockEvent_Locked() when locked != null:
return locked(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LockEvent_Unlocked value)  unlocked,required TResult Function( LockEvent_Locked value)  locked,}){
final _that = this;
switch (_that) {
case LockEvent_Unlocked():
return unlocked(_that);case LockEvent_Locked():
return locked(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LockEvent_Unlocked value)?  unlocked,TResult? Function( LockEvent_Locked value)?  locked,}){
final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked(_that);case LockEvent_Locked() when locked != null:
return locked(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  unlocked,TResult Function( LockReason reason)?  locked,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked();case LockEvent_Locked() when locked != null:
return locked(_that.reason);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  unlocked,required TResult Function( LockReason reason)  locked,}) {final _that = this;
switch (_that) {
case LockEvent_Unlocked():
return unlocked();case LockEvent_Locked():
return locked(_that.reason);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  unlocked,TResult? Function( LockReason reason)?  locked,}) {final _that = this;
switch (_that) {
case LockEvent_Unlocked() when unlocked != null:
return unlocked();case LockEvent_Locked() when locked != null:
return locked(_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class LockEvent_Unlocked extends LockEvent {
  const LockEvent_Unlocked(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockEvent_Unlocked);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockEvent.unlocked()';
}


}




/// @nodoc


class LockEvent_Locked extends LockEvent {
  const LockEvent_Locked({required this.reason}): super._();
  

 final  LockReason reason;

/// Create a copy of LockEvent
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockEvent_LockedCopyWith<LockEvent_Locked> get copyWith => _$LockEvent_LockedCopyWithImpl<LockEvent_Locked>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockEvent_Locked&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'LockEvent.locked(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $LockEvent_LockedCopyWith<$Res> implements $LockEventCopyWith<$Res> {
  factory $LockEvent_LockedCopyWith(LockEvent_Locked value, $Res Function(LockEvent_Locked) _then) = _$LockEvent_LockedCopyWithImpl;
@useResult
$Res call({
 LockReason reason
});




}
/// @nodoc
class _$LockEvent_LockedCopyWithImpl<$Res>
    implements $LockEvent_LockedCopyWith<$Res> {
  _$LockEvent_LockedCopyWithImpl(this._self, this._then);

  final LockEvent_Locked _self;
  final $Res Function(LockEvent_Locked) _then;

/// Create a copy of LockEvent
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = null,}) {
  return _then(LockEvent_Locked(
reason: null == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as LockReason,
  ));
}


}

/// @nodoc
mixin _$LockState {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockState()';
}


}

/// @nodoc
class $LockStateCopyWith<$Res>  {
$LockStateCopyWith(LockState _, $Res Function(LockState) __);
}


/// Adds pattern-matching-related methods to [LockState].
extension LockStatePatterns on LockState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( LockState_Plain value)?  plain,TResult Function( LockState_Open value)?  open,TResult Function( LockState_Closed value)?  closed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain(_that);case LockState_Open() when open != null:
return open(_that);case LockState_Closed() when closed != null:
return closed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( LockState_Plain value)  plain,required TResult Function( LockState_Open value)  open,required TResult Function( LockState_Closed value)  closed,}){
final _that = this;
switch (_that) {
case LockState_Plain():
return plain(_that);case LockState_Open():
return open(_that);case LockState_Closed():
return closed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( LockState_Plain value)?  plain,TResult? Function( LockState_Open value)?  open,TResult? Function( LockState_Closed value)?  closed,}){
final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain(_that);case LockState_Open() when open != null:
return open(_that);case LockState_Closed() when closed != null:
return closed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  plain,TResult Function()?  open,TResult Function( String? reason)?  closed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain();case LockState_Open() when open != null:
return open();case LockState_Closed() when closed != null:
return closed(_that.reason);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  plain,required TResult Function()  open,required TResult Function( String? reason)  closed,}) {final _that = this;
switch (_that) {
case LockState_Plain():
return plain();case LockState_Open():
return open();case LockState_Closed():
return closed(_that.reason);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  plain,TResult? Function()?  open,TResult? Function( String? reason)?  closed,}) {final _that = this;
switch (_that) {
case LockState_Plain() when plain != null:
return plain();case LockState_Open() when open != null:
return open();case LockState_Closed() when closed != null:
return closed(_that.reason);case _:
  return null;

}
}

}

/// @nodoc


class LockState_Plain extends LockState {
  const LockState_Plain(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState_Plain);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockState.plain()';
}


}




/// @nodoc


class LockState_Open extends LockState {
  const LockState_Open(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState_Open);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'LockState.open()';
}


}




/// @nodoc


class LockState_Closed extends LockState {
  const LockState_Closed({this.reason}): super._();
  

 final  String? reason;

/// Create a copy of LockState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$LockState_ClosedCopyWith<LockState_Closed> get copyWith => _$LockState_ClosedCopyWithImpl<LockState_Closed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is LockState_Closed&&(identical(other.reason, reason) || other.reason == reason));
}


@override
int get hashCode => Object.hash(runtimeType,reason);

@override
String toString() {
  return 'LockState.closed(reason: $reason)';
}


}

/// @nodoc
abstract mixin class $LockState_ClosedCopyWith<$Res> implements $LockStateCopyWith<$Res> {
  factory $LockState_ClosedCopyWith(LockState_Closed value, $Res Function(LockState_Closed) _then) = _$LockState_ClosedCopyWithImpl;
@useResult
$Res call({
 String? reason
});




}
/// @nodoc
class _$LockState_ClosedCopyWithImpl<$Res>
    implements $LockState_ClosedCopyWith<$Res> {
  _$LockState_ClosedCopyWithImpl(this._self, this._then);

  final LockState_Closed _self;
  final $Res Function(LockState_Closed) _then;

/// Create a copy of LockState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? reason = freezed,}) {
  return _then(LockState_Closed(
reason: freezed == reason ? _self.reason : reason // ignore: cast_nullable_to_non_nullable
as String?,
  ));
}


}

/// @nodoc
mixin _$NoteChange {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange()';
}


}

/// @nodoc
class $NoteChangeCopyWith<$Res>  {
$NoteChangeCopyWith(NoteChange _, $Res Function(NoteChange) __);
}


/// Adds pattern-matching-related methods to [NoteChange].
extension NoteChangePatterns on NoteChange {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( NoteChange_TogglePin value)?  togglePin,TResult Function( NoteChange_Move value)?  move,TResult Function( NoteChange_Trash value)?  trash,required TResult orElse(),}){
final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin(_that);case NoteChange_Move() when move != null:
return move(_that);case NoteChange_Trash() when trash != null:
return trash(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( NoteChange_TogglePin value)  togglePin,required TResult Function( NoteChange_Move value)  move,required TResult Function( NoteChange_Trash value)  trash,}){
final _that = this;
switch (_that) {
case NoteChange_TogglePin():
return togglePin(_that);case NoteChange_Move():
return move(_that);case NoteChange_Trash():
return trash(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( NoteChange_TogglePin value)?  togglePin,TResult? Function( NoteChange_Move value)?  move,TResult? Function( NoteChange_Trash value)?  trash,}){
final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin(_that);case NoteChange_Move() when move != null:
return move(_that);case NoteChange_Trash() when trash != null:
return trash(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  togglePin,TResult Function( String notebookId)?  move,TResult Function()?  trash,required TResult orElse(),}) {final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin();case NoteChange_Move() when move != null:
return move(_that.notebookId);case NoteChange_Trash() when trash != null:
return trash();case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  togglePin,required TResult Function( String notebookId)  move,required TResult Function()  trash,}) {final _that = this;
switch (_that) {
case NoteChange_TogglePin():
return togglePin();case NoteChange_Move():
return move(_that.notebookId);case NoteChange_Trash():
return trash();}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  togglePin,TResult? Function( String notebookId)?  move,TResult? Function()?  trash,}) {final _that = this;
switch (_that) {
case NoteChange_TogglePin() when togglePin != null:
return togglePin();case NoteChange_Move() when move != null:
return move(_that.notebookId);case NoteChange_Trash() when trash != null:
return trash();case _:
  return null;

}
}

}

/// @nodoc


class NoteChange_TogglePin extends NoteChange {
  const NoteChange_TogglePin(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_TogglePin);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange.togglePin()';
}


}




/// @nodoc


class NoteChange_Move extends NoteChange {
  const NoteChange_Move({required this.notebookId}): super._();
  

 final  String notebookId;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$NoteChange_MoveCopyWith<NoteChange_Move> get copyWith => _$NoteChange_MoveCopyWithImpl<NoteChange_Move>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_Move&&(identical(other.notebookId, notebookId) || other.notebookId == notebookId));
}


@override
int get hashCode => Object.hash(runtimeType,notebookId);

@override
String toString() {
  return 'NoteChange.move(notebookId: $notebookId)';
}


}

/// @nodoc
abstract mixin class $NoteChange_MoveCopyWith<$Res> implements $NoteChangeCopyWith<$Res> {
  factory $NoteChange_MoveCopyWith(NoteChange_Move value, $Res Function(NoteChange_Move) _then) = _$NoteChange_MoveCopyWithImpl;
@useResult
$Res call({
 String notebookId
});




}
/// @nodoc
class _$NoteChange_MoveCopyWithImpl<$Res>
    implements $NoteChange_MoveCopyWith<$Res> {
  _$NoteChange_MoveCopyWithImpl(this._self, this._then);

  final NoteChange_Move _self;
  final $Res Function(NoteChange_Move) _then;

/// Create a copy of NoteChange
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? notebookId = null,}) {
  return _then(NoteChange_Move(
notebookId: null == notebookId ? _self.notebookId : notebookId // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class NoteChange_Trash extends NoteChange {
  const NoteChange_Trash(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is NoteChange_Trash);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'NoteChange.trash()';
}


}




/// @nodoc
mixin _$SaveState {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SaveState);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SaveState()';
}


}

/// @nodoc
class $SaveStateCopyWith<$Res>  {
$SaveStateCopyWith(SaveState _, $Res Function(SaveState) __);
}


/// Adds pattern-matching-related methods to [SaveState].
extension SaveStatePatterns on SaveState {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SaveState_Saved value)?  saved,TResult Function( SaveState_Pending value)?  pending,TResult Function( SaveState_Conflict value)?  conflict,TResult Function( SaveState_Failed value)?  failed,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SaveState_Saved() when saved != null:
return saved(_that);case SaveState_Pending() when pending != null:
return pending(_that);case SaveState_Conflict() when conflict != null:
return conflict(_that);case SaveState_Failed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SaveState_Saved value)  saved,required TResult Function( SaveState_Pending value)  pending,required TResult Function( SaveState_Conflict value)  conflict,required TResult Function( SaveState_Failed value)  failed,}){
final _that = this;
switch (_that) {
case SaveState_Saved():
return saved(_that);case SaveState_Pending():
return pending(_that);case SaveState_Conflict():
return conflict(_that);case SaveState_Failed():
return failed(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SaveState_Saved value)?  saved,TResult? Function( SaveState_Pending value)?  pending,TResult? Function( SaveState_Conflict value)?  conflict,TResult? Function( SaveState_Failed value)?  failed,}){
final _that = this;
switch (_that) {
case SaveState_Saved() when saved != null:
return saved(_that);case SaveState_Pending() when pending != null:
return pending(_that);case SaveState_Conflict() when conflict != null:
return conflict(_that);case SaveState_Failed() when failed != null:
return failed(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String rev,  int atMs)?  saved,TResult Function()?  pending,TResult Function( bool deleted)?  conflict,TResult Function( String message)?  failed,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SaveState_Saved() when saved != null:
return saved(_that.rev,_that.atMs);case SaveState_Pending() when pending != null:
return pending();case SaveState_Conflict() when conflict != null:
return conflict(_that.deleted);case SaveState_Failed() when failed != null:
return failed(_that.message);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String rev,  int atMs)  saved,required TResult Function()  pending,required TResult Function( bool deleted)  conflict,required TResult Function( String message)  failed,}) {final _that = this;
switch (_that) {
case SaveState_Saved():
return saved(_that.rev,_that.atMs);case SaveState_Pending():
return pending();case SaveState_Conflict():
return conflict(_that.deleted);case SaveState_Failed():
return failed(_that.message);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String rev,  int atMs)?  saved,TResult? Function()?  pending,TResult? Function( bool deleted)?  conflict,TResult? Function( String message)?  failed,}) {final _that = this;
switch (_that) {
case SaveState_Saved() when saved != null:
return saved(_that.rev,_that.atMs);case SaveState_Pending() when pending != null:
return pending();case SaveState_Conflict() when conflict != null:
return conflict(_that.deleted);case SaveState_Failed() when failed != null:
return failed(_that.message);case _:
  return null;

}
}

}

/// @nodoc


class SaveState_Saved extends SaveState {
  const SaveState_Saved({required this.rev, required this.atMs}): super._();
  

 final  String rev;
 final  int atMs;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SaveState_SavedCopyWith<SaveState_Saved> get copyWith => _$SaveState_SavedCopyWithImpl<SaveState_Saved>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SaveState_Saved&&(identical(other.rev, rev) || other.rev == rev)&&(identical(other.atMs, atMs) || other.atMs == atMs));
}


@override
int get hashCode => Object.hash(runtimeType,rev,atMs);

@override
String toString() {
  return 'SaveState.saved(rev: $rev, atMs: $atMs)';
}


}

/// @nodoc
abstract mixin class $SaveState_SavedCopyWith<$Res> implements $SaveStateCopyWith<$Res> {
  factory $SaveState_SavedCopyWith(SaveState_Saved value, $Res Function(SaveState_Saved) _then) = _$SaveState_SavedCopyWithImpl;
@useResult
$Res call({
 String rev, int atMs
});




}
/// @nodoc
class _$SaveState_SavedCopyWithImpl<$Res>
    implements $SaveState_SavedCopyWith<$Res> {
  _$SaveState_SavedCopyWithImpl(this._self, this._then);

  final SaveState_Saved _self;
  final $Res Function(SaveState_Saved) _then;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? rev = null,Object? atMs = null,}) {
  return _then(SaveState_Saved(
rev: null == rev ? _self.rev : rev // ignore: cast_nullable_to_non_nullable
as String,atMs: null == atMs ? _self.atMs : atMs // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

/// @nodoc


class SaveState_Pending extends SaveState {
  const SaveState_Pending(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SaveState_Pending);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SaveState.pending()';
}


}




/// @nodoc


class SaveState_Conflict extends SaveState {
  const SaveState_Conflict({required this.deleted}): super._();
  

 final  bool deleted;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SaveState_ConflictCopyWith<SaveState_Conflict> get copyWith => _$SaveState_ConflictCopyWithImpl<SaveState_Conflict>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SaveState_Conflict&&(identical(other.deleted, deleted) || other.deleted == deleted));
}


@override
int get hashCode => Object.hash(runtimeType,deleted);

@override
String toString() {
  return 'SaveState.conflict(deleted: $deleted)';
}


}

/// @nodoc
abstract mixin class $SaveState_ConflictCopyWith<$Res> implements $SaveStateCopyWith<$Res> {
  factory $SaveState_ConflictCopyWith(SaveState_Conflict value, $Res Function(SaveState_Conflict) _then) = _$SaveState_ConflictCopyWithImpl;
@useResult
$Res call({
 bool deleted
});




}
/// @nodoc
class _$SaveState_ConflictCopyWithImpl<$Res>
    implements $SaveState_ConflictCopyWith<$Res> {
  _$SaveState_ConflictCopyWithImpl(this._self, this._then);

  final SaveState_Conflict _self;
  final $Res Function(SaveState_Conflict) _then;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? deleted = null,}) {
  return _then(SaveState_Conflict(
deleted: null == deleted ? _self.deleted : deleted // ignore: cast_nullable_to_non_nullable
as bool,
  ));
}


}

/// @nodoc


class SaveState_Failed extends SaveState {
  const SaveState_Failed({required this.message}): super._();
  

 final  String message;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SaveState_FailedCopyWith<SaveState_Failed> get copyWith => _$SaveState_FailedCopyWithImpl<SaveState_Failed>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SaveState_Failed&&(identical(other.message, message) || other.message == message));
}


@override
int get hashCode => Object.hash(runtimeType,message);

@override
String toString() {
  return 'SaveState.failed(message: $message)';
}


}

/// @nodoc
abstract mixin class $SaveState_FailedCopyWith<$Res> implements $SaveStateCopyWith<$Res> {
  factory $SaveState_FailedCopyWith(SaveState_Failed value, $Res Function(SaveState_Failed) _then) = _$SaveState_FailedCopyWithImpl;
@useResult
$Res call({
 String message
});




}
/// @nodoc
class _$SaveState_FailedCopyWithImpl<$Res>
    implements $SaveState_FailedCopyWith<$Res> {
  _$SaveState_FailedCopyWithImpl(this._self, this._then);

  final SaveState_Failed _self;
  final $Res Function(SaveState_Failed) _then;

/// Create a copy of SaveState
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? message = null,}) {
  return _then(SaveState_Failed(
message: null == message ? _self.message : message // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc
mixin _$SearchScope {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope()';
}


}

/// @nodoc
class $SearchScopeCopyWith<$Res>  {
$SearchScopeCopyWith(SearchScope _, $Res Function(SearchScope) __);
}


/// Adds pattern-matching-related methods to [SearchScope].
extension SearchScopePatterns on SearchScope {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( SearchScope_All value)?  all,TResult Function( SearchScope_Titles value)?  titles,TResult Function( SearchScope_Tags value)?  tags,TResult Function( SearchScope_Notebook value)?  notebook,required TResult orElse(),}){
final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all(_that);case SearchScope_Titles() when titles != null:
return titles(_that);case SearchScope_Tags() when tags != null:
return tags(_that);case SearchScope_Notebook() when notebook != null:
return notebook(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( SearchScope_All value)  all,required TResult Function( SearchScope_Titles value)  titles,required TResult Function( SearchScope_Tags value)  tags,required TResult Function( SearchScope_Notebook value)  notebook,}){
final _that = this;
switch (_that) {
case SearchScope_All():
return all(_that);case SearchScope_Titles():
return titles(_that);case SearchScope_Tags():
return tags(_that);case SearchScope_Notebook():
return notebook(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( SearchScope_All value)?  all,TResult? Function( SearchScope_Titles value)?  titles,TResult? Function( SearchScope_Tags value)?  tags,TResult? Function( SearchScope_Notebook value)?  notebook,}){
final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all(_that);case SearchScope_Titles() when titles != null:
return titles(_that);case SearchScope_Tags() when tags != null:
return tags(_that);case SearchScope_Notebook() when notebook != null:
return notebook(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  all,TResult Function()?  titles,TResult Function()?  tags,TResult Function( String id)?  notebook,required TResult orElse(),}) {final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all();case SearchScope_Titles() when titles != null:
return titles();case SearchScope_Tags() when tags != null:
return tags();case SearchScope_Notebook() when notebook != null:
return notebook(_that.id);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  all,required TResult Function()  titles,required TResult Function()  tags,required TResult Function( String id)  notebook,}) {final _that = this;
switch (_that) {
case SearchScope_All():
return all();case SearchScope_Titles():
return titles();case SearchScope_Tags():
return tags();case SearchScope_Notebook():
return notebook(_that.id);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  all,TResult? Function()?  titles,TResult? Function()?  tags,TResult? Function( String id)?  notebook,}) {final _that = this;
switch (_that) {
case SearchScope_All() when all != null:
return all();case SearchScope_Titles() when titles != null:
return titles();case SearchScope_Tags() when tags != null:
return tags();case SearchScope_Notebook() when notebook != null:
return notebook(_that.id);case _:
  return null;

}
}

}

/// @nodoc


class SearchScope_All extends SearchScope {
  const SearchScope_All(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_All);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope.all()';
}


}




/// @nodoc


class SearchScope_Titles extends SearchScope {
  const SearchScope_Titles(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_Titles);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope.titles()';
}


}




/// @nodoc


class SearchScope_Tags extends SearchScope {
  const SearchScope_Tags(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_Tags);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'SearchScope.tags()';
}


}




/// @nodoc


class SearchScope_Notebook extends SearchScope {
  const SearchScope_Notebook({required this.id}): super._();
  

 final  String id;

/// Create a copy of SearchScope
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$SearchScope_NotebookCopyWith<SearchScope_Notebook> get copyWith => _$SearchScope_NotebookCopyWithImpl<SearchScope_Notebook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is SearchScope_Notebook&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'SearchScope.notebook(id: $id)';
}


}

/// @nodoc
abstract mixin class $SearchScope_NotebookCopyWith<$Res> implements $SearchScopeCopyWith<$Res> {
  factory $SearchScope_NotebookCopyWith(SearchScope_Notebook value, $Res Function(SearchScope_Notebook) _then) = _$SearchScope_NotebookCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$SearchScope_NotebookCopyWithImpl<$Res>
    implements $SearchScope_NotebookCopyWith<$Res> {
  _$SearchScope_NotebookCopyWithImpl(this._self, this._then);

  final SearchScope_Notebook _self;
  final $Res Function(SearchScope_Notebook) _then;

/// Create a copy of SearchScope
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(SearchScope_Notebook(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

// dart format on

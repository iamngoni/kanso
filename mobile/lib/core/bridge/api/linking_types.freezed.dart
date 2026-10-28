// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'linking_types.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$PalettePlace {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PalettePlace);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PalettePlace()';
}


}

/// @nodoc
class $PalettePlaceCopyWith<$Res>  {
$PalettePlaceCopyWith(PalettePlace _, $Res Function(PalettePlace) __);
}


/// Adds pattern-matching-related methods to [PalettePlace].
extension PalettePlacePatterns on PalettePlace {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PalettePlace_AllNotes value)?  allNotes,TResult Function( PalettePlace_Pinned value)?  pinned,TResult Function( PalettePlace_Recent value)?  recent,TResult Function( PalettePlace_Tasks value)?  tasks,TResult Function( PalettePlace_Notebook value)?  notebook,TResult Function( PalettePlace_Status value)?  status,TResult Function( PalettePlace_Shared value)?  shared,TResult Function( PalettePlace_Trash value)?  trash,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PalettePlace_AllNotes() when allNotes != null:
return allNotes(_that);case PalettePlace_Pinned() when pinned != null:
return pinned(_that);case PalettePlace_Recent() when recent != null:
return recent(_that);case PalettePlace_Tasks() when tasks != null:
return tasks(_that);case PalettePlace_Notebook() when notebook != null:
return notebook(_that);case PalettePlace_Status() when status != null:
return status(_that);case PalettePlace_Shared() when shared != null:
return shared(_that);case PalettePlace_Trash() when trash != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PalettePlace_AllNotes value)  allNotes,required TResult Function( PalettePlace_Pinned value)  pinned,required TResult Function( PalettePlace_Recent value)  recent,required TResult Function( PalettePlace_Tasks value)  tasks,required TResult Function( PalettePlace_Notebook value)  notebook,required TResult Function( PalettePlace_Status value)  status,required TResult Function( PalettePlace_Shared value)  shared,required TResult Function( PalettePlace_Trash value)  trash,}){
final _that = this;
switch (_that) {
case PalettePlace_AllNotes():
return allNotes(_that);case PalettePlace_Pinned():
return pinned(_that);case PalettePlace_Recent():
return recent(_that);case PalettePlace_Tasks():
return tasks(_that);case PalettePlace_Notebook():
return notebook(_that);case PalettePlace_Status():
return status(_that);case PalettePlace_Shared():
return shared(_that);case PalettePlace_Trash():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PalettePlace_AllNotes value)?  allNotes,TResult? Function( PalettePlace_Pinned value)?  pinned,TResult? Function( PalettePlace_Recent value)?  recent,TResult? Function( PalettePlace_Tasks value)?  tasks,TResult? Function( PalettePlace_Notebook value)?  notebook,TResult? Function( PalettePlace_Status value)?  status,TResult? Function( PalettePlace_Shared value)?  shared,TResult? Function( PalettePlace_Trash value)?  trash,}){
final _that = this;
switch (_that) {
case PalettePlace_AllNotes() when allNotes != null:
return allNotes(_that);case PalettePlace_Pinned() when pinned != null:
return pinned(_that);case PalettePlace_Recent() when recent != null:
return recent(_that);case PalettePlace_Tasks() when tasks != null:
return tasks(_that);case PalettePlace_Notebook() when notebook != null:
return notebook(_that);case PalettePlace_Status() when status != null:
return status(_that);case PalettePlace_Shared() when shared != null:
return shared(_that);case PalettePlace_Trash() when trash != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function()?  allNotes,TResult Function()?  pinned,TResult Function()?  recent,TResult Function()?  tasks,TResult Function( String id)?  notebook,TResult Function( String name)?  status,TResult Function()?  shared,TResult Function()?  trash,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PalettePlace_AllNotes() when allNotes != null:
return allNotes();case PalettePlace_Pinned() when pinned != null:
return pinned();case PalettePlace_Recent() when recent != null:
return recent();case PalettePlace_Tasks() when tasks != null:
return tasks();case PalettePlace_Notebook() when notebook != null:
return notebook(_that.id);case PalettePlace_Status() when status != null:
return status(_that.name);case PalettePlace_Shared() when shared != null:
return shared();case PalettePlace_Trash() when trash != null:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function()  allNotes,required TResult Function()  pinned,required TResult Function()  recent,required TResult Function()  tasks,required TResult Function( String id)  notebook,required TResult Function( String name)  status,required TResult Function()  shared,required TResult Function()  trash,}) {final _that = this;
switch (_that) {
case PalettePlace_AllNotes():
return allNotes();case PalettePlace_Pinned():
return pinned();case PalettePlace_Recent():
return recent();case PalettePlace_Tasks():
return tasks();case PalettePlace_Notebook():
return notebook(_that.id);case PalettePlace_Status():
return status(_that.name);case PalettePlace_Shared():
return shared();case PalettePlace_Trash():
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function()?  allNotes,TResult? Function()?  pinned,TResult? Function()?  recent,TResult? Function()?  tasks,TResult? Function( String id)?  notebook,TResult? Function( String name)?  status,TResult? Function()?  shared,TResult? Function()?  trash,}) {final _that = this;
switch (_that) {
case PalettePlace_AllNotes() when allNotes != null:
return allNotes();case PalettePlace_Pinned() when pinned != null:
return pinned();case PalettePlace_Recent() when recent != null:
return recent();case PalettePlace_Tasks() when tasks != null:
return tasks();case PalettePlace_Notebook() when notebook != null:
return notebook(_that.id);case PalettePlace_Status() when status != null:
return status(_that.name);case PalettePlace_Shared() when shared != null:
return shared();case PalettePlace_Trash() when trash != null:
return trash();case _:
  return null;

}
}

}

/// @nodoc


class PalettePlace_AllNotes extends PalettePlace {
  const PalettePlace_AllNotes(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PalettePlace_AllNotes);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PalettePlace.allNotes()';
}


}




/// @nodoc


class PalettePlace_Pinned extends PalettePlace {
  const PalettePlace_Pinned(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PalettePlace_Pinned);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PalettePlace.pinned()';
}


}




/// @nodoc


class PalettePlace_Recent extends PalettePlace {
  const PalettePlace_Recent(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PalettePlace_Recent);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PalettePlace.recent()';
}


}




/// @nodoc


class PalettePlace_Tasks extends PalettePlace {
  const PalettePlace_Tasks(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PalettePlace_Tasks);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PalettePlace.tasks()';
}


}




/// @nodoc


class PalettePlace_Notebook extends PalettePlace {
  const PalettePlace_Notebook({required this.id}): super._();
  

 final  String id;

/// Create a copy of PalettePlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PalettePlace_NotebookCopyWith<PalettePlace_Notebook> get copyWith => _$PalettePlace_NotebookCopyWithImpl<PalettePlace_Notebook>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PalettePlace_Notebook&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'PalettePlace.notebook(id: $id)';
}


}

/// @nodoc
abstract mixin class $PalettePlace_NotebookCopyWith<$Res> implements $PalettePlaceCopyWith<$Res> {
  factory $PalettePlace_NotebookCopyWith(PalettePlace_Notebook value, $Res Function(PalettePlace_Notebook) _then) = _$PalettePlace_NotebookCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$PalettePlace_NotebookCopyWithImpl<$Res>
    implements $PalettePlace_NotebookCopyWith<$Res> {
  _$PalettePlace_NotebookCopyWithImpl(this._self, this._then);

  final PalettePlace_Notebook _self;
  final $Res Function(PalettePlace_Notebook) _then;

/// Create a copy of PalettePlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(PalettePlace_Notebook(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PalettePlace_Status extends PalettePlace {
  const PalettePlace_Status({required this.name}): super._();
  

 final  String name;

/// Create a copy of PalettePlace
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PalettePlace_StatusCopyWith<PalettePlace_Status> get copyWith => _$PalettePlace_StatusCopyWithImpl<PalettePlace_Status>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PalettePlace_Status&&(identical(other.name, name) || other.name == name));
}


@override
int get hashCode => Object.hash(runtimeType,name);

@override
String toString() {
  return 'PalettePlace.status(name: $name)';
}


}

/// @nodoc
abstract mixin class $PalettePlace_StatusCopyWith<$Res> implements $PalettePlaceCopyWith<$Res> {
  factory $PalettePlace_StatusCopyWith(PalettePlace_Status value, $Res Function(PalettePlace_Status) _then) = _$PalettePlace_StatusCopyWithImpl;
@useResult
$Res call({
 String name
});




}
/// @nodoc
class _$PalettePlace_StatusCopyWithImpl<$Res>
    implements $PalettePlace_StatusCopyWith<$Res> {
  _$PalettePlace_StatusCopyWithImpl(this._self, this._then);

  final PalettePlace_Status _self;
  final $Res Function(PalettePlace_Status) _then;

/// Create a copy of PalettePlace
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? name = null,}) {
  return _then(PalettePlace_Status(
name: null == name ? _self.name : name // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PalettePlace_Shared extends PalettePlace {
  const PalettePlace_Shared(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PalettePlace_Shared);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PalettePlace.shared()';
}


}




/// @nodoc


class PalettePlace_Trash extends PalettePlace {
  const PalettePlace_Trash(): super._();
  






@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PalettePlace_Trash);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PalettePlace.trash()';
}


}




/// @nodoc
mixin _$PaletteTarget {





@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteTarget);
}


@override
int get hashCode => runtimeType.hashCode;

@override
String toString() {
  return 'PaletteTarget()';
}


}

/// @nodoc
class $PaletteTargetCopyWith<$Res>  {
$PaletteTargetCopyWith(PaletteTarget _, $Res Function(PaletteTarget) __);
}


/// Adds pattern-matching-related methods to [PaletteTarget].
extension PaletteTargetPatterns on PaletteTarget {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>({TResult Function( PaletteTarget_Command value)?  command,TResult Function( PaletteTarget_Place value)?  place,TResult Function( PaletteTarget_Note value)?  note,TResult Function( PaletteTarget_Heading value)?  heading,required TResult orElse(),}){
final _that = this;
switch (_that) {
case PaletteTarget_Command() when command != null:
return command(_that);case PaletteTarget_Place() when place != null:
return place(_that);case PaletteTarget_Note() when note != null:
return note(_that);case PaletteTarget_Heading() when heading != null:
return heading(_that);case _:
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

@optionalTypeArgs TResult map<TResult extends Object?>({required TResult Function( PaletteTarget_Command value)  command,required TResult Function( PaletteTarget_Place value)  place,required TResult Function( PaletteTarget_Note value)  note,required TResult Function( PaletteTarget_Heading value)  heading,}){
final _that = this;
switch (_that) {
case PaletteTarget_Command():
return command(_that);case PaletteTarget_Place():
return place(_that);case PaletteTarget_Note():
return note(_that);case PaletteTarget_Heading():
return heading(_that);}
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>({TResult? Function( PaletteTarget_Command value)?  command,TResult? Function( PaletteTarget_Place value)?  place,TResult? Function( PaletteTarget_Note value)?  note,TResult? Function( PaletteTarget_Heading value)?  heading,}){
final _that = this;
switch (_that) {
case PaletteTarget_Command() when command != null:
return command(_that);case PaletteTarget_Place() when place != null:
return place(_that);case PaletteTarget_Note() when note != null:
return note(_that);case PaletteTarget_Heading() when heading != null:
return heading(_that);case _:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>({TResult Function( String id,  PalettePicker? picker)?  command,TResult Function( PalettePlace place)?  place,TResult Function( String id)?  note,TResult Function( String noteId,  int line)?  heading,required TResult orElse(),}) {final _that = this;
switch (_that) {
case PaletteTarget_Command() when command != null:
return command(_that.id,_that.picker);case PaletteTarget_Place() when place != null:
return place(_that.place);case PaletteTarget_Note() when note != null:
return note(_that.id);case PaletteTarget_Heading() when heading != null:
return heading(_that.noteId,_that.line);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>({required TResult Function( String id,  PalettePicker? picker)  command,required TResult Function( PalettePlace place)  place,required TResult Function( String id)  note,required TResult Function( String noteId,  int line)  heading,}) {final _that = this;
switch (_that) {
case PaletteTarget_Command():
return command(_that.id,_that.picker);case PaletteTarget_Place():
return place(_that.place);case PaletteTarget_Note():
return note(_that.id);case PaletteTarget_Heading():
return heading(_that.noteId,_that.line);}
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>({TResult? Function( String id,  PalettePicker? picker)?  command,TResult? Function( PalettePlace place)?  place,TResult? Function( String id)?  note,TResult? Function( String noteId,  int line)?  heading,}) {final _that = this;
switch (_that) {
case PaletteTarget_Command() when command != null:
return command(_that.id,_that.picker);case PaletteTarget_Place() when place != null:
return place(_that.place);case PaletteTarget_Note() when note != null:
return note(_that.id);case PaletteTarget_Heading() when heading != null:
return heading(_that.noteId,_that.line);case _:
  return null;

}
}

}

/// @nodoc


class PaletteTarget_Command extends PaletteTarget {
  const PaletteTarget_Command({required this.id, this.picker}): super._();
  

 final  String id;
 final  PalettePicker? picker;

/// Create a copy of PaletteTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PaletteTarget_CommandCopyWith<PaletteTarget_Command> get copyWith => _$PaletteTarget_CommandCopyWithImpl<PaletteTarget_Command>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteTarget_Command&&(identical(other.id, id) || other.id == id)&&(identical(other.picker, picker) || other.picker == picker));
}


@override
int get hashCode => Object.hash(runtimeType,id,picker);

@override
String toString() {
  return 'PaletteTarget.command(id: $id, picker: $picker)';
}


}

/// @nodoc
abstract mixin class $PaletteTarget_CommandCopyWith<$Res> implements $PaletteTargetCopyWith<$Res> {
  factory $PaletteTarget_CommandCopyWith(PaletteTarget_Command value, $Res Function(PaletteTarget_Command) _then) = _$PaletteTarget_CommandCopyWithImpl;
@useResult
$Res call({
 String id, PalettePicker? picker
});




}
/// @nodoc
class _$PaletteTarget_CommandCopyWithImpl<$Res>
    implements $PaletteTarget_CommandCopyWith<$Res> {
  _$PaletteTarget_CommandCopyWithImpl(this._self, this._then);

  final PaletteTarget_Command _self;
  final $Res Function(PaletteTarget_Command) _then;

/// Create a copy of PaletteTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,Object? picker = freezed,}) {
  return _then(PaletteTarget_Command(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,picker: freezed == picker ? _self.picker : picker // ignore: cast_nullable_to_non_nullable
as PalettePicker?,
  ));
}


}

/// @nodoc


class PaletteTarget_Place extends PaletteTarget {
  const PaletteTarget_Place({required this.place}): super._();
  

 final  PalettePlace place;

/// Create a copy of PaletteTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PaletteTarget_PlaceCopyWith<PaletteTarget_Place> get copyWith => _$PaletteTarget_PlaceCopyWithImpl<PaletteTarget_Place>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteTarget_Place&&(identical(other.place, place) || other.place == place));
}


@override
int get hashCode => Object.hash(runtimeType,place);

@override
String toString() {
  return 'PaletteTarget.place(place: $place)';
}


}

/// @nodoc
abstract mixin class $PaletteTarget_PlaceCopyWith<$Res> implements $PaletteTargetCopyWith<$Res> {
  factory $PaletteTarget_PlaceCopyWith(PaletteTarget_Place value, $Res Function(PaletteTarget_Place) _then) = _$PaletteTarget_PlaceCopyWithImpl;
@useResult
$Res call({
 PalettePlace place
});


$PalettePlaceCopyWith<$Res> get place;

}
/// @nodoc
class _$PaletteTarget_PlaceCopyWithImpl<$Res>
    implements $PaletteTarget_PlaceCopyWith<$Res> {
  _$PaletteTarget_PlaceCopyWithImpl(this._self, this._then);

  final PaletteTarget_Place _self;
  final $Res Function(PaletteTarget_Place) _then;

/// Create a copy of PaletteTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? place = null,}) {
  return _then(PaletteTarget_Place(
place: null == place ? _self.place : place // ignore: cast_nullable_to_non_nullable
as PalettePlace,
  ));
}

/// Create a copy of PaletteTarget
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$PalettePlaceCopyWith<$Res> get place {
  
  return $PalettePlaceCopyWith<$Res>(_self.place, (value) {
    return _then(_self.copyWith(place: value));
  });
}
}

/// @nodoc


class PaletteTarget_Note extends PaletteTarget {
  const PaletteTarget_Note({required this.id}): super._();
  

 final  String id;

/// Create a copy of PaletteTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PaletteTarget_NoteCopyWith<PaletteTarget_Note> get copyWith => _$PaletteTarget_NoteCopyWithImpl<PaletteTarget_Note>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteTarget_Note&&(identical(other.id, id) || other.id == id));
}


@override
int get hashCode => Object.hash(runtimeType,id);

@override
String toString() {
  return 'PaletteTarget.note(id: $id)';
}


}

/// @nodoc
abstract mixin class $PaletteTarget_NoteCopyWith<$Res> implements $PaletteTargetCopyWith<$Res> {
  factory $PaletteTarget_NoteCopyWith(PaletteTarget_Note value, $Res Function(PaletteTarget_Note) _then) = _$PaletteTarget_NoteCopyWithImpl;
@useResult
$Res call({
 String id
});




}
/// @nodoc
class _$PaletteTarget_NoteCopyWithImpl<$Res>
    implements $PaletteTarget_NoteCopyWith<$Res> {
  _$PaletteTarget_NoteCopyWithImpl(this._self, this._then);

  final PaletteTarget_Note _self;
  final $Res Function(PaletteTarget_Note) _then;

/// Create a copy of PaletteTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? id = null,}) {
  return _then(PaletteTarget_Note(
id: null == id ? _self.id : id // ignore: cast_nullable_to_non_nullable
as String,
  ));
}


}

/// @nodoc


class PaletteTarget_Heading extends PaletteTarget {
  const PaletteTarget_Heading({required this.noteId, required this.line}): super._();
  

 final  String noteId;
 final  int line;

/// Create a copy of PaletteTarget
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$PaletteTarget_HeadingCopyWith<PaletteTarget_Heading> get copyWith => _$PaletteTarget_HeadingCopyWithImpl<PaletteTarget_Heading>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is PaletteTarget_Heading&&(identical(other.noteId, noteId) || other.noteId == noteId)&&(identical(other.line, line) || other.line == line));
}


@override
int get hashCode => Object.hash(runtimeType,noteId,line);

@override
String toString() {
  return 'PaletteTarget.heading(noteId: $noteId, line: $line)';
}


}

/// @nodoc
abstract mixin class $PaletteTarget_HeadingCopyWith<$Res> implements $PaletteTargetCopyWith<$Res> {
  factory $PaletteTarget_HeadingCopyWith(PaletteTarget_Heading value, $Res Function(PaletteTarget_Heading) _then) = _$PaletteTarget_HeadingCopyWithImpl;
@useResult
$Res call({
 String noteId, int line
});




}
/// @nodoc
class _$PaletteTarget_HeadingCopyWithImpl<$Res>
    implements $PaletteTarget_HeadingCopyWith<$Res> {
  _$PaletteTarget_HeadingCopyWithImpl(this._self, this._then);

  final PaletteTarget_Heading _self;
  final $Res Function(PaletteTarget_Heading) _then;

/// Create a copy of PaletteTarget
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') $Res call({Object? noteId = null,Object? line = null,}) {
  return _then(PaletteTarget_Heading(
noteId: null == noteId ? _self.noteId : noteId // ignore: cast_nullable_to_non_nullable
as String,line: null == line ? _self.line : line // ignore: cast_nullable_to_non_nullable
as int,
  ));
}


}

// dart format on

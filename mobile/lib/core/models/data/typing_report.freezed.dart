// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'typing_report.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;
/// @nodoc
mixin _$TypingReport {

/// How many words and UTF-16 units the note had when typing began.
 int get words; int get characters;/// How many keystrokes (insertions, Returns, undos) were made.
 int get keystrokes;/// The frames drawn while typing, against the display's refresh interval.
 FrameReport get frames;/// The frames drawn while the caret was put somewhere far away (the end
/// of the note, its middle): which of them took longest, in
/// milliseconds, and how many there were. They are not in [frames]: a
/// person scrolls to such a place before tapping it.
 int get jumpFrames; double get jumpBuildMax;/// How many times the window moved or was replaced in all.
 int get windowMoves;/// From a keystroke being handed to the field to the end of the raster
/// of the frame that shows it: 50th, 95th and 99th percentile, in
/// milliseconds.
 double get latencyP50; double get latencyP95; double get latencyP99;/// The median and 99th percentile of the field's call to the core for a
/// change (`kanso.apply`), how many calls were timed, and all their time,
/// in milliseconds.
 double get applyP50; double get applyP99; int get applyCalls; double get applyTotalMs;/// The same for building the text's spans (`kanso.buildTextSpan`) when
/// they were not already built.
 double get buildSpanP50; double get buildSpanP99; int get buildSpanCalls; double get buildSpanTotalMs;/// All the time the frames took to build, on the UI thread, in
/// milliseconds: what [applyTotalMs] and [buildSpanTotalMs] are part of
/// (or, for the calls outside a frame, are not).
 double get frameBuildTotalMs;/// From a change to the fresh colours of its text being shown: 50th and
/// 95th percentile, in milliseconds, and how many were timed.
 double get freshRunsP50; double get freshRunsP95; int get freshRunsTimed;/// The process's resident memory before and after the long stretch of
/// typing, in megabytes.
 double get rssBeforeMb; double get rssAfterMb;/// How far the engine's clock was ahead of the end of the last frame
/// when it was reported, in milliseconds: small and positive when the
/// timestamps of frames and the clock keystrokes are stamped with are
/// the same.
 double get clockSkewMs;
/// Create a copy of TypingReport
/// with the given fields replaced by the non-null parameter values.
@JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
$TypingReportCopyWith<TypingReport> get copyWith => _$TypingReportCopyWithImpl<TypingReport>(this as TypingReport, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is TypingReport&&(identical(other.words, words) || other.words == words)&&(identical(other.characters, characters) || other.characters == characters)&&(identical(other.keystrokes, keystrokes) || other.keystrokes == keystrokes)&&(identical(other.frames, frames) || other.frames == frames)&&(identical(other.jumpFrames, jumpFrames) || other.jumpFrames == jumpFrames)&&(identical(other.jumpBuildMax, jumpBuildMax) || other.jumpBuildMax == jumpBuildMax)&&(identical(other.windowMoves, windowMoves) || other.windowMoves == windowMoves)&&(identical(other.latencyP50, latencyP50) || other.latencyP50 == latencyP50)&&(identical(other.latencyP95, latencyP95) || other.latencyP95 == latencyP95)&&(identical(other.latencyP99, latencyP99) || other.latencyP99 == latencyP99)&&(identical(other.applyP50, applyP50) || other.applyP50 == applyP50)&&(identical(other.applyP99, applyP99) || other.applyP99 == applyP99)&&(identical(other.applyCalls, applyCalls) || other.applyCalls == applyCalls)&&(identical(other.applyTotalMs, applyTotalMs) || other.applyTotalMs == applyTotalMs)&&(identical(other.buildSpanP50, buildSpanP50) || other.buildSpanP50 == buildSpanP50)&&(identical(other.buildSpanP99, buildSpanP99) || other.buildSpanP99 == buildSpanP99)&&(identical(other.buildSpanCalls, buildSpanCalls) || other.buildSpanCalls == buildSpanCalls)&&(identical(other.buildSpanTotalMs, buildSpanTotalMs) || other.buildSpanTotalMs == buildSpanTotalMs)&&(identical(other.frameBuildTotalMs, frameBuildTotalMs) || other.frameBuildTotalMs == frameBuildTotalMs)&&(identical(other.freshRunsP50, freshRunsP50) || other.freshRunsP50 == freshRunsP50)&&(identical(other.freshRunsP95, freshRunsP95) || other.freshRunsP95 == freshRunsP95)&&(identical(other.freshRunsTimed, freshRunsTimed) || other.freshRunsTimed == freshRunsTimed)&&(identical(other.rssBeforeMb, rssBeforeMb) || other.rssBeforeMb == rssBeforeMb)&&(identical(other.rssAfterMb, rssAfterMb) || other.rssAfterMb == rssAfterMb)&&(identical(other.clockSkewMs, clockSkewMs) || other.clockSkewMs == clockSkewMs));
}


@override
int get hashCode => Object.hashAll([runtimeType,words,characters,keystrokes,frames,jumpFrames,jumpBuildMax,windowMoves,latencyP50,latencyP95,latencyP99,applyP50,applyP99,applyCalls,applyTotalMs,buildSpanP50,buildSpanP99,buildSpanCalls,buildSpanTotalMs,frameBuildTotalMs,freshRunsP50,freshRunsP95,freshRunsTimed,rssBeforeMb,rssAfterMb,clockSkewMs]);

@override
String toString() {
  return 'TypingReport(words: $words, characters: $characters, keystrokes: $keystrokes, frames: $frames, jumpFrames: $jumpFrames, jumpBuildMax: $jumpBuildMax, windowMoves: $windowMoves, latencyP50: $latencyP50, latencyP95: $latencyP95, latencyP99: $latencyP99, applyP50: $applyP50, applyP99: $applyP99, applyCalls: $applyCalls, applyTotalMs: $applyTotalMs, buildSpanP50: $buildSpanP50, buildSpanP99: $buildSpanP99, buildSpanCalls: $buildSpanCalls, buildSpanTotalMs: $buildSpanTotalMs, frameBuildTotalMs: $frameBuildTotalMs, freshRunsP50: $freshRunsP50, freshRunsP95: $freshRunsP95, freshRunsTimed: $freshRunsTimed, rssBeforeMb: $rssBeforeMb, rssAfterMb: $rssAfterMb, clockSkewMs: $clockSkewMs)';
}


}

/// @nodoc
abstract mixin class $TypingReportCopyWith<$Res>  {
  factory $TypingReportCopyWith(TypingReport value, $Res Function(TypingReport) _then) = _$TypingReportCopyWithImpl;
@useResult
$Res call({
 int words, int characters, int keystrokes, FrameReport frames, int jumpFrames, double jumpBuildMax, int windowMoves, double latencyP50, double latencyP95, double latencyP99, double applyP50, double applyP99, int applyCalls, double applyTotalMs, double buildSpanP50, double buildSpanP99, int buildSpanCalls, double buildSpanTotalMs, double frameBuildTotalMs, double freshRunsP50, double freshRunsP95, int freshRunsTimed, double rssBeforeMb, double rssAfterMb, double clockSkewMs
});


$FrameReportCopyWith<$Res> get frames;

}
/// @nodoc
class _$TypingReportCopyWithImpl<$Res>
    implements $TypingReportCopyWith<$Res> {
  _$TypingReportCopyWithImpl(this._self, this._then);

  final TypingReport _self;
  final $Res Function(TypingReport) _then;

/// Create a copy of TypingReport
/// with the given fields replaced by the non-null parameter values.
@pragma('vm:prefer-inline') @override $Res call({Object? words = null,Object? characters = null,Object? keystrokes = null,Object? frames = null,Object? jumpFrames = null,Object? jumpBuildMax = null,Object? windowMoves = null,Object? latencyP50 = null,Object? latencyP95 = null,Object? latencyP99 = null,Object? applyP50 = null,Object? applyP99 = null,Object? applyCalls = null,Object? applyTotalMs = null,Object? buildSpanP50 = null,Object? buildSpanP99 = null,Object? buildSpanCalls = null,Object? buildSpanTotalMs = null,Object? frameBuildTotalMs = null,Object? freshRunsP50 = null,Object? freshRunsP95 = null,Object? freshRunsTimed = null,Object? rssBeforeMb = null,Object? rssAfterMb = null,Object? clockSkewMs = null,}) {
  return _then(_self.copyWith(
words: null == words ? _self.words : words // ignore: cast_nullable_to_non_nullable
as int,characters: null == characters ? _self.characters : characters // ignore: cast_nullable_to_non_nullable
as int,keystrokes: null == keystrokes ? _self.keystrokes : keystrokes // ignore: cast_nullable_to_non_nullable
as int,frames: null == frames ? _self.frames : frames // ignore: cast_nullable_to_non_nullable
as FrameReport,jumpFrames: null == jumpFrames ? _self.jumpFrames : jumpFrames // ignore: cast_nullable_to_non_nullable
as int,jumpBuildMax: null == jumpBuildMax ? _self.jumpBuildMax : jumpBuildMax // ignore: cast_nullable_to_non_nullable
as double,windowMoves: null == windowMoves ? _self.windowMoves : windowMoves // ignore: cast_nullable_to_non_nullable
as int,latencyP50: null == latencyP50 ? _self.latencyP50 : latencyP50 // ignore: cast_nullable_to_non_nullable
as double,latencyP95: null == latencyP95 ? _self.latencyP95 : latencyP95 // ignore: cast_nullable_to_non_nullable
as double,latencyP99: null == latencyP99 ? _self.latencyP99 : latencyP99 // ignore: cast_nullable_to_non_nullable
as double,applyP50: null == applyP50 ? _self.applyP50 : applyP50 // ignore: cast_nullable_to_non_nullable
as double,applyP99: null == applyP99 ? _self.applyP99 : applyP99 // ignore: cast_nullable_to_non_nullable
as double,applyCalls: null == applyCalls ? _self.applyCalls : applyCalls // ignore: cast_nullable_to_non_nullable
as int,applyTotalMs: null == applyTotalMs ? _self.applyTotalMs : applyTotalMs // ignore: cast_nullable_to_non_nullable
as double,buildSpanP50: null == buildSpanP50 ? _self.buildSpanP50 : buildSpanP50 // ignore: cast_nullable_to_non_nullable
as double,buildSpanP99: null == buildSpanP99 ? _self.buildSpanP99 : buildSpanP99 // ignore: cast_nullable_to_non_nullable
as double,buildSpanCalls: null == buildSpanCalls ? _self.buildSpanCalls : buildSpanCalls // ignore: cast_nullable_to_non_nullable
as int,buildSpanTotalMs: null == buildSpanTotalMs ? _self.buildSpanTotalMs : buildSpanTotalMs // ignore: cast_nullable_to_non_nullable
as double,frameBuildTotalMs: null == frameBuildTotalMs ? _self.frameBuildTotalMs : frameBuildTotalMs // ignore: cast_nullable_to_non_nullable
as double,freshRunsP50: null == freshRunsP50 ? _self.freshRunsP50 : freshRunsP50 // ignore: cast_nullable_to_non_nullable
as double,freshRunsP95: null == freshRunsP95 ? _self.freshRunsP95 : freshRunsP95 // ignore: cast_nullable_to_non_nullable
as double,freshRunsTimed: null == freshRunsTimed ? _self.freshRunsTimed : freshRunsTimed // ignore: cast_nullable_to_non_nullable
as int,rssBeforeMb: null == rssBeforeMb ? _self.rssBeforeMb : rssBeforeMb // ignore: cast_nullable_to_non_nullable
as double,rssAfterMb: null == rssAfterMb ? _self.rssAfterMb : rssAfterMb // ignore: cast_nullable_to_non_nullable
as double,clockSkewMs: null == clockSkewMs ? _self.clockSkewMs : clockSkewMs // ignore: cast_nullable_to_non_nullable
as double,
  ));
}
/// Create a copy of TypingReport
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$FrameReportCopyWith<$Res> get frames {
  
  return $FrameReportCopyWith<$Res>(_self.frames, (value) {
    return _then(_self.copyWith(frames: value));
  });
}
}


/// Adds pattern-matching-related methods to [TypingReport].
extension TypingReportPatterns on TypingReport {
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

@optionalTypeArgs TResult maybeMap<TResult extends Object?>(TResult Function( _TypingReport value)?  $default,{required TResult orElse(),}){
final _that = this;
switch (_that) {
case _TypingReport() when $default != null:
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

@optionalTypeArgs TResult map<TResult extends Object?>(TResult Function( _TypingReport value)  $default,){
final _that = this;
switch (_that) {
case _TypingReport():
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

@optionalTypeArgs TResult? mapOrNull<TResult extends Object?>(TResult? Function( _TypingReport value)?  $default,){
final _that = this;
switch (_that) {
case _TypingReport() when $default != null:
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

@optionalTypeArgs TResult maybeWhen<TResult extends Object?>(TResult Function( int words,  int characters,  int keystrokes,  FrameReport frames,  int jumpFrames,  double jumpBuildMax,  int windowMoves,  double latencyP50,  double latencyP95,  double latencyP99,  double applyP50,  double applyP99,  int applyCalls,  double applyTotalMs,  double buildSpanP50,  double buildSpanP99,  int buildSpanCalls,  double buildSpanTotalMs,  double frameBuildTotalMs,  double freshRunsP50,  double freshRunsP95,  int freshRunsTimed,  double rssBeforeMb,  double rssAfterMb,  double clockSkewMs)?  $default,{required TResult orElse(),}) {final _that = this;
switch (_that) {
case _TypingReport() when $default != null:
return $default(_that.words,_that.characters,_that.keystrokes,_that.frames,_that.jumpFrames,_that.jumpBuildMax,_that.windowMoves,_that.latencyP50,_that.latencyP95,_that.latencyP99,_that.applyP50,_that.applyP99,_that.applyCalls,_that.applyTotalMs,_that.buildSpanP50,_that.buildSpanP99,_that.buildSpanCalls,_that.buildSpanTotalMs,_that.frameBuildTotalMs,_that.freshRunsP50,_that.freshRunsP95,_that.freshRunsTimed,_that.rssBeforeMb,_that.rssAfterMb,_that.clockSkewMs);case _:
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

@optionalTypeArgs TResult when<TResult extends Object?>(TResult Function( int words,  int characters,  int keystrokes,  FrameReport frames,  int jumpFrames,  double jumpBuildMax,  int windowMoves,  double latencyP50,  double latencyP95,  double latencyP99,  double applyP50,  double applyP99,  int applyCalls,  double applyTotalMs,  double buildSpanP50,  double buildSpanP99,  int buildSpanCalls,  double buildSpanTotalMs,  double frameBuildTotalMs,  double freshRunsP50,  double freshRunsP95,  int freshRunsTimed,  double rssBeforeMb,  double rssAfterMb,  double clockSkewMs)  $default,) {final _that = this;
switch (_that) {
case _TypingReport():
return $default(_that.words,_that.characters,_that.keystrokes,_that.frames,_that.jumpFrames,_that.jumpBuildMax,_that.windowMoves,_that.latencyP50,_that.latencyP95,_that.latencyP99,_that.applyP50,_that.applyP99,_that.applyCalls,_that.applyTotalMs,_that.buildSpanP50,_that.buildSpanP99,_that.buildSpanCalls,_that.buildSpanTotalMs,_that.frameBuildTotalMs,_that.freshRunsP50,_that.freshRunsP95,_that.freshRunsTimed,_that.rssBeforeMb,_that.rssAfterMb,_that.clockSkewMs);case _:
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

@optionalTypeArgs TResult? whenOrNull<TResult extends Object?>(TResult? Function( int words,  int characters,  int keystrokes,  FrameReport frames,  int jumpFrames,  double jumpBuildMax,  int windowMoves,  double latencyP50,  double latencyP95,  double latencyP99,  double applyP50,  double applyP99,  int applyCalls,  double applyTotalMs,  double buildSpanP50,  double buildSpanP99,  int buildSpanCalls,  double buildSpanTotalMs,  double frameBuildTotalMs,  double freshRunsP50,  double freshRunsP95,  int freshRunsTimed,  double rssBeforeMb,  double rssAfterMb,  double clockSkewMs)?  $default,) {final _that = this;
switch (_that) {
case _TypingReport() when $default != null:
return $default(_that.words,_that.characters,_that.keystrokes,_that.frames,_that.jumpFrames,_that.jumpBuildMax,_that.windowMoves,_that.latencyP50,_that.latencyP95,_that.latencyP99,_that.applyP50,_that.applyP99,_that.applyCalls,_that.applyTotalMs,_that.buildSpanP50,_that.buildSpanP99,_that.buildSpanCalls,_that.buildSpanTotalMs,_that.frameBuildTotalMs,_that.freshRunsP50,_that.freshRunsP95,_that.freshRunsTimed,_that.rssBeforeMb,_that.rssAfterMb,_that.clockSkewMs);case _:
  return null;

}
}

}

/// @nodoc


class _TypingReport extends TypingReport {
  const _TypingReport({required this.words, required this.characters, required this.keystrokes, required this.frames, required this.jumpFrames, required this.jumpBuildMax, required this.windowMoves, required this.latencyP50, required this.latencyP95, required this.latencyP99, required this.applyP50, required this.applyP99, required this.applyCalls, required this.applyTotalMs, required this.buildSpanP50, required this.buildSpanP99, required this.buildSpanCalls, required this.buildSpanTotalMs, required this.frameBuildTotalMs, required this.freshRunsP50, required this.freshRunsP95, required this.freshRunsTimed, required this.rssBeforeMb, required this.rssAfterMb, required this.clockSkewMs}): super._();
  

/// How many words and UTF-16 units the note had when typing began.
@override final  int words;
@override final  int characters;
/// How many keystrokes (insertions, Returns, undos) were made.
@override final  int keystrokes;
/// The frames drawn while typing, against the display's refresh interval.
@override final  FrameReport frames;
/// The frames drawn while the caret was put somewhere far away (the end
/// of the note, its middle): which of them took longest, in
/// milliseconds, and how many there were. They are not in [frames]: a
/// person scrolls to such a place before tapping it.
@override final  int jumpFrames;
@override final  double jumpBuildMax;
/// How many times the window moved or was replaced in all.
@override final  int windowMoves;
/// From a keystroke being handed to the field to the end of the raster
/// of the frame that shows it: 50th, 95th and 99th percentile, in
/// milliseconds.
@override final  double latencyP50;
@override final  double latencyP95;
@override final  double latencyP99;
/// The median and 99th percentile of the field's call to the core for a
/// change (`kanso.apply`), how many calls were timed, and all their time,
/// in milliseconds.
@override final  double applyP50;
@override final  double applyP99;
@override final  int applyCalls;
@override final  double applyTotalMs;
/// The same for building the text's spans (`kanso.buildTextSpan`) when
/// they were not already built.
@override final  double buildSpanP50;
@override final  double buildSpanP99;
@override final  int buildSpanCalls;
@override final  double buildSpanTotalMs;
/// All the time the frames took to build, on the UI thread, in
/// milliseconds: what [applyTotalMs] and [buildSpanTotalMs] are part of
/// (or, for the calls outside a frame, are not).
@override final  double frameBuildTotalMs;
/// From a change to the fresh colours of its text being shown: 50th and
/// 95th percentile, in milliseconds, and how many were timed.
@override final  double freshRunsP50;
@override final  double freshRunsP95;
@override final  int freshRunsTimed;
/// The process's resident memory before and after the long stretch of
/// typing, in megabytes.
@override final  double rssBeforeMb;
@override final  double rssAfterMb;
/// How far the engine's clock was ahead of the end of the last frame
/// when it was reported, in milliseconds: small and positive when the
/// timestamps of frames and the clock keystrokes are stamped with are
/// the same.
@override final  double clockSkewMs;

/// Create a copy of TypingReport
/// with the given fields replaced by the non-null parameter values.
@override @JsonKey(includeFromJson: false, includeToJson: false)
@pragma('vm:prefer-inline')
_$TypingReportCopyWith<_TypingReport> get copyWith => __$TypingReportCopyWithImpl<_TypingReport>(this, _$identity);



@override
bool operator ==(Object other) {
  return identical(this, other) || (other.runtimeType == runtimeType&&other is _TypingReport&&(identical(other.words, words) || other.words == words)&&(identical(other.characters, characters) || other.characters == characters)&&(identical(other.keystrokes, keystrokes) || other.keystrokes == keystrokes)&&(identical(other.frames, frames) || other.frames == frames)&&(identical(other.jumpFrames, jumpFrames) || other.jumpFrames == jumpFrames)&&(identical(other.jumpBuildMax, jumpBuildMax) || other.jumpBuildMax == jumpBuildMax)&&(identical(other.windowMoves, windowMoves) || other.windowMoves == windowMoves)&&(identical(other.latencyP50, latencyP50) || other.latencyP50 == latencyP50)&&(identical(other.latencyP95, latencyP95) || other.latencyP95 == latencyP95)&&(identical(other.latencyP99, latencyP99) || other.latencyP99 == latencyP99)&&(identical(other.applyP50, applyP50) || other.applyP50 == applyP50)&&(identical(other.applyP99, applyP99) || other.applyP99 == applyP99)&&(identical(other.applyCalls, applyCalls) || other.applyCalls == applyCalls)&&(identical(other.applyTotalMs, applyTotalMs) || other.applyTotalMs == applyTotalMs)&&(identical(other.buildSpanP50, buildSpanP50) || other.buildSpanP50 == buildSpanP50)&&(identical(other.buildSpanP99, buildSpanP99) || other.buildSpanP99 == buildSpanP99)&&(identical(other.buildSpanCalls, buildSpanCalls) || other.buildSpanCalls == buildSpanCalls)&&(identical(other.buildSpanTotalMs, buildSpanTotalMs) || other.buildSpanTotalMs == buildSpanTotalMs)&&(identical(other.frameBuildTotalMs, frameBuildTotalMs) || other.frameBuildTotalMs == frameBuildTotalMs)&&(identical(other.freshRunsP50, freshRunsP50) || other.freshRunsP50 == freshRunsP50)&&(identical(other.freshRunsP95, freshRunsP95) || other.freshRunsP95 == freshRunsP95)&&(identical(other.freshRunsTimed, freshRunsTimed) || other.freshRunsTimed == freshRunsTimed)&&(identical(other.rssBeforeMb, rssBeforeMb) || other.rssBeforeMb == rssBeforeMb)&&(identical(other.rssAfterMb, rssAfterMb) || other.rssAfterMb == rssAfterMb)&&(identical(other.clockSkewMs, clockSkewMs) || other.clockSkewMs == clockSkewMs));
}


@override
int get hashCode => Object.hashAll([runtimeType,words,characters,keystrokes,frames,jumpFrames,jumpBuildMax,windowMoves,latencyP50,latencyP95,latencyP99,applyP50,applyP99,applyCalls,applyTotalMs,buildSpanP50,buildSpanP99,buildSpanCalls,buildSpanTotalMs,frameBuildTotalMs,freshRunsP50,freshRunsP95,freshRunsTimed,rssBeforeMb,rssAfterMb,clockSkewMs]);

@override
String toString() {
  return 'TypingReport(words: $words, characters: $characters, keystrokes: $keystrokes, frames: $frames, jumpFrames: $jumpFrames, jumpBuildMax: $jumpBuildMax, windowMoves: $windowMoves, latencyP50: $latencyP50, latencyP95: $latencyP95, latencyP99: $latencyP99, applyP50: $applyP50, applyP99: $applyP99, applyCalls: $applyCalls, applyTotalMs: $applyTotalMs, buildSpanP50: $buildSpanP50, buildSpanP99: $buildSpanP99, buildSpanCalls: $buildSpanCalls, buildSpanTotalMs: $buildSpanTotalMs, frameBuildTotalMs: $frameBuildTotalMs, freshRunsP50: $freshRunsP50, freshRunsP95: $freshRunsP95, freshRunsTimed: $freshRunsTimed, rssBeforeMb: $rssBeforeMb, rssAfterMb: $rssAfterMb, clockSkewMs: $clockSkewMs)';
}


}

/// @nodoc
abstract mixin class _$TypingReportCopyWith<$Res> implements $TypingReportCopyWith<$Res> {
  factory _$TypingReportCopyWith(_TypingReport value, $Res Function(_TypingReport) _then) = __$TypingReportCopyWithImpl;
@override @useResult
$Res call({
 int words, int characters, int keystrokes, FrameReport frames, int jumpFrames, double jumpBuildMax, int windowMoves, double latencyP50, double latencyP95, double latencyP99, double applyP50, double applyP99, int applyCalls, double applyTotalMs, double buildSpanP50, double buildSpanP99, int buildSpanCalls, double buildSpanTotalMs, double frameBuildTotalMs, double freshRunsP50, double freshRunsP95, int freshRunsTimed, double rssBeforeMb, double rssAfterMb, double clockSkewMs
});


@override $FrameReportCopyWith<$Res> get frames;

}
/// @nodoc
class __$TypingReportCopyWithImpl<$Res>
    implements _$TypingReportCopyWith<$Res> {
  __$TypingReportCopyWithImpl(this._self, this._then);

  final _TypingReport _self;
  final $Res Function(_TypingReport) _then;

/// Create a copy of TypingReport
/// with the given fields replaced by the non-null parameter values.
@override @pragma('vm:prefer-inline') $Res call({Object? words = null,Object? characters = null,Object? keystrokes = null,Object? frames = null,Object? jumpFrames = null,Object? jumpBuildMax = null,Object? windowMoves = null,Object? latencyP50 = null,Object? latencyP95 = null,Object? latencyP99 = null,Object? applyP50 = null,Object? applyP99 = null,Object? applyCalls = null,Object? applyTotalMs = null,Object? buildSpanP50 = null,Object? buildSpanP99 = null,Object? buildSpanCalls = null,Object? buildSpanTotalMs = null,Object? frameBuildTotalMs = null,Object? freshRunsP50 = null,Object? freshRunsP95 = null,Object? freshRunsTimed = null,Object? rssBeforeMb = null,Object? rssAfterMb = null,Object? clockSkewMs = null,}) {
  return _then(_TypingReport(
words: null == words ? _self.words : words // ignore: cast_nullable_to_non_nullable
as int,characters: null == characters ? _self.characters : characters // ignore: cast_nullable_to_non_nullable
as int,keystrokes: null == keystrokes ? _self.keystrokes : keystrokes // ignore: cast_nullable_to_non_nullable
as int,frames: null == frames ? _self.frames : frames // ignore: cast_nullable_to_non_nullable
as FrameReport,jumpFrames: null == jumpFrames ? _self.jumpFrames : jumpFrames // ignore: cast_nullable_to_non_nullable
as int,jumpBuildMax: null == jumpBuildMax ? _self.jumpBuildMax : jumpBuildMax // ignore: cast_nullable_to_non_nullable
as double,windowMoves: null == windowMoves ? _self.windowMoves : windowMoves // ignore: cast_nullable_to_non_nullable
as int,latencyP50: null == latencyP50 ? _self.latencyP50 : latencyP50 // ignore: cast_nullable_to_non_nullable
as double,latencyP95: null == latencyP95 ? _self.latencyP95 : latencyP95 // ignore: cast_nullable_to_non_nullable
as double,latencyP99: null == latencyP99 ? _self.latencyP99 : latencyP99 // ignore: cast_nullable_to_non_nullable
as double,applyP50: null == applyP50 ? _self.applyP50 : applyP50 // ignore: cast_nullable_to_non_nullable
as double,applyP99: null == applyP99 ? _self.applyP99 : applyP99 // ignore: cast_nullable_to_non_nullable
as double,applyCalls: null == applyCalls ? _self.applyCalls : applyCalls // ignore: cast_nullable_to_non_nullable
as int,applyTotalMs: null == applyTotalMs ? _self.applyTotalMs : applyTotalMs // ignore: cast_nullable_to_non_nullable
as double,buildSpanP50: null == buildSpanP50 ? _self.buildSpanP50 : buildSpanP50 // ignore: cast_nullable_to_non_nullable
as double,buildSpanP99: null == buildSpanP99 ? _self.buildSpanP99 : buildSpanP99 // ignore: cast_nullable_to_non_nullable
as double,buildSpanCalls: null == buildSpanCalls ? _self.buildSpanCalls : buildSpanCalls // ignore: cast_nullable_to_non_nullable
as int,buildSpanTotalMs: null == buildSpanTotalMs ? _self.buildSpanTotalMs : buildSpanTotalMs // ignore: cast_nullable_to_non_nullable
as double,frameBuildTotalMs: null == frameBuildTotalMs ? _self.frameBuildTotalMs : frameBuildTotalMs // ignore: cast_nullable_to_non_nullable
as double,freshRunsP50: null == freshRunsP50 ? _self.freshRunsP50 : freshRunsP50 // ignore: cast_nullable_to_non_nullable
as double,freshRunsP95: null == freshRunsP95 ? _self.freshRunsP95 : freshRunsP95 // ignore: cast_nullable_to_non_nullable
as double,freshRunsTimed: null == freshRunsTimed ? _self.freshRunsTimed : freshRunsTimed // ignore: cast_nullable_to_non_nullable
as int,rssBeforeMb: null == rssBeforeMb ? _self.rssBeforeMb : rssBeforeMb // ignore: cast_nullable_to_non_nullable
as double,rssAfterMb: null == rssAfterMb ? _self.rssAfterMb : rssAfterMb // ignore: cast_nullable_to_non_nullable
as double,clockSkewMs: null == clockSkewMs ? _self.clockSkewMs : clockSkewMs // ignore: cast_nullable_to_non_nullable
as double,
  ));
}

/// Create a copy of TypingReport
/// with the given fields replaced by the non-null parameter values.
@override
@pragma('vm:prefer-inline')
$FrameReportCopyWith<$Res> get frames {
  
  return $FrameReportCopyWith<$Res>(_self.frames, (value) {
    return _then(_self.copyWith(frames: value));
  });
}
}

// dart format on

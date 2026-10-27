// Kanso — protobuf_reader.dart
// Created 2026-10-26.
// Licensed under AGPL-3.0-or-later (see LICENSE).

import 'dart:convert';
import 'dart:typed_data';

/// A minimal reader of the protobuf wire format: tags, varints, fixed-width
/// numbers and length-delimited fields, each only as far as the bytes go.
///
/// It reads what it is asked for and [skip]s the rest by wire type; it never
/// looks at a field's meaning. Anything malformed throws a [FormatException]
/// (the caller turns that into "no frames").
final class ProtobufReader {
  ProtobufReader(this._bytes) : _end = _bytes.length;

  final Uint8List _bytes;
  final int _end;
  int _position = 0;

  /// The wire type of a varint.
  static const int varintType = 0;

  /// The wire type of eight fixed bytes.
  static const int fixed64Type = 1;

  /// The wire type of a length and that many bytes.
  static const int delimitedType = 2;

  /// The wire type of four fixed bytes.
  static const int fixed32Type = 5;

  /// The longest string [readString] reads; longer ones are not strings of
  /// the kind a report may hold.
  static const int maxStringBytes = 1024;

  /// Whether every byte has been read.
  bool get atEnd => _position >= _end;

  /// Reads a varint as a 64-bit number (a negative [int] is a number with
  /// its top bit set).
  int readVarint() {
    var result = 0;
    for (var shift = 0; shift < 70; shift += 7) {
      if (_position >= _end) {
        throw const FormatException('varint ends early');
      }
      final byte = _bytes[_position++];
      if (shift < 64) {
        result |= (byte & 0x7f) << shift;
      }
      if (byte & 0x80 == 0) {
        return result;
      }
    }
    throw const FormatException('varint too long');
  }

  /// Reads a tag: the field number and the wire type.
  (int field, int wire) readTag() {
    final tag = readVarint();
    final field = tag >>> 3;
    if (field == 0) {
      throw const FormatException('field 0');
    }
    return (field, tag & 7);
  }

  /// Reads a length-delimited field as a view of the bytes, without a copy.
  Uint8List readBytes() {
    final length = readVarint();
    if (length < 0 || length > _end - _position) {
      throw const FormatException('length past the end');
    }
    final view = Uint8List.sublistView(_bytes, _position, _position + length);
    _position += length;
    return view;
  }

  /// Reads a length-delimited field as text, or null if it is longer than
  /// [maxStringBytes].
  String? readString() {
    final bytes = readBytes();
    return bytes.length > maxStringBytes
        ? null
        : utf8.decode(bytes, allowMalformed: true);
  }

  /// Skips the field of [wire] type that follows its tag.
  void skip(int wire) {
    switch (wire) {
      case varintType:
        readVarint();
      case fixed64Type:
        _advance(8);
      case delimitedType:
        readBytes();
      case fixed32Type:
        _advance(4);
      default:
        throw FormatException('wire type $wire');
    }
  }

  void _advance(int count) {
    if (count > _end - _position) {
      throw const FormatException('fixed field past the end');
    }
    _position += count;
  }
}

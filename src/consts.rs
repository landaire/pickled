// Copyright (c) 2015-2021 Georg Brandl.  Licensed under the Apache License,
// Version 2.0 <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0>
// or the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at
// your option. This file may not be copied, modified, or distributed except
// according to those terms.

//! Constants for pickle opcodes.
//!
//! These constants use the names Python's pickle.py uses.  They are not in an
//! enum because it's not very useful to make it one.

use std::convert::TryFrom;

use crate::ErrorCode;

pub const MARK             : u8 = b'(';    // push special markobject on stack
pub const STOP             : u8 = b'.';    // every pickle ends with STOP
pub const POP              : u8 = b'0';    // discard topmost stack item
pub const POP_MARK         : u8 = b'1';    // discard stack top through topmost markobject
pub const DUP              : u8 = b'2';    // duplicate top stack item
pub const FLOAT            : u8 = b'F';    // push float object; decimal string argument
pub const INT              : u8 = b'I';    // push integer or bool; decimal string argument
pub const BININT           : u8 = b'J';    // push four-byte signed int
pub const BININT1          : u8 = b'K';    // push 1-byte unsigned int
pub const LONG             : u8 = b'L';    // push long; decimal string argument
pub const BININT2          : u8 = b'M';    // push 2-byte unsigned int
pub const NONE             : u8 = b'N';    // push None
pub const STRING           : u8 = b'S';    // push string; NL-terminated string argument
pub const BINSTRING        : u8 = b'T';    // push string; counted binary string argument
pub const SHORT_BINSTRING  : u8 = b'U';    //  "     "   ;    "      "       "      " < 256 bytes
pub const UNICODE          : u8 = b'V';    // push Unicode string; raw-unicode-escaped'd argument
pub const BINUNICODE       : u8 = b'X';    //   "     "       "  ; counted UTF-8 string argument
pub const APPEND           : u8 = b'a';    // append stack top to list below it
pub const DICT             : u8 = b'd';    // build a dict from stack items
pub const EMPTY_DICT       : u8 = b'}';    // push empty dict
pub const APPENDS          : u8 = b'e';    // extend list on stack by topmost stack slice
pub const LIST             : u8 = b'l';    // build list from topmost stack items
pub const EMPTY_LIST       : u8 = b']';    // push empty list
pub const SETITEM          : u8 = b's';    // add key+value pair to dict
pub const TUPLE            : u8 = b't';    // build tuple from topmost stack items
pub const EMPTY_TUPLE      : u8 = b')';    // push empty tuple
pub const SETITEMS         : u8 = b'u';    // modify dict by adding topmost key+value pairs
pub const BINFLOAT         : u8 = b'G';    // push float; arg is 8-byte float encoding
pub const PUT              : u8 = b'p';    // store stack top in memo; index is string arg
pub const BINPUT           : u8 = b'q';    //   "     "    "   "   " ;   "    " 1-byte arg
pub const LONG_BINPUT      : u8 = b'r';    //   "     "    "   "   " ;   "    " 4-byte arg
pub const GET              : u8 = b'g';    // push item from memo on stack; index is string arg
pub const BINGET           : u8 = b'h';    //   "    "    "    "   "   "  ;   "    " 1-byte arg
pub const LONG_BINGET      : u8 = b'j';    // push item from memo on stack; index is 4-byte arg
pub const GLOBAL           : u8 = b'c';    // push self.find_class(modname, name); 2 string args
pub const STACK_GLOBAL     : u8 = b'\x93'; // same as GLOBAL but using names on the stacks
pub const REDUCE           : u8 = b'R';    // apply callable to argtuple, both on stack
pub const PROTO            : u8 = b'\x80'; // identify pickle protocol
pub const TUPLE1           : u8 = b'\x85'; // build 1-tuple from stack top
pub const TUPLE2           : u8 = b'\x86'; // build 2-tuple from two topmost stack items
pub const TUPLE3           : u8 = b'\x87'; // build 3-tuple from three topmost stack items
pub const NEWTRUE          : u8 = b'\x88'; // push True
pub const NEWFALSE         : u8 = b'\x89'; // push False
pub const LONG1            : u8 = b'\x8a'; // push long from < 256 bytes
pub const LONG4            : u8 = b'\x8b'; // push really big long
pub const BINBYTES         : u8 = b'B';    // push bytes; counted binary string argument
pub const SHORT_BINBYTES   : u8 = b'C';    //  "     "   ;    "      "       "      " < 256 bytes
pub const SHORT_BINUNICODE : u8 = b'\x8c'; // push short string; UTF-8 length < 256 bytes
pub const BINUNICODE8      : u8 = b'\x8d'; // push very long string
pub const BINBYTES8        : u8 = b'\x8e'; // push very long bytes string
pub const EMPTY_SET        : u8 = b'\x8f'; // push empty set on the stack
pub const ADDITEMS         : u8 = b'\x90'; // modify set by adding topmost stack items
pub const FROZENSET        : u8 = b'\x91'; // build frozenset from topmost stack items
pub const MEMOIZE          : u8 = b'\x94'; // store top of the stack in memo
pub const FRAME            : u8 = b'\x95'; // indicate the beginning of a new frame
pub const INST             : u8 = b'i';    // build & push class instance
pub const OBJ              : u8 = b'o';    // build & push class instance
pub const BUILD            : u8 = b'b';    // call __setstate__ or __dict__.update()
pub const NEWOBJ           : u8 = b'\x81'; // build object by applying cls.__new__ to argtuple
pub const NEWOBJ_EX        : u8 = b'\x92'; // like NEWOBJ but work with keyword only arguments
pub const BYTEARRAY8       : u8 = b'\x96'; // push bytearray

#[repr(u8)]
#[derive(Debug, Copy, Clone)]
pub enum Opcode {
    Mark = b'(',             // push special markobject on stack
    Stop = b'.',             // every pickle ends with STOP
    Pop = b'0',              // discard topmost stack item
    PopMark = b'1',          // discard stack top through topmost markobject
    Dup = b'2',              // duplicate top stack item
    Float = b'F',            // push float object; decimal string argument
    Int = b'I',              // push integer or bool; decimal string argument
    BinInt = b'J',           // push four-byte signed int
    BinInt1 = b'K',          // push 1-byte unsigned int
    Long = b'L',             // push long; decimal string argument
    BinInt2 = b'M',          // push 2-byte unsigned int
    None = b'N',             // push None
    String = b'S',           // push string; NL-terminated string argument
    BinString = b'T',        // push string; counted binary string argument
    ShortBinString = b'U',   // "     "   ;    "      "       "      " < 256 bytes
    Unicode = b'V',          // push Unicode string; raw-unicode-escaped'd argument
    BinUnicode = b'X',       // "     "       "  ; counted UTF-8 string argument
    Append = b'a',           // append stack top to list below it
    Dict = b'd',             // build a dict from stack items
    EmptyDict = b'}',        // push empty dict
    Appends = b'e',          // extend list on stack by topmost stack slice
    List = b'l',             // build list from topmost stack items
    EmptyList = b']',        // push empty list
    SetItem = b's',          // add key+value pair to dict
    Tuple = b't',            // build tuple from topmost stack items
    EmptyTuple = b')',       // push empty tuple
    SetItems = b'u',         // modify dict by adding topmost key+value pairs
    BinFloat = b'G',         // push float; arg is 8-byte float encoding
    Put = b'p',              // store stack top in memo; index is string arg
    BinPut = b'q',           // "     "    "   "   " ;   "    " 1-byte arg
    LongBinPut = b'r',       // "     "    "   "   " ;   "    " 4-byte arg
    Get = b'g',              // push item from memo on stack; index is string arg
    BinGet = b'h',           // "    "    "    "   "   "  ;   "    " 1-byte arg
    LongBinGet = b'j',       // push item from memo on stack; index is 4-byte arg
    Global = b'c',           // push self.find_class(modname, name); 2 string args
    StackGlobal = b'\x93',   // same as GLOBAL but using names on the stacks
    Reduce = b'R',           // apply callable to argtuple, both on stack
    Proto = b'\x80',         // identify pickle protocol
    Tuple1 = b'\x85',        // build 1-tuple from stack top
    Tuple2 = b'\x86',        // build 2-tuple from two topmost stack items
    Tuple3 = b'\x87',        // build 3-tuple from three topmost stack items
    NewTrue = b'\x88',       // push True
    NewFalse = b'\x89',      // push False
    Long1 = b'\x8a',         // push long from < 256 bytes
    Long4 = b'\x8b',         // push really big long
    BinBytes = b'B',         // push bytes; counted binary string argument
    ShortBinBytes = b'C',    // "     "   ;    "      "       "      " < 256 bytes
    ShortBinUnicode = b'\x8c', // push short string; UTF-8 length < 256 bytes
    BinUnicode8 = b'\x8d',   // push very long string
    BinBytes8 = b'\x8e',     // push very long bytes string
    EmptySet = b'\x8f',      // push empty set on the stack
    AddItems = b'\x90',      // modify set by adding topmost stack items
    FrozenSet = b'\x91',     // build frozenset from topmost stack items
    Memoize = b'\x94',       // store top of the stack in memo
    Frame = b'\x95',         // indicate the beginning of a new frame
    Inst = b'i',             // build & push class instance
    Obj = b'o',              // build & push class instance
    Build = b'b',            // call __setstate__ or __dict__.update()
    NewObj = b'\x81',        // build object by applying cls.__new__ to argtuple
    NewObjEx = b'\x92',      // like NEWOBJ but work with keyword only arguments
    ByteArray8 = b'\x96',    // push bytearray
}

impl TryFrom<u8> for Opcode {
    type Error = ErrorCode;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'(' => Ok(Opcode::Mark),
            b'.' => Ok(Opcode::Stop),
            b'0' => Ok(Opcode::Pop),
            b'1' => Ok(Opcode::PopMark),
            b'2' => Ok(Opcode::Dup),
            b'F' => Ok(Opcode::Float),
            b'I' => Ok(Opcode::Int),
            b'J' => Ok(Opcode::BinInt),
            b'K' => Ok(Opcode::BinInt1),
            b'L' => Ok(Opcode::Long),
            b'M' => Ok(Opcode::BinInt2),
            b'N' => Ok(Opcode::None),
            b'S' => Ok(Opcode::String),
            b'T' => Ok(Opcode::BinString),
            b'U' => Ok(Opcode::ShortBinString),
            b'V' => Ok(Opcode::Unicode),
            b'X' => Ok(Opcode::BinUnicode),
            b'a' => Ok(Opcode::Append),
            b'd' => Ok(Opcode::Dict),
            b'}' => Ok(Opcode::EmptyDict),
            b'e' => Ok(Opcode::Appends),
            b'l' => Ok(Opcode::List),
            b']' => Ok(Opcode::EmptyList),
            b's' => Ok(Opcode::SetItem),
            b't' => Ok(Opcode::Tuple),
            b')' => Ok(Opcode::EmptyTuple),
            b'u' => Ok(Opcode::SetItems),
            b'G' => Ok(Opcode::BinFloat),
            b'p' => Ok(Opcode::Put),
            b'q' => Ok(Opcode::BinPut),
            b'r' => Ok(Opcode::LongBinPut),
            b'g' => Ok(Opcode::Get),
            b'h' => Ok(Opcode::BinGet),
            b'j' => Ok(Opcode::LongBinGet),
            b'c' => Ok(Opcode::Global),
            b'\x93' => Ok(Opcode::StackGlobal),
            b'R' => Ok(Opcode::Reduce),
            b'\x80' => Ok(Opcode::Proto),
            b'\x85' => Ok(Opcode::Tuple1),
            b'\x86' => Ok(Opcode::Tuple2),
            b'\x87' => Ok(Opcode::Tuple3),
            b'\x88' => Ok(Opcode::NewTrue),
            b'\x89' => Ok(Opcode::NewFalse),
            b'\x8a' => Ok(Opcode::Long1),
            b'\x8b' => Ok(Opcode::Long4),
            b'B' => Ok(Opcode::BinBytes),
            b'C' => Ok(Opcode::ShortBinBytes),
            b'\x8c' => Ok(Opcode::ShortBinUnicode),
            b'\x8d' => Ok(Opcode::BinUnicode8),
            b'\x8e' => Ok(Opcode::BinBytes8),
            b'\x8f' => Ok(Opcode::EmptySet),
            b'\x90' => Ok(Opcode::AddItems),
            b'\x91' => Ok(Opcode::FrozenSet),
            b'\x94' => Ok(Opcode::Memoize),
            b'\x95' => Ok(Opcode::Frame),
            b'i' => Ok(Opcode::Inst),
            b'o' => Ok(Opcode::Obj),
            b'b' => Ok(Opcode::Build),
            b'\x81' => Ok(Opcode::NewObj),
            b'\x92' => Ok(Opcode::NewObjEx),
            b'\x96' => Ok(Opcode::ByteArray8),
            _ => Err(ErrorCode::Unsupported(value as char)),
        }
    }
}

// Ops used for out-of-band buffers; these are unsupported.
// pub const NEXT_BUFFER      : u8 = b'\x97'; // push next out-of-band buffer
// pub const READONLY_BUFFER  : u8 = b'\x98'; // make top of stack readonly

// Ops only used for recursive objects; these are unsupported.
// pub const PERSID           : u8 = b'P';    // push persistent object; id is taken from string arg
// pub const BINPERSID        : u8 = b'Q';    //  "       "         "  ;  "  "   "     "  stack
// pub const EXT1             : u8 = b'\x82'; // push object from extension registry; 1-byte index
// pub const EXT2             : u8 = b'\x83'; // ditto, but 2-byte index
// pub const EXT4             : u8 = b'\x84'; // ditto, but 4-byte index

/* Copyright 2017 Mozilla Foundation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! A simple event-driven library for parsing WebAssembly binary files
//! (or streams).
//!
//! The parser library reports events as they happen and only stores
//! parsing information for a brief period of time, making it very fast
//! and memory-efficient. The event-driven model, however, has some drawbacks.
//! If you need random access to the entire WebAssembly data-structure,
//! this is not the right library for you. You could however, build such
//! a data-structure using this library.
//!
//! To get started, create a [`Parser`] using [`Parser::new`] and then follow
//! the examples documented for [`Parser::parse`] or [`Parser::parse_all`].

#![deny(missing_docs)]
#![no_std]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;
#[cfg(feature = "std")]
#[macro_use]
extern crate std;

/// A small "prelude" to use throughout this crate.
///
/// This crate is tagged with `#![no_std]` meaning that we get libcore's prelude
/// by default. This crate also uses `alloc`, however, and common types there
/// like `String`. This custom prelude helps bring those types into scope to
/// avoid having to import each of them manually.
mod prelude {
    pub use alloc::borrow::ToOwned;
    pub use alloc::boxed::Box;
    pub use alloc::format;
    pub use alloc::string::{String, ToString};
    pub use alloc::vec;
    pub use alloc::vec::Vec;

    #[cfg(all(feature = "validate", feature = "component-model"))]
    pub use crate::collections::IndexSet;
    #[cfg(feature = "validate")]
    pub use crate::collections::{IndexMap, Map, Set};
}

/// A helper macro which is used to itself define further macros below.
///
/// This is a little complicated, so first off sorry about that. The idea here
/// though is that there's one source of truth for the listing of instructions
/// in `wasmparser` and this is the one location. All other locations should be
/// derivative from this. As this one source of truth it has all instructions
/// from all proposals all grouped together. Down below though, for compile
/// time, currently the simd instructions are split out into their own macro.
/// The structure/syntax of this macro is to facilitate easily splitting out
/// entire groups of instructions.
///
/// This is used below to define more macros.
macro_rules! _for_each_operator_group {
    ($mac:ident) => {
        $mac! {
            @mvp {
                Unreachable => visit_unreachable (arity 0 -> 0)
                Nop => visit_nop (arity 0 -> 0)
                Block { blockty: $crate::BlockType } => visit_block (arity custom)
                Loop { blockty: $crate::BlockType } => visit_loop (arity custom)
                If { blockty: $crate::BlockType } => visit_if (arity custom)
                Else => visit_else (arity custom)
                End => visit_end (arity custom)
                Br { relative_depth: u32 } => visit_br (arity custom)
                BrIf { relative_depth: u32 } => visit_br_if (arity custom)
                BrTable { targets: $crate::BrTable<'a> } => visit_br_table (arity custom)
                Return => visit_return (arity custom)
                Call { function_index: u32 } => visit_call (arity custom)
                CallIndirect { type_index: u32, table_index: u32 } => visit_call_indirect (arity custom)
                Drop => visit_drop (arity 1 -> 0)
                Select => visit_select (arity 3 -> 1)
                LocalGet { local_index: u32 } => visit_local_get (arity 0 -> 1)
                LocalSet { local_index: u32 } => visit_local_set (arity 1 -> 0)
                LocalTee { local_index: u32 } => visit_local_tee (arity 1 -> 1)
                GlobalGet { global_index: u32 } => visit_global_get (arity 0 -> 1)
                GlobalSet { global_index: u32 } => visit_global_set (arity 1 -> 0)
                I32Load { memarg: $crate::MemArg } => visit_i32_load (arity 1 -> 1)
                I64Load { memarg: $crate::MemArg } => visit_i64_load (arity 1 -> 1)
                F32Load { memarg: $crate::MemArg } => visit_f32_load (arity 1 -> 1)
                F64Load { memarg: $crate::MemArg } => visit_f64_load (arity 1 -> 1)
                I32Load8S { memarg: $crate::MemArg } => visit_i32_load8_s (arity 1 -> 1)
                I32Load8U { memarg: $crate::MemArg } => visit_i32_load8_u (arity 1 -> 1)
                I32Load16S { memarg: $crate::MemArg } => visit_i32_load16_s (arity 1 -> 1)
                I32Load16U { memarg: $crate::MemArg } => visit_i32_load16_u (arity 1 -> 1)
                I64Load8S { memarg: $crate::MemArg } => visit_i64_load8_s (arity 1 -> 1)
                I64Load8U { memarg: $crate::MemArg } => visit_i64_load8_u (arity 1 -> 1)
                I64Load16S { memarg: $crate::MemArg } => visit_i64_load16_s (arity 1 -> 1)
                I64Load16U { memarg: $crate::MemArg } => visit_i64_load16_u (arity 1 -> 1)
                I64Load32S { memarg: $crate::MemArg } => visit_i64_load32_s (arity 1 -> 1)
                I64Load32U { memarg: $crate::MemArg } => visit_i64_load32_u (arity 1 -> 1)
                I32Store { memarg: $crate::MemArg } => visit_i32_store (arity 2 -> 0)
                I64Store { memarg: $crate::MemArg } => visit_i64_store (arity 2 -> 0)
                F32Store { memarg: $crate::MemArg } => visit_f32_store (arity 2 -> 0)
                F64Store { memarg: $crate::MemArg } => visit_f64_store (arity 2 -> 0)
                I32Store8 { memarg: $crate::MemArg } => visit_i32_store8 (arity 2 -> 0)
                I32Store16 { memarg: $crate::MemArg } => visit_i32_store16 (arity 2 -> 0)
                I64Store8 { memarg: $crate::MemArg } => visit_i64_store8 (arity 2 -> 0)
                I64Store16 { memarg: $crate::MemArg } => visit_i64_store16 (arity 2 -> 0)
                I64Store32 { memarg: $crate::MemArg } => visit_i64_store32 (arity 2 -> 0)
                MemorySize { mem: u32 } => visit_memory_size (arity 0 -> 1)
                MemoryGrow { mem: u32 } => visit_memory_grow (arity 1 -> 1)
                I32Const { value: i32 } => visit_i32_const (arity 0 -> 1)
                I64Const { value: i64 } => visit_i64_const (arity 0 -> 1)
                F32Const { value: $crate::Ieee32 } => visit_f32_const (arity 0 -> 1)
                F64Const { value: $crate::Ieee64 } => visit_f64_const (arity 0 -> 1)
                I32Eqz => visit_i32_eqz (arity 1 -> 1)
                I32Eq => visit_i32_eq (arity 2 -> 1)
                I32Ne => visit_i32_ne (arity 2 -> 1)
                I32LtS => visit_i32_lt_s (arity 2 -> 1)
                I32LtU => visit_i32_lt_u (arity 2 -> 1)
                I32GtS => visit_i32_gt_s (arity 2 -> 1)
                I32GtU => visit_i32_gt_u (arity 2 -> 1)
                I32LeS => visit_i32_le_s (arity 2 -> 1)
                I32LeU => visit_i32_le_u (arity 2 -> 1)
                I32GeS => visit_i32_ge_s (arity 2 -> 1)
                I32GeU => visit_i32_ge_u (arity 2 -> 1)
                I64Eqz => visit_i64_eqz (arity 1 -> 1)
                I64Eq => visit_i64_eq (arity 2 -> 1)
                I64Ne => visit_i64_ne (arity 2 -> 1)
                I64LtS => visit_i64_lt_s (arity 2 -> 1)
                I64LtU => visit_i64_lt_u (arity 2 -> 1)
                I64GtS => visit_i64_gt_s (arity 2 -> 1)
                I64GtU => visit_i64_gt_u (arity 2 -> 1)
                I64LeS => visit_i64_le_s (arity 2 -> 1)
                I64LeU => visit_i64_le_u (arity 2 -> 1)
                I64GeS => visit_i64_ge_s (arity 2 -> 1)
                I64GeU => visit_i64_ge_u (arity 2 -> 1)
                F32Eq => visit_f32_eq (arity 2 -> 1)
                F32Ne => visit_f32_ne (arity 2 -> 1)
                F32Lt => visit_f32_lt (arity 2 -> 1)
                F32Gt => visit_f32_gt (arity 2 -> 1)
                F32Le => visit_f32_le (arity 2 -> 1)
                F32Ge => visit_f32_ge (arity 2 -> 1)
                F64Eq => visit_f64_eq (arity 2 -> 1)
                F64Ne => visit_f64_ne (arity 2 -> 1)
                F64Lt => visit_f64_lt (arity 2 -> 1)
                F64Gt => visit_f64_gt (arity 2 -> 1)
                F64Le => visit_f64_le (arity 2 -> 1)
                F64Ge => visit_f64_ge (arity 2 -> 1)
                I32Clz => visit_i32_clz (arity 1 -> 1)
                I32Ctz => visit_i32_ctz (arity 1 -> 1)
                I32Popcnt => visit_i32_popcnt (arity 1 -> 1)
                I32Add => visit_i32_add (arity 2 -> 1)
                I32Sub => visit_i32_sub (arity 2 -> 1)
                I32Mul => visit_i32_mul (arity 2 -> 1)
                I32DivS => visit_i32_div_s (arity 2 -> 1)
                I32DivU => visit_i32_div_u (arity 2 -> 1)
                I32RemS => visit_i32_rem_s (arity 2 -> 1)
                I32RemU => visit_i32_rem_u (arity 2 -> 1)
                I32And => visit_i32_and (arity 2 -> 1)
                I32Or => visit_i32_or (arity 2 -> 1)
                I32Xor => visit_i32_xor (arity 2 -> 1)
                I32Shl => visit_i32_shl (arity 2 -> 1)
                I32ShrS => visit_i32_shr_s (arity 2 -> 1)
                I32ShrU => visit_i32_shr_u (arity 2 -> 1)
                I32Rotl => visit_i32_rotl (arity 2 -> 1)
                I32Rotr => visit_i32_rotr (arity 2 -> 1)
                I64Clz => visit_i64_clz (arity 1 -> 1)
                I64Ctz => visit_i64_ctz (arity 1 -> 1)
                I64Popcnt => visit_i64_popcnt (arity 1 -> 1)
                I64Add => visit_i64_add (arity 2 -> 1)
                I64Sub => visit_i64_sub (arity 2 -> 1)
                I64Mul => visit_i64_mul (arity 2 -> 1)
                I64DivS => visit_i64_div_s (arity 2 -> 1)
                I64DivU => visit_i64_div_u (arity 2 -> 1)
                I64RemS => visit_i64_rem_s (arity 2 -> 1)
                I64RemU => visit_i64_rem_u (arity 2 -> 1)
                I64And => visit_i64_and (arity 2 -> 1)
                I64Or => visit_i64_or (arity 2 -> 1)
                I64Xor => visit_i64_xor (arity 2 -> 1)
                I64Shl => visit_i64_shl (arity 2 -> 1)
                I64ShrS => visit_i64_shr_s (arity 2 -> 1)
                I64ShrU => visit_i64_shr_u (arity 2 -> 1)
                I64Rotl => visit_i64_rotl (arity 2 -> 1)
                I64Rotr => visit_i64_rotr (arity 2 -> 1)
                F32Abs => visit_f32_abs (arity 1 -> 1)
                F32Neg => visit_f32_neg (arity 1 -> 1)
                F32Ceil => visit_f32_ceil (arity 1 -> 1)
                F32Floor => visit_f32_floor (arity 1 -> 1)
                F32Trunc => visit_f32_trunc (arity 1 -> 1)
                F32Nearest => visit_f32_nearest (arity 1 -> 1)
                F32Sqrt => visit_f32_sqrt (arity 1 -> 1)
                F32Add => visit_f32_add (arity 2 -> 1)
                F32Sub => visit_f32_sub (arity 2 -> 1)
                F32Mul => visit_f32_mul (arity 2 -> 1)
                F32Div => visit_f32_div (arity 2 -> 1)
                F32Min => visit_f32_min (arity 2 -> 1)
                F32Max => visit_f32_max (arity 2 -> 1)
                F32Copysign => visit_f32_copysign (arity 2 -> 1)
                F64Abs => visit_f64_abs (arity 1 -> 1)
                F64Neg => visit_f64_neg (arity 1 -> 1)
                F64Ceil => visit_f64_ceil (arity 1 -> 1)
                F64Floor => visit_f64_floor (arity 1 -> 1)
                F64Trunc => visit_f64_trunc (arity 1 -> 1)
                F64Nearest => visit_f64_nearest (arity 1 -> 1)
                F64Sqrt => visit_f64_sqrt (arity 1 -> 1)
                F64Add => visit_f64_add (arity 2 -> 1)
                F64Sub => visit_f64_sub (arity 2 -> 1)
                F64Mul => visit_f64_mul (arity 2 -> 1)
                F64Div => visit_f64_div (arity 2 -> 1)
                F64Min => visit_f64_min (arity 2 -> 1)
                F64Max => visit_f64_max (arity 2 -> 1)
                F64Copysign => visit_f64_copysign (arity 2 -> 1)
                I32WrapI64 => visit_i32_wrap_i64 (arity 1 -> 1)
                I32TruncF32S => visit_i32_trunc_f32_s (arity 1 -> 1)
                I32TruncF32U => visit_i32_trunc_f32_u (arity 1 -> 1)
                I32TruncF64S => visit_i32_trunc_f64_s (arity 1 -> 1)
                I32TruncF64U => visit_i32_trunc_f64_u (arity 1 -> 1)
                I64ExtendI32S => visit_i64_extend_i32_s (arity 1 -> 1)
                I64ExtendI32U => visit_i64_extend_i32_u (arity 1 -> 1)
                I64TruncF32S => visit_i64_trunc_f32_s (arity 1 -> 1)
                I64TruncF32U => visit_i64_trunc_f32_u (arity 1 -> 1)
                I64TruncF64S => visit_i64_trunc_f64_s (arity 1 -> 1)
                I64TruncF64U => visit_i64_trunc_f64_u (arity 1 -> 1)
                F32ConvertI32S => visit_f32_convert_i32_s (arity 1 -> 1)
                F32ConvertI32U => visit_f32_convert_i32_u (arity 1 -> 1)
                F32ConvertI64S => visit_f32_convert_i64_s (arity 1 -> 1)
                F32ConvertI64U => visit_f32_convert_i64_u (arity 1 -> 1)
                F32DemoteF64 => visit_f32_demote_f64 (arity 1 -> 1)
                F64ConvertI32S => visit_f64_convert_i32_s (arity 1 -> 1)
                F64ConvertI32U => visit_f64_convert_i32_u (arity 1 -> 1)
                F64ConvertI64S => visit_f64_convert_i64_s (arity 1 -> 1)
                F64ConvertI64U => visit_f64_convert_i64_u (arity 1 -> 1)
                F64PromoteF32 => visit_f64_promote_f32 (arity 1 -> 1)
                I32ReinterpretF32 => visit_i32_reinterpret_f32 (arity 1 -> 1)
                I64ReinterpretF64 => visit_i64_reinterpret_f64 (arity 1 -> 1)
                F32ReinterpretI32 => visit_f32_reinterpret_i32 (arity 1 -> 1)
                F64ReinterpretI64 => visit_f64_reinterpret_i64 (arity 1 -> 1)
            }

            @sign_extension {
                I32Extend8S => visit_i32_extend8_s (arity 1 -> 1)
                I32Extend16S => visit_i32_extend16_s (arity 1 -> 1)
                I64Extend8S => visit_i64_extend8_s (arity 1 -> 1)
                I64Extend16S => visit_i64_extend16_s (arity 1 -> 1)
                I64Extend32S => visit_i64_extend32_s (arity 1 -> 1)
            }

            // 0xFB prefixed operators
            // Garbage Collection
            // http://github.com/WebAssembly/gc
            @gc {
                RefEq => visit_ref_eq (arity 2 -> 1)
                StructNew { struct_type_index: u32 } => visit_struct_new (arity custom)
                StructNewDefault { struct_type_index: u32 } => visit_struct_new_default (arity 0 -> 1)
                StructGet { struct_type_index: u32, field_index: u32 } => visit_struct_get (arity 1 -> 1)
                StructGetS { struct_type_index: u32, field_index: u32 } => visit_struct_get_s (arity 1 -> 1)
                StructGetU { struct_type_index: u32, field_index: u32 } => visit_struct_get_u (arity 1 -> 1)
                StructSet { struct_type_index: u32, field_index: u32 } => visit_struct_set (arity 2 -> 0)
                ArrayNew { array_type_index: u32 } => visit_array_new (arity 2 -> 1)
                ArrayNewDefault { array_type_index: u32 } => visit_array_new_default (arity 1 -> 1)
                ArrayNewFixed { array_type_index: u32, array_size: u32 } => visit_array_new_fixed (arity custom)
                ArrayNewData { array_type_index: u32, array_data_index: u32 } => visit_array_new_data (arity 2 -> 1)
                ArrayNewElem { array_type_index: u32, array_elem_index: u32 } => visit_array_new_elem (arity 2 -> 1)
                ArrayGet { array_type_index: u32 } => visit_array_get (arity 2 -> 1)
                ArrayGetS { array_type_index: u32 } => visit_array_get_s (arity 2 -> 1)
                ArrayGetU { array_type_index: u32 } => visit_array_get_u (arity 2 -> 1)
                ArraySet { array_type_index: u32 } => visit_array_set (arity 3 -> 0)
                ArrayLen => visit_array_len (arity 1 -> 1)
                ArrayFill { array_type_index: u32 } => visit_array_fill (arity 4 -> 0)
                ArrayCopy { array_type_index_dst: u32, array_type_index_src: u32 } => visit_array_copy (arity 5 -> 0)
                ArrayInitData { array_type_index: u32, array_data_index: u32 } => visit_array_init_data (arity 4 -> 0)
                ArrayInitElem { array_type_index: u32, array_elem_index: u32 } => visit_array_init_elem (arity 4 -> 0)
                RefTestNonNull { hty: $crate::HeapType } => visit_ref_test_non_null (arity 1 -> 1)
                RefTestNullable { hty: $crate::HeapType } => visit_ref_test_nullable (arity 1 -> 1)
                RefCastNonNull { hty: $crate::HeapType } => visit_ref_cast_non_null (arity 1 -> 1)
                RefCastNullable { hty: $crate::HeapType } => visit_ref_cast_nullable (arity 1 -> 1)
                BrOnCast {
                    relative_depth: u32,
                    from_ref_type: $crate::RefType,
                    to_ref_type: $crate::RefType
                } => visit_br_on_cast (arity custom)
                BrOnCastFail {
                    relative_depth: u32,
                    from_ref_type: $crate::RefType,
                    to_ref_type: $crate::RefType
                } => visit_br_on_cast_fail (arity custom)
                AnyConvertExtern => visit_any_convert_extern  (arity 1 -> 1)
                ExternConvertAny => visit_extern_convert_any (arity 1 -> 1)
                RefI31 => visit_ref_i31 (arity 1 -> 1)
                I31GetS => visit_i31_get_s (arity 1 -> 1)
                I31GetU => visit_i31_get_u (arity 1 -> 1)
            }

            // 0xFC operators
            // Non-trapping Float-to-int Conversions
            // https://github.com/WebAssembly/nontrapping-float-to-int-conversions
            @saturating_float_to_int {
                I32TruncSatF32S => visit_i32_trunc_sat_f32_s (arity 1 -> 1)
                I32TruncSatF32U => visit_i32_trunc_sat_f32_u (arity 1 -> 1)
                I32TruncSatF64S => visit_i32_trunc_sat_f64_s (arity 1 -> 1)
                I32TruncSatF64U => visit_i32_trunc_sat_f64_u (arity 1 -> 1)
                I64TruncSatF32S => visit_i64_trunc_sat_f32_s (arity 1 -> 1)
                I64TruncSatF32U => visit_i64_trunc_sat_f32_u (arity 1 -> 1)
                I64TruncSatF64S => visit_i64_trunc_sat_f64_s (arity 1 -> 1)
                I64TruncSatF64U => visit_i64_trunc_sat_f64_u (arity 1 -> 1)
            }

            // 0xFC prefixed operators
            // bulk memory operations
            // https://github.com/WebAssembly/bulk-memory-operations
            @bulk_memory {
                MemoryInit { data_index: u32, mem: u32 } => visit_memory_init (arity 3 -> 0)
                DataDrop { data_index: u32 } => visit_data_drop (arity 0 -> 0)
                MemoryCopy { dst_mem: u32, src_mem: u32 } => visit_memory_copy (arity 3 -> 0)
                MemoryFill { mem: u32 } => visit_memory_fill (arity 3 -> 0)
                TableInit { elem_index: u32, table: u32 } => visit_table_init (arity 3 -> 0)
                ElemDrop { elem_index: u32 } => visit_elem_drop (arity 0 -> 0)
                TableCopy { dst_table: u32, src_table: u32 } => visit_table_copy (arity 3 -> 0)
            }

            // 0xFC prefixed operators
            // reference-types
            // https://github.com/WebAssembly/reference-types
            @reference_types {
                TypedSelect { ty: $crate::ValType } => visit_typed_select (arity 3 -> 1)
                // multi-result select is currently invalid, but can be parsed and printed
                TypedSelectMulti { tys: Vec<$crate::ValType> } => visit_typed_select_multi (arity custom)
                RefNull { hty: $crate::HeapType } => visit_ref_null (arity 0 -> 1)
                RefIsNull => visit_ref_is_null (arity 1 -> 1)
                RefFunc { function_index: u32 } => visit_ref_func (arity 0 -> 1)
                TableFill { table: u32 } => visit_table_fill (arity 3 -> 0)
                TableGet { table: u32 } => visit_table_get (arity 1 -> 1)
                TableSet { table: u32 } => visit_table_set (arity 2 -> 0)
                TableGrow { table: u32 } => visit_table_grow (arity 2 -> 1)
                TableSize { table: u32 } => visit_table_size (arity 0 -> 1)
            }

            // Wasm tail-call proposal
            // https://github.com/WebAssembly/tail-call
            @tail_call {
                ReturnCall { function_index: u32 } => visit_return_call (arity custom)
                ReturnCallIndirect { type_index: u32, table_index: u32 } => visit_return_call_indirect (arity custom)
            }

            // OxFC prefixed operators
            // memory control (experimental)
            // https://github.com/WebAssembly/design/issues/1439
            @memory_control {
                MemoryDiscard { mem: u32 } => visit_memory_discard (arity 2 -> 0)
            }

            // 0xFE prefixed operators
            // threads
            // https://github.com/WebAssembly/threads
            @threads {
                MemoryAtomicNotify { memarg: $crate::MemArg } => visit_memory_atomic_notify (arity 2 -> 1)
                MemoryAtomicWait32 { memarg: $crate::MemArg } => visit_memory_atomic_wait32 (arity 3 -> 1)
                MemoryAtomicWait64 { memarg: $crate::MemArg } => visit_memory_atomic_wait64 (arity 3 -> 1)
                AtomicFence => visit_atomic_fence (arity 0 -> 0)
                I32AtomicLoad { memarg: $crate::MemArg } => visit_i32_atomic_load (arity 1 -> 1)
                I64AtomicLoad { memarg: $crate::MemArg } => visit_i64_atomic_load (arity 1 -> 1)
                I32AtomicLoad8U { memarg: $crate::MemArg } => visit_i32_atomic_load8_u (arity 1 -> 1)
                I32AtomicLoad16U { memarg: $crate::MemArg } => visit_i32_atomic_load16_u (arity 1 -> 1)
                I64AtomicLoad8U { memarg: $crate::MemArg } => visit_i64_atomic_load8_u (arity 1 -> 1)
                I64AtomicLoad16U { memarg: $crate::MemArg } => visit_i64_atomic_load16_u (arity 1 -> 1)
                I64AtomicLoad32U { memarg: $crate::MemArg } => visit_i64_atomic_load32_u (arity 1 -> 1)
                I32AtomicStore { memarg: $crate::MemArg } => visit_i32_atomic_store (arity 2 -> 0)
                I64AtomicStore { memarg: $crate::MemArg } => visit_i64_atomic_store (arity 2 -> 0)
                I32AtomicStore8 { memarg: $crate::MemArg } => visit_i32_atomic_store8 (arity 2 -> 0)
                I32AtomicStore16 { memarg: $crate::MemArg } => visit_i32_atomic_store16 (arity 2 -> 0)
                I64AtomicStore8 { memarg: $crate::MemArg } => visit_i64_atomic_store8 (arity 2 -> 0)
                I64AtomicStore16 { memarg: $crate::MemArg } => visit_i64_atomic_store16 (arity 2 -> 0)
                I64AtomicStore32 { memarg: $crate::MemArg } => visit_i64_atomic_store32 (arity 2 -> 0)
                I32AtomicRmwAdd { memarg: $crate::MemArg } => visit_i32_atomic_rmw_add (arity 2 -> 1)
                I64AtomicRmwAdd { memarg: $crate::MemArg } => visit_i64_atomic_rmw_add (arity 2 -> 1)
                I32AtomicRmw8AddU { memarg: $crate::MemArg } => visit_i32_atomic_rmw8_add_u (arity 2 -> 1)
                I32AtomicRmw16AddU { memarg: $crate::MemArg } => visit_i32_atomic_rmw16_add_u (arity 2 -> 1)
                I64AtomicRmw8AddU { memarg: $crate::MemArg } => visit_i64_atomic_rmw8_add_u (arity 2 -> 1)
                I64AtomicRmw16AddU { memarg: $crate::MemArg } => visit_i64_atomic_rmw16_add_u (arity 2 -> 1)
                I64AtomicRmw32AddU { memarg: $crate::MemArg } => visit_i64_atomic_rmw32_add_u (arity 2 -> 1)
                I32AtomicRmwSub { memarg: $crate::MemArg } => visit_i32_atomic_rmw_sub (arity 2 -> 1)
                I64AtomicRmwSub { memarg: $crate::MemArg } => visit_i64_atomic_rmw_sub (arity 2 -> 1)
                I32AtomicRmw8SubU { memarg: $crate::MemArg } => visit_i32_atomic_rmw8_sub_u (arity 2 -> 1)
                I32AtomicRmw16SubU { memarg: $crate::MemArg } => visit_i32_atomic_rmw16_sub_u (arity 2 -> 1)
                I64AtomicRmw8SubU { memarg: $crate::MemArg } => visit_i64_atomic_rmw8_sub_u (arity 2 -> 1)
                I64AtomicRmw16SubU { memarg: $crate::MemArg } => visit_i64_atomic_rmw16_sub_u (arity 2 -> 1)
                I64AtomicRmw32SubU { memarg: $crate::MemArg } => visit_i64_atomic_rmw32_sub_u (arity 2 -> 1)
                I32AtomicRmwAnd { memarg: $crate::MemArg } => visit_i32_atomic_rmw_and (arity 2 -> 1)
                I64AtomicRmwAnd { memarg: $crate::MemArg } => visit_i64_atomic_rmw_and (arity 2 -> 1)
                I32AtomicRmw8AndU { memarg: $crate::MemArg } => visit_i32_atomic_rmw8_and_u (arity 2 -> 1)
                I32AtomicRmw16AndU { memarg: $crate::MemArg } => visit_i32_atomic_rmw16_and_u (arity 2 -> 1)
                I64AtomicRmw8AndU { memarg: $crate::MemArg } => visit_i64_atomic_rmw8_and_u (arity 2 -> 1)
                I64AtomicRmw16AndU { memarg: $crate::MemArg } => visit_i64_atomic_rmw16_and_u (arity 2 -> 1)
                I64AtomicRmw32AndU { memarg: $crate::MemArg } => visit_i64_atomic_rmw32_and_u (arity 2 -> 1)
                I32AtomicRmwOr { memarg: $crate::MemArg } => visit_i32_atomic_rmw_or (arity 2 -> 1)
                I64AtomicRmwOr { memarg: $crate::MemArg } => visit_i64_atomic_rmw_or (arity 2 -> 1)
                I32AtomicRmw8OrU { memarg: $crate::MemArg } => visit_i32_atomic_rmw8_or_u (arity 2 -> 1)
                I32AtomicRmw16OrU { memarg: $crate::MemArg } => visit_i32_atomic_rmw16_or_u (arity 2 -> 1)
                I64AtomicRmw8OrU { memarg: $crate::MemArg } => visit_i64_atomic_rmw8_or_u (arity 2 -> 1)
                I64AtomicRmw16OrU { memarg: $crate::MemArg } => visit_i64_atomic_rmw16_or_u (arity 2 -> 1)
                I64AtomicRmw32OrU { memarg: $crate::MemArg } => visit_i64_atomic_rmw32_or_u (arity 2 -> 1)
                I32AtomicRmwXor { memarg: $crate::MemArg } => visit_i32_atomic_rmw_xor (arity 2 -> 1)
                I64AtomicRmwXor { memarg: $crate::MemArg } => visit_i64_atomic_rmw_xor (arity 2 -> 1)
                I32AtomicRmw8XorU { memarg: $crate::MemArg } => visit_i32_atomic_rmw8_xor_u (arity 2 -> 1)
                I32AtomicRmw16XorU { memarg: $crate::MemArg } => visit_i32_atomic_rmw16_xor_u (arity 2 -> 1)
                I64AtomicRmw8XorU { memarg: $crate::MemArg } => visit_i64_atomic_rmw8_xor_u (arity 2 -> 1)
                I64AtomicRmw16XorU { memarg: $crate::MemArg } => visit_i64_atomic_rmw16_xor_u (arity 2 -> 1)
                I64AtomicRmw32XorU { memarg: $crate::MemArg } => visit_i64_atomic_rmw32_xor_u (arity 2 -> 1)
                I32AtomicRmwXchg { memarg: $crate::MemArg } => visit_i32_atomic_rmw_xchg (arity 2 -> 1)
                I64AtomicRmwXchg { memarg: $crate::MemArg } => visit_i64_atomic_rmw_xchg (arity 2 -> 1)
                I32AtomicRmw8XchgU { memarg: $crate::MemArg } => visit_i32_atomic_rmw8_xchg_u (arity 2 -> 1)
                I32AtomicRmw16XchgU { memarg: $crate::MemArg } => visit_i32_atomic_rmw16_xchg_u (arity 2 -> 1)
                I64AtomicRmw8XchgU { memarg: $crate::MemArg } => visit_i64_atomic_rmw8_xchg_u (arity 2 -> 1)
                I64AtomicRmw16XchgU { memarg: $crate::MemArg } => visit_i64_atomic_rmw16_xchg_u (arity 2 -> 1)
                I64AtomicRmw32XchgU { memarg: $crate::MemArg } => visit_i64_atomic_rmw32_xchg_u (arity 2 -> 1)
                I32AtomicRmwCmpxchg { memarg: $crate::MemArg } => visit_i32_atomic_rmw_cmpxchg (arity 3 -> 1)
                I64AtomicRmwCmpxchg { memarg: $crate::MemArg } => visit_i64_atomic_rmw_cmpxchg (arity 3 -> 1)
                I32AtomicRmw8CmpxchgU { memarg: $crate::MemArg } => visit_i32_atomic_rmw8_cmpxchg_u (arity 3 -> 1)
                I32AtomicRmw16CmpxchgU { memarg: $crate::MemArg } => visit_i32_atomic_rmw16_cmpxchg_u (arity 3 -> 1)
                I64AtomicRmw8CmpxchgU { memarg: $crate::MemArg } => visit_i64_atomic_rmw8_cmpxchg_u (arity 3 -> 1)
                I64AtomicRmw16CmpxchgU { memarg: $crate::MemArg } => visit_i64_atomic_rmw16_cmpxchg_u (arity 3 -> 1)
                I64AtomicRmw32CmpxchgU { memarg: $crate::MemArg } => visit_i64_atomic_rmw32_cmpxchg_u (arity 3 -> 1)
            }

            // 0xFD operators
            // 128-bit SIMD
            // - https://github.com/webassembly/simd
            // - https://webassembly.github.io/simd/core/binary/instructions.html
            @simd {
                V128Load { memarg: $crate::MemArg } => visit_v128_load (arity 1 -> 1)
                V128Load8x8S { memarg: $crate::MemArg } => visit_v128_load8x8_s (arity 1 -> 1)
                V128Load8x8U { memarg: $crate::MemArg } => visit_v128_load8x8_u (arity 1 -> 1)
                V128Load16x4S { memarg: $crate::MemArg } => visit_v128_load16x4_s (arity 1 -> 1)
                V128Load16x4U { memarg: $crate::MemArg } => visit_v128_load16x4_u (arity 1 -> 1)
                V128Load32x2S { memarg: $crate::MemArg } => visit_v128_load32x2_s (arity 1 -> 1)
                V128Load32x2U { memarg: $crate::MemArg } => visit_v128_load32x2_u (arity 1 -> 1)
                V128Load8Splat { memarg: $crate::MemArg } => visit_v128_load8_splat (arity 1 -> 1)
                V128Load16Splat { memarg: $crate::MemArg } => visit_v128_load16_splat (arity 1 -> 1)
                V128Load32Splat { memarg: $crate::MemArg } => visit_v128_load32_splat (arity 1 -> 1)
                V128Load64Splat { memarg: $crate::MemArg } => visit_v128_load64_splat (arity 1 -> 1)
                V128Load32Zero { memarg: $crate::MemArg } => visit_v128_load32_zero (arity 1 -> 1)
                V128Load64Zero { memarg: $crate::MemArg } => visit_v128_load64_zero (arity 1 -> 1)
                V128Store { memarg: $crate::MemArg } => visit_v128_store (arity 2 -> 0)
                V128Load8Lane { memarg: $crate::MemArg, lane: u8 } => visit_v128_load8_lane (arity 2 -> 1)
                V128Load16Lane { memarg: $crate::MemArg, lane: u8 } => visit_v128_load16_lane (arity 2 -> 1)
                V128Load32Lane { memarg: $crate::MemArg, lane: u8 } => visit_v128_load32_lane (arity 2 -> 1)
                V128Load64Lane { memarg: $crate::MemArg, lane: u8 } => visit_v128_load64_lane (arity 2 -> 1)
                V128Store8Lane { memarg: $crate::MemArg, lane: u8 } => visit_v128_store8_lane (arity 2 -> 0)
                V128Store16Lane { memarg: $crate::MemArg, lane: u8 } => visit_v128_store16_lane (arity 2 -> 0)
                V128Store32Lane { memarg: $crate::MemArg, lane: u8 } => visit_v128_store32_lane (arity 2 -> 0)
                V128Store64Lane { memarg: $crate::MemArg, lane: u8 } => visit_v128_store64_lane (arity 2 -> 0)
                V128Const { value: $crate::V128 } => visit_v128_const (arity 0 -> 1)
                I8x16Shuffle { lanes: [u8; 16] } => visit_i8x16_shuffle (arity 2 -> 1)
                I8x16ExtractLaneS { lane: u8 } => visit_i8x16_extract_lane_s (arity 1 -> 1)
                I8x16ExtractLaneU { lane: u8 } => visit_i8x16_extract_lane_u (arity 1 -> 1)
                I8x16ReplaceLane { lane: u8 } => visit_i8x16_replace_lane (arity 2 -> 1)
                I16x8ExtractLaneS { lane: u8 } => visit_i16x8_extract_lane_s (arity 1 -> 1)
                I16x8ExtractLaneU { lane: u8 } => visit_i16x8_extract_lane_u (arity 1 -> 1)
                I16x8ReplaceLane { lane: u8 } => visit_i16x8_replace_lane (arity 2 -> 1)
                I32x4ExtractLane { lane: u8 } => visit_i32x4_extract_lane (arity 1 -> 1)
                I32x4ReplaceLane { lane: u8 } => visit_i32x4_replace_lane (arity 2 -> 1)
                I64x2ExtractLane { lane: u8 } => visit_i64x2_extract_lane (arity 1 -> 1)
                I64x2ReplaceLane { lane: u8 } => visit_i64x2_replace_lane (arity 2 -> 1)
                F32x4ExtractLane { lane: u8 } => visit_f32x4_extract_lane (arity 1 -> 1)
                F32x4ReplaceLane { lane: u8 } => visit_f32x4_replace_lane (arity 2 -> 1)
                F64x2ExtractLane { lane: u8 } => visit_f64x2_extract_lane (arity 1 -> 1)
                F64x2ReplaceLane { lane: u8 } => visit_f64x2_replace_lane (arity 2 -> 1)
                I8x16Swizzle => visit_i8x16_swizzle (arity 2 -> 1)
                I8x16Splat => visit_i8x16_splat (arity 1 -> 1)
                I16x8Splat => visit_i16x8_splat (arity 1 -> 1)
                I32x4Splat => visit_i32x4_splat (arity 1 -> 1)
                I64x2Splat => visit_i64x2_splat (arity 1 -> 1)
                F32x4Splat => visit_f32x4_splat (arity 1 -> 1)
                F64x2Splat => visit_f64x2_splat (arity 1 -> 1)
                I8x16Eq => visit_i8x16_eq (arity 2 -> 1)
                I8x16Ne => visit_i8x16_ne (arity 2 -> 1)
                I8x16LtS => visit_i8x16_lt_s (arity 2 -> 1)
                I8x16LtU => visit_i8x16_lt_u (arity 2 -> 1)
                I8x16GtS => visit_i8x16_gt_s (arity 2 -> 1)
                I8x16GtU => visit_i8x16_gt_u (arity 2 -> 1)
                I8x16LeS => visit_i8x16_le_s (arity 2 -> 1)
                I8x16LeU => visit_i8x16_le_u (arity 2 -> 1)
                I8x16GeS => visit_i8x16_ge_s (arity 2 -> 1)
                I8x16GeU => visit_i8x16_ge_u (arity 2 -> 1)
                I16x8Eq => visit_i16x8_eq (arity 2 -> 1)
                I16x8Ne => visit_i16x8_ne (arity 2 -> 1)
                I16x8LtS => visit_i16x8_lt_s (arity 2 -> 1)
                I16x8LtU => visit_i16x8_lt_u (arity 2 -> 1)
                I16x8GtS => visit_i16x8_gt_s (arity 2 -> 1)
                I16x8GtU => visit_i16x8_gt_u (arity 2 -> 1)
                I16x8LeS => visit_i16x8_le_s (arity 2 -> 1)
                I16x8LeU => visit_i16x8_le_u (arity 2 -> 1)
                I16x8GeS => visit_i16x8_ge_s (arity 2 -> 1)
                I16x8GeU => visit_i16x8_ge_u (arity 2 -> 1)
                I32x4Eq => visit_i32x4_eq (arity 2 -> 1)
                I32x4Ne => visit_i32x4_ne (arity 2 -> 1)
                I32x4LtS => visit_i32x4_lt_s (arity 2 -> 1)
                I32x4LtU => visit_i32x4_lt_u (arity 2 -> 1)
                I32x4GtS => visit_i32x4_gt_s (arity 2 -> 1)
                I32x4GtU => visit_i32x4_gt_u (arity 2 -> 1)
                I32x4LeS => visit_i32x4_le_s (arity 2 -> 1)
                I32x4LeU => visit_i32x4_le_u (arity 2 -> 1)
                I32x4GeS => visit_i32x4_ge_s (arity 2 -> 1)
                I32x4GeU => visit_i32x4_ge_u (arity 2 -> 1)
                I64x2Eq => visit_i64x2_eq (arity 2 -> 1)
                I64x2Ne => visit_i64x2_ne (arity 2 -> 1)
                I64x2LtS => visit_i64x2_lt_s (arity 2 -> 1)
                I64x2GtS => visit_i64x2_gt_s (arity 2 -> 1)
                I64x2LeS => visit_i64x2_le_s (arity 2 -> 1)
                I64x2GeS => visit_i64x2_ge_s (arity 2 -> 1)
                F32x4Eq => visit_f32x4_eq (arity 2 -> 1)
                F32x4Ne => visit_f32x4_ne (arity 2 -> 1)
                F32x4Lt => visit_f32x4_lt (arity 2 -> 1)
                F32x4Gt => visit_f32x4_gt (arity 2 -> 1)
                F32x4Le => visit_f32x4_le (arity 2 -> 1)
                F32x4Ge => visit_f32x4_ge (arity 2 -> 1)
                F64x2Eq => visit_f64x2_eq (arity 2 -> 1)
                F64x2Ne => visit_f64x2_ne (arity 2 -> 1)
                F64x2Lt => visit_f64x2_lt (arity 2 -> 1)
                F64x2Gt => visit_f64x2_gt (arity 2 -> 1)
                F64x2Le => visit_f64x2_le (arity 2 -> 1)
                F64x2Ge => visit_f64x2_ge (arity 2 -> 1)
                V128Not => visit_v128_not (arity 1 -> 1)
                V128And => visit_v128_and (arity 2 -> 1)
                V128AndNot => visit_v128_andnot (arity 2 -> 1)
                V128Or => visit_v128_or (arity 2 -> 1)
                V128Xor => visit_v128_xor (arity 2 -> 1)
                V128Bitselect => visit_v128_bitselect (arity 3 -> 1)
                V128AnyTrue => visit_v128_any_true (arity 1 -> 1)
                I8x16Abs => visit_i8x16_abs (arity 1 -> 1)
                I8x16Neg => visit_i8x16_neg (arity 1 -> 1)
                I8x16Popcnt => visit_i8x16_popcnt (arity 1 -> 1)
                I8x16AllTrue => visit_i8x16_all_true (arity 1 -> 1)
                I8x16Bitmask => visit_i8x16_bitmask (arity 1 -> 1)
                I8x16NarrowI16x8S => visit_i8x16_narrow_i16x8_s (arity 2 -> 1)
                I8x16NarrowI16x8U => visit_i8x16_narrow_i16x8_u (arity 2 -> 1)
                I8x16Shl => visit_i8x16_shl (arity 2 -> 1)
                I8x16ShrS => visit_i8x16_shr_s (arity 2 -> 1)
                I8x16ShrU => visit_i8x16_shr_u (arity 2 -> 1)
                I8x16Add => visit_i8x16_add (arity 2 -> 1)
                I8x16AddSatS => visit_i8x16_add_sat_s (arity 2 -> 1)
                I8x16AddSatU => visit_i8x16_add_sat_u (arity 2 -> 1)
                I8x16Sub => visit_i8x16_sub (arity 2 -> 1)
                I8x16SubSatS => visit_i8x16_sub_sat_s (arity 2 -> 1)
                I8x16SubSatU => visit_i8x16_sub_sat_u (arity 2 -> 1)
                I8x16MinS => visit_i8x16_min_s (arity 2 -> 1)
                I8x16MinU => visit_i8x16_min_u (arity 2 -> 1)
                I8x16MaxS => visit_i8x16_max_s (arity 2 -> 1)
                I8x16MaxU => visit_i8x16_max_u (arity 2 -> 1)
                I8x16AvgrU => visit_i8x16_avgr_u (arity 2 -> 1)
                I16x8ExtAddPairwiseI8x16S => visit_i16x8_extadd_pairwise_i8x16_s (arity 1 -> 1)
                I16x8ExtAddPairwiseI8x16U => visit_i16x8_extadd_pairwise_i8x16_u (arity 1 -> 1)
                I16x8Abs => visit_i16x8_abs (arity 1 -> 1)
                I16x8Neg => visit_i16x8_neg (arity 1 -> 1)
                I16x8Q15MulrSatS => visit_i16x8_q15mulr_sat_s (arity 2 -> 1)
                I16x8AllTrue => visit_i16x8_all_true (arity 1 -> 1)
                I16x8Bitmask => visit_i16x8_bitmask (arity 1 -> 1)
                I16x8NarrowI32x4S => visit_i16x8_narrow_i32x4_s (arity 2 -> 1)
                I16x8NarrowI32x4U => visit_i16x8_narrow_i32x4_u (arity 2 -> 1)
                I16x8ExtendLowI8x16S => visit_i16x8_extend_low_i8x16_s (arity 1 -> 1)
                I16x8ExtendHighI8x16S => visit_i16x8_extend_high_i8x16_s (arity 1 -> 1)
                I16x8ExtendLowI8x16U => visit_i16x8_extend_low_i8x16_u (arity 1 -> 1)
                I16x8ExtendHighI8x16U => visit_i16x8_extend_high_i8x16_u (arity 1 -> 1)
                I16x8Shl => visit_i16x8_shl (arity 2 -> 1)
                I16x8ShrS => visit_i16x8_shr_s (arity 2 -> 1)
                I16x8ShrU => visit_i16x8_shr_u (arity 2 -> 1)
                I16x8Add => visit_i16x8_add (arity 2 -> 1)
                I16x8AddSatS => visit_i16x8_add_sat_s (arity 2 -> 1)
                I16x8AddSatU => visit_i16x8_add_sat_u (arity 2 -> 1)
                I16x8Sub => visit_i16x8_sub (arity 2 -> 1)
                I16x8SubSatS => visit_i16x8_sub_sat_s (arity 2 -> 1)
                I16x8SubSatU => visit_i16x8_sub_sat_u (arity 2 -> 1)
                I16x8Mul => visit_i16x8_mul (arity 2 -> 1)
                I16x8MinS => visit_i16x8_min_s (arity 2 -> 1)
                I16x8MinU => visit_i16x8_min_u (arity 2 -> 1)
                I16x8MaxS => visit_i16x8_max_s (arity 2 -> 1)
                I16x8MaxU => visit_i16x8_max_u (arity 2 -> 1)
                I16x8AvgrU => visit_i16x8_avgr_u (arity 2 -> 1)
                I16x8ExtMulLowI8x16S => visit_i16x8_extmul_low_i8x16_s (arity 2 -> 1)
                I16x8ExtMulHighI8x16S => visit_i16x8_extmul_high_i8x16_s (arity 2 -> 1)
                I16x8ExtMulLowI8x16U => visit_i16x8_extmul_low_i8x16_u (arity 2 -> 1)
                I16x8ExtMulHighI8x16U => visit_i16x8_extmul_high_i8x16_u (arity 2 -> 1)
                I32x4ExtAddPairwiseI16x8S => visit_i32x4_extadd_pairwise_i16x8_s (arity 1 -> 1)
                I32x4ExtAddPairwiseI16x8U => visit_i32x4_extadd_pairwise_i16x8_u (arity 1 -> 1)
                I32x4Abs => visit_i32x4_abs (arity 1 -> 1)
                I32x4Neg => visit_i32x4_neg (arity 1 -> 1)
                I32x4AllTrue => visit_i32x4_all_true (arity 1 -> 1)
                I32x4Bitmask => visit_i32x4_bitmask (arity 1 -> 1)
                I32x4ExtendLowI16x8S => visit_i32x4_extend_low_i16x8_s (arity 1 -> 1)
                I32x4ExtendHighI16x8S => visit_i32x4_extend_high_i16x8_s (arity 1 -> 1)
                I32x4ExtendLowI16x8U => visit_i32x4_extend_low_i16x8_u (arity 1 -> 1)
                I32x4ExtendHighI16x8U => visit_i32x4_extend_high_i16x8_u (arity 1 -> 1)
                I32x4Shl => visit_i32x4_shl (arity 2 -> 1)
                I32x4ShrS => visit_i32x4_shr_s (arity 2 -> 1)
                I32x4ShrU => visit_i32x4_shr_u (arity 2 -> 1)
                I32x4Add => visit_i32x4_add (arity 2 -> 1)
                I32x4Sub => visit_i32x4_sub (arity 2 -> 1)
                I32x4Mul => visit_i32x4_mul (arity 2 -> 1)
                I32x4MinS => visit_i32x4_min_s (arity 2 -> 1)
                I32x4MinU => visit_i32x4_min_u (arity 2 -> 1)
                I32x4MaxS => visit_i32x4_max_s (arity 2 -> 1)
                I32x4MaxU => visit_i32x4_max_u (arity 2 -> 1)
                I32x4DotI16x8S => visit_i32x4_dot_i16x8_s (arity 2 -> 1)
                I32x4ExtMulLowI16x8S => visit_i32x4_extmul_low_i16x8_s (arity 2 -> 1)
                I32x4ExtMulHighI16x8S => visit_i32x4_extmul_high_i16x8_s (arity 2 -> 1)
                I32x4ExtMulLowI16x8U => visit_i32x4_extmul_low_i16x8_u (arity 2 -> 1)
                I32x4ExtMulHighI16x8U => visit_i32x4_extmul_high_i16x8_u (arity 2 -> 1)
                I64x2Abs => visit_i64x2_abs (arity 1 -> 1)
                I64x2Neg => visit_i64x2_neg (arity 1 -> 1)
                I64x2AllTrue => visit_i64x2_all_true (arity 1 -> 1)
                I64x2Bitmask => visit_i64x2_bitmask (arity 1 -> 1)
                I64x2ExtendLowI32x4S => visit_i64x2_extend_low_i32x4_s (arity 1 -> 1)
                I64x2ExtendHighI32x4S => visit_i64x2_extend_high_i32x4_s (arity 1 -> 1)
                I64x2ExtendLowI32x4U => visit_i64x2_extend_low_i32x4_u (arity 1 -> 1)
                I64x2ExtendHighI32x4U => visit_i64x2_extend_high_i32x4_u (arity 1 -> 1)
                I64x2Shl => visit_i64x2_shl (arity 2 -> 1)
                I64x2ShrS => visit_i64x2_shr_s (arity 2 -> 1)
                I64x2ShrU => visit_i64x2_shr_u (arity 2 -> 1)
                I64x2Add => visit_i64x2_add (arity 2 -> 1)
                I64x2Sub => visit_i64x2_sub (arity 2 -> 1)
                I64x2Mul => visit_i64x2_mul (arity 2 -> 1)
                I64x2ExtMulLowI32x4S => visit_i64x2_extmul_low_i32x4_s (arity 2 -> 1)
                I64x2ExtMulHighI32x4S => visit_i64x2_extmul_high_i32x4_s (arity 2 -> 1)
                I64x2ExtMulLowI32x4U => visit_i64x2_extmul_low_i32x4_u (arity 2 -> 1)
                I64x2ExtMulHighI32x4U => visit_i64x2_extmul_high_i32x4_u (arity 2 -> 1)
                F32x4Ceil => visit_f32x4_ceil (arity 1 -> 1)
                F32x4Floor => visit_f32x4_floor (arity 1 -> 1)
                F32x4Trunc => visit_f32x4_trunc (arity 1 -> 1)
                F32x4Nearest => visit_f32x4_nearest (arity 1 -> 1)
                F32x4Abs => visit_f32x4_abs (arity 1 -> 1)
                F32x4Neg => visit_f32x4_neg (arity 1 -> 1)
                F32x4Sqrt => visit_f32x4_sqrt (arity 1 -> 1)
                F32x4Add => visit_f32x4_add (arity 2 -> 1)
                F32x4Sub => visit_f32x4_sub (arity 2 -> 1)
                F32x4Mul => visit_f32x4_mul (arity 2 -> 1)
                F32x4Div => visit_f32x4_div (arity 2 -> 1)
                F32x4Min => visit_f32x4_min (arity 2 -> 1)
                F32x4Max => visit_f32x4_max (arity 2 -> 1)
                F32x4PMin => visit_f32x4_pmin (arity 2 -> 1)
                F32x4PMax => visit_f32x4_pmax (arity 2 -> 1)
                F64x2Ceil => visit_f64x2_ceil (arity 1 -> 1)
                F64x2Floor => visit_f64x2_floor (arity 1 -> 1)
                F64x2Trunc => visit_f64x2_trunc (arity 1 -> 1)
                F64x2Nearest => visit_f64x2_nearest (arity 1 -> 1)
                F64x2Abs => visit_f64x2_abs (arity 1 -> 1)
                F64x2Neg => visit_f64x2_neg (arity 1 -> 1)
                F64x2Sqrt => visit_f64x2_sqrt (arity 1 -> 1)
                F64x2Add => visit_f64x2_add (arity 2 -> 1)
                F64x2Sub => visit_f64x2_sub (arity 2 -> 1)
                F64x2Mul => visit_f64x2_mul (arity 2 -> 1)
                F64x2Div => visit_f64x2_div (arity 2 -> 1)
                F64x2Min => visit_f64x2_min (arity 2 -> 1)
                F64x2Max => visit_f64x2_max (arity 2 -> 1)
                F64x2PMin => visit_f64x2_pmin (arity 2 -> 1)
                F64x2PMax => visit_f64x2_pmax (arity 2 -> 1)
                I32x4TruncSatF32x4S => visit_i32x4_trunc_sat_f32x4_s (arity 1 -> 1)
                I32x4TruncSatF32x4U => visit_i32x4_trunc_sat_f32x4_u (arity 1 -> 1)
                F32x4ConvertI32x4S => visit_f32x4_convert_i32x4_s (arity 1 -> 1)
                F32x4ConvertI32x4U => visit_f32x4_convert_i32x4_u (arity 1 -> 1)
                I32x4TruncSatF64x2SZero => visit_i32x4_trunc_sat_f64x2_s_zero (arity 1 -> 1)
                I32x4TruncSatF64x2UZero => visit_i32x4_trunc_sat_f64x2_u_zero (arity 1 -> 1)
                F64x2ConvertLowI32x4S => visit_f64x2_convert_low_i32x4_s (arity 1 -> 1)
                F64x2ConvertLowI32x4U => visit_f64x2_convert_low_i32x4_u (arity 1 -> 1)
                F32x4DemoteF64x2Zero => visit_f32x4_demote_f64x2_zero (arity 1 -> 1)
                F64x2PromoteLowF32x4 => visit_f64x2_promote_low_f32x4 (arity 1 -> 1)
            }

            // Relaxed SIMD operators
            // https://github.com/WebAssembly/relaxed-simd
            @relaxed_simd {
                I8x16RelaxedSwizzle => visit_i8x16_relaxed_swizzle (arity 2 -> 1)
                I32x4RelaxedTruncF32x4S => visit_i32x4_relaxed_trunc_f32x4_s (arity 1 -> 1)
                I32x4RelaxedTruncF32x4U => visit_i32x4_relaxed_trunc_f32x4_u (arity 1 -> 1)
                I32x4RelaxedTruncF64x2SZero => visit_i32x4_relaxed_trunc_f64x2_s_zero (arity 1 -> 1)
                I32x4RelaxedTruncF64x2UZero => visit_i32x4_relaxed_trunc_f64x2_u_zero (arity 1 -> 1)
                F32x4RelaxedMadd => visit_f32x4_relaxed_madd (arity 3 -> 1)
                F32x4RelaxedNmadd => visit_f32x4_relaxed_nmadd (arity 3 -> 1)
                F64x2RelaxedMadd => visit_f64x2_relaxed_madd  (arity 3 -> 1)
                F64x2RelaxedNmadd => visit_f64x2_relaxed_nmadd (arity 3 -> 1)
                I8x16RelaxedLaneselect => visit_i8x16_relaxed_laneselect (arity 3 -> 1)
                I16x8RelaxedLaneselect => visit_i16x8_relaxed_laneselect (arity 3 -> 1)
                I32x4RelaxedLaneselect => visit_i32x4_relaxed_laneselect (arity 3 -> 1)
                I64x2RelaxedLaneselect => visit_i64x2_relaxed_laneselect (arity 3 -> 1)
                F32x4RelaxedMin => visit_f32x4_relaxed_min (arity 2 -> 1)
                F32x4RelaxedMax => visit_f32x4_relaxed_max (arity 2 -> 1)
                F64x2RelaxedMin => visit_f64x2_relaxed_min (arity 2 -> 1)
                F64x2RelaxedMax => visit_f64x2_relaxed_max (arity 2 -> 1)
                I16x8RelaxedQ15mulrS => visit_i16x8_relaxed_q15mulr_s (arity 2 -> 1)
                I16x8RelaxedDotI8x16I7x16S => visit_i16x8_relaxed_dot_i8x16_i7x16_s (arity 2 -> 1)
                I32x4RelaxedDotI8x16I7x16AddS => visit_i32x4_relaxed_dot_i8x16_i7x16_add_s (arity 3 -> 1)
            }

            @exceptions {
                TryTable { try_table: $crate::TryTable } => visit_try_table (arity custom)
                Throw { tag_index: u32 } => visit_throw (arity custom)
                ThrowRef => visit_throw_ref (arity 1 -> 0)
            }
            // Deprecated old instructions from the exceptions proposal
            @legacy_exceptions {
                Try { blockty: $crate::BlockType } => visit_try (arity custom)
                Catch { tag_index: u32 } => visit_catch (arity custom)
                Rethrow { relative_depth: u32 } => visit_rethrow (arity 0 -> 0)
                Delegate { relative_depth: u32 } => visit_delegate (arity custom)
                CatchAll => visit_catch_all (arity custom)
            }

            // Also 0xFE prefixed operators
            // shared-everything threads
            // https://github.com/WebAssembly/shared-everything-threads
            @shared_everything_threads {
                GlobalAtomicGet { ordering: $crate::Ordering, global_index: u32 } => visit_global_atomic_get (arity 0 -> 1)
                GlobalAtomicSet { ordering: $crate::Ordering, global_index: u32 } => visit_global_atomic_set (arity 1 -> 0)
                GlobalAtomicRmwAdd { ordering: $crate::Ordering, global_index: u32 } => visit_global_atomic_rmw_add (arity 1 -> 1)
                GlobalAtomicRmwSub { ordering: $crate::Ordering, global_index: u32 } => visit_global_atomic_rmw_sub (arity 1 -> 1)
                GlobalAtomicRmwAnd { ordering: $crate::Ordering, global_index: u32 } => visit_global_atomic_rmw_and (arity 1 -> 1)
                GlobalAtomicRmwOr { ordering: $crate::Ordering, global_index: u32 } => visit_global_atomic_rmw_or (arity 1 -> 1)
                GlobalAtomicRmwXor { ordering: $crate::Ordering, global_index: u32 } => visit_global_atomic_rmw_xor (arity 1 -> 1)
                GlobalAtomicRmwXchg { ordering: $crate::Ordering, global_index: u32 } => visit_global_atomic_rmw_xchg (arity 1 -> 1)
                GlobalAtomicRmwCmpxchg { ordering: $crate::Ordering, global_index: u32 } => visit_global_atomic_rmw_cmpxchg (arity 2 -> 1)
                TableAtomicGet { ordering: $crate::Ordering, table_index: u32 } => visit_table_atomic_get (arity 1 -> 1)
                TableAtomicSet { ordering: $crate::Ordering, table_index: u32 } => visit_table_atomic_set (arity 2 -> 0)
                TableAtomicRmwXchg { ordering: $crate::Ordering, table_index: u32 } => visit_table_atomic_rmw_xchg (arity 2 -> 1)
                TableAtomicRmwCmpxchg { ordering: $crate::Ordering, table_index: u32 } => visit_table_atomic_rmw_cmpxchg (arity 3 -> 1)
                StructAtomicGet { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_get (arity 1 -> 1)
                StructAtomicGetS { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_get_s (arity 1 -> 1)
                StructAtomicGetU { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_get_u (arity 1 -> 1)
                StructAtomicSet { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_set (arity 2 -> 0)
                StructAtomicRmwAdd { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_rmw_add (arity 2 -> 1)
                StructAtomicRmwSub { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_rmw_sub (arity 2 -> 1)
                StructAtomicRmwAnd { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_rmw_and (arity 2 -> 1)
                StructAtomicRmwOr { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_rmw_or (arity 2 -> 1)
                StructAtomicRmwXor { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_rmw_xor (arity 2 -> 1)
                StructAtomicRmwXchg { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_rmw_xchg (arity 2 -> 1)
                StructAtomicRmwCmpxchg { ordering: $crate::Ordering, struct_type_index: u32, field_index: u32  } => visit_struct_atomic_rmw_cmpxchg (arity 3 -> 1)
                ArrayAtomicGet { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_get (arity 2 -> 1)
                ArrayAtomicGetS { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_get_s (arity 2 -> 1)
                ArrayAtomicGetU { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_get_u (arity 2 -> 1)
                ArrayAtomicSet { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_set (arity 3 -> 0)
                ArrayAtomicRmwAdd { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_rmw_add (arity 3 -> 1)
                ArrayAtomicRmwSub { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_rmw_sub (arity 3 -> 1)
                ArrayAtomicRmwAnd { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_rmw_and (arity 3 -> 1)
                ArrayAtomicRmwOr { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_rmw_or (arity 3 -> 1)
                ArrayAtomicRmwXor { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_rmw_xor (arity 3 -> 1)
                ArrayAtomicRmwXchg { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_rmw_xchg (arity 3 -> 1)
                ArrayAtomicRmwCmpxchg { ordering: $crate::Ordering, array_type_index: u32 } => visit_array_atomic_rmw_cmpxchg (arity 4 -> 1)
                RefI31Shared => visit_ref_i31_shared (arity 1 -> 1)
            }

            // Typed Function references
            @function_references {
                CallRef { type_index: u32 } => visit_call_ref (arity custom)
                ReturnCallRef { type_index: u32 } => visit_return_call_ref (arity custom)
                RefAsNonNull => visit_ref_as_non_null (arity 1 -> 1)
                BrOnNull { relative_depth: u32 } => visit_br_on_null (arity custom)
                BrOnNonNull { relative_depth: u32 } => visit_br_on_non_null (arity custom)
            }

            // Stack switching
            @stack_switching {
                ContNew { cont_type_index: u32 } => visit_cont_new (arity 1 -> 1)
                ContBind { argument_index: u32, result_index: u32 } => visit_cont_bind (arity custom)
                Suspend { tag_index: u32 } => visit_suspend (arity custom)
                Resume { cont_type_index: u32, resume_table: $crate::ResumeTable } => visit_resume (arity custom)
                ResumeThrow { cont_type_index: u32, tag_index: u32, resume_table: $crate::ResumeTable } => visit_resume_throw (arity custom)
                Switch { cont_type_index: u32, tag_index: u32 } => visit_switch (arity custom)
            }

            @wide_arithmetic {
                I64Add128 => visit_i64_add128 (arity 4 -> 2)
                I64Sub128 => visit_i64_sub128 (arity 4 -> 2)
                I64MulWideS => visit_i64_mul_wide_s (arity 2 -> 2)
                I64MulWideU => visit_i64_mul_wide_u (arity 2 -> 2)
            }
        }
    };
}

/// Helper macro to define a `_for_each_non_simd_operator` which receives
/// the syntax of each instruction individually, but only the non-simd
/// operators.
macro_rules! define_for_each_non_simd_operator {
    // Switch from `_for_each_operator_group` syntax to this macro's syntax to
    // be a "tt muncher macro"
    (@ $($t:tt)*) => {define_for_each_non_simd_operator!(filter [] @ $($t)*);};

    // filter out simd/relaxed-simd proposals
    (filter $filter:tt @simd $simd:tt $($rest:tt)*) => {
        define_for_each_non_simd_operator!(filter $filter $($rest)*);
    };
    (filter $filter:tt @relaxed_simd $simd:tt $($rest:tt)*) => {
        define_for_each_non_simd_operator!(filter $filter $($rest)*);
    };

    // For all other proposals add in tokens where the `@proposal` is prepended
    // before each instruction.
    (
        filter [$($t:tt)*]
        @$proposal:ident {
            $( $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*) )*
        }
        $($rest:tt)*
    ) => {
        define_for_each_non_simd_operator!(
            filter [
                $($t)*
                $( @$proposal $op $({ $($arg: $argty),* })? => $visit ($($ann)*) )*
            ]
            $($rest)*
        );
    };

    // At the end the `$t` list here is how we want to define
    // `_for_each_non_simd_operator`, so define the macro with these tokens.
    (filter [$($t:tt)*]) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! _for_each_visit_operator_impl {
            ($m:ident) => {
                $m! { $($t)* }
            }
        }

        // When simd is disabled then this macro is additionally the
        // `for_each_operator!` macro implementation
        #[cfg(not(feature = "simd"))]
        #[doc(hidden)]
        pub use _for_each_visit_operator_impl as _for_each_operator_impl;
    };
}
_for_each_operator_group!(define_for_each_non_simd_operator);

/// When the simd feature is enabled then `_for_each_operator_impl` is defined
/// to be the same as the above `define_for_each_non_simd_operator` macro except
/// with all proposals thrown in.
#[cfg(feature = "simd")]
macro_rules! define_for_each_operator_impl_with_simd {
    (
        $(
            @$proposal:ident {
                $( $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*) )*
            }
        )*
    ) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! _for_each_operator_impl {
            ($m:ident) => {
                $m! {
                    $(
                        $(
                            @$proposal $op $({$($arg: $argty),*})? => $visit ($($ann)*)
                        )*
                    )*
                }
            }
        }
    };
}
#[cfg(feature = "simd")]
_for_each_operator_group!(define_for_each_operator_impl_with_simd);

/// Helper macro to define the `_for_each_simd_operator_impl` macro.
///
/// This is basically the same as `define_for_each_non_simd_operator` above
/// except that it's filtering on different proposals.
#[cfg(feature = "simd")]
macro_rules! define_for_each_simd_operator {
    // Switch to "tt muncher" mode
    (@ $($t:tt)*) => {define_for_each_simd_operator!(filter [] @ $($t)*);};

    // Collect the `@simd` and `@relaxed_simd` proposals.
    (
        filter [$($t:tt)*]
        @simd {
            $( $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*) )*
        }
        $($rest:tt)*
    ) => {
        define_for_each_simd_operator!(
            filter [
                $($t)*
                $( @simd $op $({ $($arg: $argty),* })? => $visit ($($ann)*) )*
            ]
            $($rest)*
        );
    };
    (
        filter [$($t:tt)*]
        @relaxed_simd {
            $( $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*) )*
        }
        $($rest:tt)*
    ) => {
        define_for_each_simd_operator!(
            filter [
                $($t)*
                $( @relaxed_simd $op $({ $($arg: $argty),* })? => $visit ($($ann)*) )*
            ]
            $($rest)*
        );
    };

    // Skip all other proposals.
    (filter $filter:tt @$proposal:ident $instrs:tt $($rest:tt)*) => {
        define_for_each_simd_operator!(filter $filter $($rest)*);
    };

    // Base case to define the base macro.
    (filter [$($t:tt)*]) => {
        #[macro_export]
        #[doc(hidden)]
        macro_rules! _for_each_visit_simd_operator_impl {
            ($m:ident) => {
                $m! { $($t)* }
            }
        }
    };
}
#[cfg(feature = "simd")]
_for_each_operator_group!(define_for_each_simd_operator);

/// Used to implement routines for the [`Operator`] enum.
///
/// A helper macro to conveniently iterate over all opcodes recognized by this
/// crate. This can be used to work with either the [`Operator`] enumeration or
/// the [`VisitOperator`] trait if your use case uniformly handles all operators
/// the same way.
///
/// It is also possible to specialize handling of operators depending on the
/// Wasm proposal from which they are originating.
///
/// This is an "iterator macro" where this macro is invoked with the name of
/// another macro, and then that macro is invoked with the list of all
/// operators.
///
/// The list of specializable Wasm proposals is as follows:
///
/// - `@mvp`: Denoting a Wasm operator from the initial Wasm MVP version.
/// - `@exceptions`: [Wasm `exception-handling` proposal]
/// - `@tail_call`: [Wasm `tail-calls` proposal]
/// - `@reference_types`: [Wasm `reference-types` proposal]
/// - `@sign_extension`: [Wasm `sign-extension-ops` proposal]
/// - `@saturating_float_to_int`: [Wasm `non_trapping_float-to-int-conversions` proposal]
/// - `@bulk_memory `:[Wasm `bulk-memory` proposal]
/// - `@simd`: [Wasm `simd` proposal]
/// - `@relaxed_simd`: [Wasm `relaxed-simd` proposal]
/// - `@threads`: [Wasm `threads` proposal]
/// - `@gc`: [Wasm `gc` proposal]
/// - `@stack_switching`: [Wasm `stack-switching` proposal]
/// - `@wide_arithmetic`: [Wasm `wide-arithmetic` proposal]
///
/// [Wasm `exception-handling` proposal]:
/// https://github.com/WebAssembly/exception-handling
///
/// [Wasm `tail-calls` proposal]:
/// https://github.com/WebAssembly/tail-call
///
/// [Wasm `reference-types` proposal]:
/// https://github.com/WebAssembly/reference-types
///
/// [Wasm `sign-extension-ops` proposal]:
/// https://github.com/WebAssembly/sign-extension-ops
///
/// [Wasm `non_trapping_float-to-int-conversions` proposal]:
/// https://github.com/WebAssembly/nontrapping-float-to-int-conversions
///
/// [Wasm `bulk-memory` proposal]:
/// https://github.com/WebAssembly/bulk-memory-operations
///
/// [Wasm `simd` proposal]:
/// https://github.com/webassembly/simd
///
/// [Wasm `relaxed-simd` proposal]:
/// https://github.com/WebAssembly/relaxed-simd
///
/// [Wasm `threads` proposal]:
/// https://github.com/webassembly/threads
///
/// [Wasm `gc` proposal]:
/// https://github.com/WebAssembly/gc
///
/// [Wasm `stack-switching` proposal]:
/// https://github.com/WebAssembly/stack-switching
///
/// [Wasm `wide-arithmetic` proposal]:
/// https://github.com/WebAssembly/wide-arithmetic
///
/// ```
/// fn do_nothing(op: &wasmparser::Operator) {
///     macro_rules! define_match_operator {
///         // The outer layer of repetition represents how all operators are
///         // provided to the macro at the same time.
///         //
///         // The `$proposal` identifier indicates the Wasm proposals from which
///         // the Wasm operator is originating.
///         // For example to specialize the macro match arm for Wasm SIMD proposal
///         // operators you could write `@simd` instead of `@$proposal:ident` to
///         // only catch those operators.
///         //
///         // The `$op` name is bound to the `Operator` variant name. The
///         // payload of the operator is optionally specified (the `$(...)?`
///         // clause) since not all instructions have payloads. Within the payload
///         // each argument is named and has its type specified.
///         //
///         // The `$visit` name is bound to the corresponding name in the
///         // `VisitOperator` trait that this corresponds to.
///         //
///         // The `$ann` annotations give information about the operator's type (e.g. binary i32 or arity 2 -> 1).
///         ($( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*))*) => {
///             match op {
///                 $(
///                     wasmparser::Operator::$op $( { $($arg),* } )? => {
///                         // do nothing for this example
///                     }
///                 )*
///                 _ => unreachable!(), // required because `Operator` enum is non-exhaustive
///             }
///         }
///     }
///     wasmparser::for_each_operator!(define_match_operator);
/// }
/// ```
///
/// If you only wanted to visit the initial base set of wasm instructions, for
/// example, you could do:
///
/// ```
/// fn is_mvp_operator(op: &wasmparser::Operator) -> bool {
///     macro_rules! define_match_operator {
///         // delegate the macro invocation to sub-invocations of this macro to
///         // deal with each instruction on a case-by-case basis.
///         ($( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*))*) => {
///             match op {
///                 $(
///                     wasmparser::Operator::$op $( { $($arg),* } )? => {
///                         define_match_operator!(impl_one @$proposal)
///                     }
///                 )*
///                 _ => unreachable!(), // required because `Operator` enum is non-exhaustive
///             }
///         };
///
///         (impl_one @mvp) => { true };
///         (impl_one @$proposal:ident) => { false };
///     }
///     wasmparser::for_each_operator!(define_match_operator)
/// }
/// ```
#[doc(inline)]
pub use _for_each_operator_impl as for_each_operator;

/// Used to implement the [`VisitOperator`] trait.
///
/// A helper macro to conveniently iterate over all opcodes recognized by this
/// crate. This can be used to work with either the [`Operator`] enumeration or
/// the [`VisitOperator`] trait if your use case uniformly handles all operators
/// the same way.
///
/// It is also possible to specialize handling of operators depending on the
/// Wasm proposal from which they are originating.
///
/// This is an "iterator macro" where this macro is invoked with the name of
/// another macro, and then that macro is invoked with the list of all
/// operators.
///
/// The list of specializable Wasm proposals is as follows:
///
/// - `@mvp`: Denoting a Wasm operator from the initial Wasm MVP version.
/// - `@exceptions`: [Wasm `exception-handling` proposal]
/// - `@tail_call`: [Wasm `tail-calls` proposal]
/// - `@reference_types`: [Wasm `reference-types` proposal]
/// - `@sign_extension`: [Wasm `sign-extension-ops` proposal]
/// - `@saturating_float_to_int`: [Wasm `non_trapping_float-to-int-conversions` proposal]
/// - `@bulk_memory `:[Wasm `bulk-memory` proposal]
/// - `@threads`: [Wasm `threads` proposal]
/// - `@gc`: [Wasm `gc` proposal]
/// - `@stack_switching`: [Wasm `stack-switching` proposal]
/// - `@wide_arithmetic`: [Wasm `wide-arithmetic` proposal]
///
/// Note that this macro does not iterate over the SIMD-related proposals. Those
/// are provided in [`VisitSimdOperator`] and [`for_each_visit_simd_operator`].
/// This macro only handles non-SIMD related operators and so users wanting to
/// handle the SIMD-class of operators need to use that trait/macro as well.
///
/// [Wasm `exception-handling` proposal]:
/// https://github.com/WebAssembly/exception-handling
///
/// [Wasm `tail-calls` proposal]:
/// https://github.com/WebAssembly/tail-call
///
/// [Wasm `reference-types` proposal]:
/// https://github.com/WebAssembly/reference-types
///
/// [Wasm `sign-extension-ops` proposal]:
/// https://github.com/WebAssembly/sign-extension-ops
///
/// [Wasm `non_trapping_float-to-int-conversions` proposal]:
/// https://github.com/WebAssembly/nontrapping-float-to-int-conversions
///
/// [Wasm `bulk-memory` proposal]:
/// https://github.com/WebAssembly/bulk-memory-operations
/// [Wasm `simd` proposal]:
/// https://github.com/webassembly/simd
///
/// [Wasm `relaxed-simd` proposal]:
/// https://github.com/WebAssembly/relaxed-simd
///
/// [Wasm `threads` proposal]:
/// https://github.com/webassembly/threads
///
/// [Wasm `gc` proposal]:
/// https://github.com/WebAssembly/gc
///
/// [Wasm `stack-switching` proposal]:
/// https://github.com/WebAssembly/stack-switching
///
/// [Wasm `wide-arithmetic` proposal]:
/// https://github.com/WebAssembly/wide-arithmetic
///
/// ```
/// macro_rules! define_visit_operator {
///     // The outer layer of repetition represents how all operators are
///     // provided to the macro at the same time.
///     //
///     // The `$proposal` identifier indicates the Wasm proposals from which
///     // the Wasm operator is originating.
///     // For example to specialize the macro match arm for Wasm `gc` proposal
///     // operators you could write `@gc` instead of `@$proposal:ident` to
///     // only catch those operators.
///     //
///     // The `$op` name is bound to the `Operator` variant name. The
///     // payload of the operator is optionally specified (the `$(...)?`
///     // clause) since not all instructions have payloads. Within the payload
///     // each argument is named and has its type specified.
///     //
///     // The `$visit` name is bound to the corresponding name in the
///     // `VisitOperator` trait that this corresponds to.
///     //
///     // The `$ann` annotations give information about the operator's type (e.g. binary i32 or arity 2 -> 1).
///     ($( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*))*) => {
///         $(
///             fn $visit(&mut self $($(,$arg: $argty)*)?) {
///                 // do nothing for this example
///             }
///         )*
///     }
/// }
///
/// pub struct VisitAndDoNothing;
///
/// impl<'a> wasmparser::VisitOperator<'a> for VisitAndDoNothing {
///     type Output = ();
///
///     wasmparser::for_each_visit_operator!(define_visit_operator);
/// }
/// ```
///
/// If you only wanted to visit the initial base set of wasm instructions, for
/// example, you could do:
///
/// ```
/// macro_rules! visit_only_mvp {
///     // delegate the macro invocation to sub-invocations of this macro to
///     // deal with each instruction on a case-by-case basis.
///     ($( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*))*) => {
///         $(
///             visit_only_mvp!(visit_one @$proposal $op $({ $($arg: $argty),* })? => $visit);
///         )*
///     };
///
///     // MVP instructions are defined manually, so do nothing.
///     (visit_one @mvp $($rest:tt)*) => {};
///
///     // Non-MVP instructions all return `false` here. The exact type depends
///     // on `type Output` in the trait implementation below. You could change
///     // it to `Result<()>` for example and return an error here too.
///     (visit_one @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident) => {
///         fn $visit(&mut self $($(,$arg: $argty)*)?) -> bool {
///             false
///         }
///     }
/// }
/// # // to get this example to compile another macro is used here to define
/// # // visit methods for all mvp operators.
/// # macro_rules! visit_mvp {
/// #     ($( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*))*) => {
/// #         $(
/// #             visit_mvp!(visit_one @$proposal $op $({ $($arg: $argty),* })? => $visit);
/// #         )*
/// #     };
/// #     (visit_one @mvp $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident) => {
/// #         fn $visit(&mut self $($(,$arg: $argty)*)?) -> bool {
/// #             true
/// #         }
/// #     };
/// #     (visit_one @$proposal:ident $($rest:tt)*) => {};
/// # }
///
/// pub struct VisitOnlyMvp;
///
/// impl<'a> wasmparser::VisitOperator<'a> for VisitOnlyMvp {
///     type Output = bool;
///
///     wasmparser::for_each_visit_operator!(visit_only_mvp);
/// #   wasmparser::for_each_visit_operator!(visit_mvp);
///
///     // manually define `visit_*` for all MVP operators here
/// }
/// ```
#[doc(inline)]
pub use _for_each_visit_operator_impl as for_each_visit_operator;

/// Used to implement the [`VisitSimdOperator`] trait.
///
/// The list of specializable Wasm proposals is as follows:
///
/// - `@simd`: [Wasm `simd` proposal]
/// - `@relaxed_simd`: [Wasm `relaxed-simd` proposal]
///
/// For more information about the structure and use of this macro please
/// refer to the documentation of the [`for_each_operator`] macro.
///
/// [Wasm `simd` proposal]:
/// https://github.com/webassembly/simd
///
/// [Wasm `relaxed-simd` proposal]:
/// https://github.com/WebAssembly/relaxed-simd
///
/// [`VisitSimdOperator`]: crate::VisitSimdOperator
///
/// ```
/// # macro_rules! define_visit_operator {
/// #     ($( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*))*) => {
/// #         $( fn $visit(&mut self $($(,$arg: $argty)*)?) {} )*
/// #     }
/// # }
/// pub struct VisitAndDoNothing;
///
/// impl<'a> wasmparser::VisitOperator<'a> for VisitAndDoNothing {
///     type Output = ();
///
///     // implement all the visit methods ..
///     # wasmparser::for_each_visit_operator!(define_visit_operator);
/// }
///
/// macro_rules! define_visit_simd_operator {
///     // The outer layer of repetition represents how all operators are
///     // provided to the macro at the same time.
///     //
///     // The `$proposal` identifier is either `@simd` or `@relaxed_simd`.
///     //
///     // The shape of this macro is identical to [`for_each_visit_operator`].
///     // Please refer to its documentation if you want to learn more.
///     ($( @$proposal:ident $op:ident $({ $($arg:ident: $argty:ty),* })? => $visit:ident ($($ann:tt)*))*) => {
///         $(
///             fn $visit(&mut self $($(,$arg: $argty)*)?) {
///                 // do nothing for this example
///             }
///         )*
///     }
/// }
///
/// impl<'a> wasmparser::VisitSimdOperator<'a> for VisitAndDoNothing {
///     wasmparser::for_each_visit_simd_operator!(define_visit_simd_operator);
/// }
/// ```
#[cfg(feature = "simd")]
#[doc(inline)]
pub use _for_each_visit_simd_operator_impl as for_each_visit_simd_operator;

macro_rules! format_err {
    ($offset:expr, $($arg:tt)*) => {
        crate::BinaryReaderError::fmt(format_args!($($arg)*), $offset)
    }
}

macro_rules! bail {
    ($($arg:tt)*) => {return Err(format_err!($($arg)*))}
}

#[cfg(all(feature = "component-model", feature = "validate"))] // Only used in component-model code right now.
macro_rules! ensure {
    ($cond:expr, $($arg:tt)*) => {
        if !$cond {
            bail!($($arg)*);
        }
    }
}

pub use crate::arity::*;
pub use crate::binary_reader::{BinaryReader, BinaryReaderError, Result};
pub use crate::features::*;
pub use crate::parser::*;
pub use crate::readers::*;

mod arity;
mod binary_reader;
mod features;
mod limits;
mod parser;
mod readers;

#[cfg(feature = "validate")]
mod resources;
#[cfg(feature = "validate")]
mod validator;
#[cfg(feature = "validate")]
pub use crate::resources::*;
#[cfg(feature = "validate")]
pub use crate::validator::*;

#[cfg(feature = "validate")]
pub mod collections;

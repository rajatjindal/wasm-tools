(component
  (type (func))
  (type (func (param "x" s8)))
  (type (func (param "x" u8)))
  (type (func (param "x" s16)))
  (type (func (param "x" u16)))
  (type (func (param "x" s32)))
  (type (func (param "x" u32)))
  (type (func (param "x" s64)))
  (type (func (param "x" u64)))
  (type (func (param "x" float32)))
  (type (func (param "x" float64)))
  (type (func (param "x" bool)))
  (type (func (param "x" string)))
  (type (record (field "a" s8) (field "b" u8) (field "c" s16) (field "d" u16) (field "e" s32) (field "f" u32) (field "g" s64) (field "h" u64) (field "i" float32) (field "j" float64) (field "k" bool) (field "l" string)))
  (type (func (param "x" 13)))
  (type (list 13))
  (type (func (param "x" 15)))
  (type (tuple 13 string))
  (type (func (param "x" 17)))
  (type (flags "a" "b" "c"))
  (type (func (param "x" 19)))
  (type (enum "a" "b" "c"))
  (type (func (param "x" 21)))
  (type (union s8 string 13))
  (type (func (param "x" 23)))
  (type (variant (case "a" s8) (case "b" u8) (case "c" s16) (case "d" u16) (case "e" s32) (case "f" u32) (case "g" s64) (case "h" u64) (case "i" float32) (case "j" float64) (case "k" bool) (case "l" string) (case "m" 13)))
  (type (option 25))
  (type (func (param "x" 26)))
  (type (result 13 (error string)))
  (type (func (result 28)))
  (export "record1" (type 13))
  (export "flags1" (type 19))
  (export "enum1" (type 21))
  (export "union1" (type 23))
  (export "variant1" (type 25))
  (core module
    (func $a unreachable)
    (func $b (param i32) unreachable)
    (func $c (param i32) unreachable)
    (func $d (param i32) unreachable)
    (func $e (param i32) unreachable)
    (func $f (param i32) unreachable)
    (func $g (param i32) unreachable)
    (func $h (param i64) unreachable)
    (func $i (param i64) unreachable)
    (func $j (param f32) unreachable)
    (func $k (param f64) unreachable)
    (func $l (param i32) unreachable)
    (func $m (param i32 i32) unreachable)
    (func $n (param i32 i32 i32 i32 i32 i32 i64 i64 f32 f64 i32 i32 i32) unreachable)
    (func $o (param i32 i32) unreachable)
    (func $p (param i32 i32 i32 i32 i32 i32 i64 i64 f32 f64 i32 i32 i32 i32 i32) unreachable)
    (func $q (param i32) unreachable)
    (func $r (param i32) unreachable)
    (func $s (param i32 i32 i32 i32 i32 i32 i32 i64 i64 f32 f64 i32 i32 i32) unreachable)
    (func $t (param i32 i32 i64 i32 i32 i32 i32 i32 i64 i64 f32 f64 i32 i32 i32) unreachable)
    (func $u (result i32) unreachable)
    (func $canonical_abi_realloc (param i32 i32 i32 i32) (result i32) unreachable)
    (memory 0)
    (export "memory" (memory 0))
    (export "a" (func $a))
    (export "b" (func $b))
    (export "c" (func $c))
    (export "d" (func $d))
    (export "e" (func $e))
    (export "f" (func $f))
    (export "g" (func $g))
    (export "h" (func $h))
    (export "i" (func $i))
    (export "j" (func $j))
    (export "k" (func $k))
    (export "l" (func $l))
    (export "m" (func $m))
    (export "n" (func $n))
    (export "o" (func $o))
    (export "p" (func $p))
    (export "q" (func $q))
    (export "r" (func $r))
    (export "s" (func $s))
    (export "t" (func $t))
    (export "u" (func $u))
    (export "canonical_abi_realloc" (func $canonical_abi_realloc))
  )
  (core instance (instantiate 0))
  (alias core export 0 "memory" (core memory))
  (alias core export 0 "canonical_abi_realloc" (core func))
  (alias core export 0 "a" (core func))
  (alias core export 0 "b" (core func))
  (alias core export 0 "c" (core func))
  (alias core export 0 "d" (core func))
  (alias core export 0 "e" (core func))
  (alias core export 0 "f" (core func))
  (alias core export 0 "g" (core func))
  (alias core export 0 "h" (core func))
  (alias core export 0 "i" (core func))
  (alias core export 0 "j" (core func))
  (alias core export 0 "k" (core func))
  (alias core export 0 "l" (core func))
  (alias core export 0 "m" (core func))
  (alias core export 0 "n" (core func))
  (alias core export 0 "o" (core func))
  (alias core export 0 "p" (core func))
  (alias core export 0 "q" (core func))
  (alias core export 0 "r" (core func))
  (alias core export 0 "s" (core func))
  (alias core export 0 "t" (core func))
  (alias core export 0 "u" (core func))
  (func (type 0) (canon lift (core func 1)))
  (func (type 1) (canon lift (core func 2)))
  (func (type 2) (canon lift (core func 3)))
  (func (type 3) (canon lift (core func 4)))
  (func (type 4) (canon lift (core func 5)))
  (func (type 5) (canon lift (core func 6)))
  (func (type 6) (canon lift (core func 7)))
  (func (type 7) (canon lift (core func 8)))
  (func (type 8) (canon lift (core func 9)))
  (func (type 9) (canon lift (core func 10)))
  (func (type 10) (canon lift (core func 11)))
  (func (type 11) (canon lift (core func 12)))
  (func (type 12) (canon lift (core func 13) (memory 0) (realloc 0) string-encoding=utf8))
  (func (type 14) (canon lift (core func 14) (memory 0) (realloc 0) string-encoding=utf8))
  (func (type 16) (canon lift (core func 15) (memory 0) (realloc 0) string-encoding=utf8))
  (func (type 18) (canon lift (core func 16) (memory 0) (realloc 0) string-encoding=utf8))
  (func (type 20) (canon lift (core func 17)))
  (func (type 22) (canon lift (core func 18)))
  (func (type 24) (canon lift (core func 19) (memory 0) (realloc 0) string-encoding=utf8))
  (func (type 27) (canon lift (core func 20) (memory 0) (realloc 0) string-encoding=utf8))
  (func (type 29) (canon lift (core func 21) (memory 0) (realloc 0) string-encoding=utf8))
  (export "a" (func 0))
  (export "b" (func 1))
  (export "c" (func 2))
  (export "d" (func 3))
  (export "e" (func 4))
  (export "f" (func 5))
  (export "g" (func 6))
  (export "h" (func 7))
  (export "i" (func 8))
  (export "j" (func 9))
  (export "k" (func 10))
  (export "l" (func 11))
  (export "m" (func 12))
  (export "n" (func 13))
  (export "o" (func 14))
  (export "p" (func 15))
  (export "q" (func 16))
  (export "r" (func 17))
  (export "s" (func 18))
  (export "t" (func 19))
  (export "u" (func 20))
)
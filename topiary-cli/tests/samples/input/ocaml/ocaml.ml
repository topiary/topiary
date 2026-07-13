(**************************************************************************)
(*                                                                        *)
(*                                 OCaml                                  *)
(*                                                                        *)
(*    Pierre Weis and Xavier Leroy, projet Cristal, INRIA Rocquencourt    *)
(*                                                                        *)
(*   Copyright 1999 Institut National de Recherche en Informatique et     *)
(*     en Automatique.                                                    *)
(*                                                                        *)
(*   All rights reserved.  This file is distributed under the terms of    *)
(*   the GNU Lesser General Public License version 2.1, with the          *)
(*   special exception on linking described in the file LICENSE.          *)
(*                                                                        *)
(**************************************************************************)

(* Issue #541 *)

let one = foo
  |> bar
  >> baz
  <|> quux

let two = foo @
bar @@
baz ^^
quux

let three =
  raise @@
  Exception

(* Issue 529 *)

let id1 = if true then Fun.id else function x -> x

let id2 = if true then Fun.id else function x -> x | y -> y

let id3 = if 
true 
then 
Fun.id 
else 
function x -> x

let id4 = function x -> x

let id5 = if true then Fun.id else function | x -> x

let id6 = function | x -> x

(* Extensible buffers *)

type t =      (* Multi-
               * line comment with
               * too much padding.
               *)
 {mutable buffer : bytes;
  mutable position : int;  (* End-of-line comment *)
  mutable length : int;
  initial_buffer : bytes}
(* Invariants: all parts of the code preserve the invariants that:
   - [0 <= b.position <= b.length]
   - [b.length = Bytes.length b.buffer]

   Note in particular that [b.position = b.length] is legal,
   it means that the buffer is full and will have to be extended
   before any further addition. *)

let create n =
  let n = if n < 1 then 1 else n in
  let n = if n > Sys.max_string_length then Sys.max_string_length else n in
  let s = Bytes.create n in
  {buffer = s; position = 0; length = n; initial_buffer = s}

let contents b = Bytes.sub_string b.buffer 0 b.position
let to_bytes b = Bytes.sub b.buffer 0 b.position

let sub b ofs len =
  if ofs < 0 || len < 0 || ofs > b.position - len
  then invalid_arg "Buffer.sub"
  else Bytes.sub_string b.buffer ofs len


let blit src srcoff dst dstoff len =
  if len < 0 || srcoff < 0 || srcoff > src.position - len
             || dstoff < 0 || dstoff > (Bytes.length dst) - len
  then
    invalid_arg "Buffer.blit"
  else
    Bytes.unsafe_blit src.buffer srcoff dst dstoff len


let nth b ofs =
  if ofs < 0 || ofs >= b.position then
   invalid_arg "Buffer.nth"
  else Bytes.unsafe_get b.buffer ofs


let length b = b.position

let clear b = b.position <- 0

let reset b =
  b.position <- 0;
  b.buffer <- b.initial_buffer;
  b.length <- Bytes.length b.buffer

(* [resize b more] ensures that [b.position + more <= b.length] holds
   by dynamically extending [b.buffer] if necessary -- and thus
   increasing [b.length].

   In particular, after [resize b more] is called, a direct access of
   size [more] at [b.position] will always be in-bounds, so that
   (unsafe_{get,set}) may be used for performance.
*)
let resize b more =
  let old_pos = b.position in
  let old_len = b.length in
  let new_len = ref old_len in
  while old_pos + more > !new_len do new_len := 2 * !new_len done;
  if !new_len > Sys.max_string_length then begin
    if old_pos + more <= Sys.max_string_length
    then new_len := Sys.max_string_length
    else failwith "Buffer.add: cannot grow buffer"
  end;
  let new_buffer = Bytes.create !new_len in
  (* PR#6148: let's keep using [blit] rather than [unsafe_blit] in
     this tricky function that is slow anyway. *)
  Bytes.blit b.buffer 0 new_buffer 0 b.position;
  b.buffer <- new_buffer;
  b.length <- !new_len;
  assert (b.position + more <= b.length);
  assert (old_pos + more <= b.length);
  ()
  (* Note: there are various situations (preemptive threads, signals and
     gc finalizers) where OCaml code may be run asynchronously; in
     particular, there may be a race with another user of [b], changing
     its mutable fields in the middle of the [resize] call. The Buffer
     module does not provide any correctness guarantee if that happens,
     but we must still ensure that the datastructure invariants hold for
     memory-safety -- as we plan to use [unsafe_{get,set}].

     There are two potential allocation points in this function,
     [ref] and [Bytes.create], but all reads and writes to the fields
     of [b] happen before both of them or after both of them.

     We therefore assume that [b.position] may change at these allocations,
     and check that the [b.position + more <= b.length] postcondition
     holds for both values of [b.position], before or after the function
     is called. More precisely, the following invariants must hold if the
     function returns correctly, in addition to the usual buffer invariants:
     - [old(b.position) + more <= new(b.length)]
     - [new(b.position) + more <= new(b.length)]
     - [old(b.length) <= new(b.length)]

     Note: [b.position + more <= old(b.length)] does *not*
     hold in general, as it is precisely the case where you need
     to call [resize] to increase [b.length].

     Note: [assert] above does not mean that we know the conditions
     always hold, but that the function may return correctly
     only if they hold.

     Note: the other functions in this module does not need
     to be checked with this level of scrutiny, given that they
     read/write the buffer immediately after checking that
     [b.position + more <= b.length] hold or calling [resize].
  *)

let add_char b c =
  let pos = b.position in
  if pos >= b.length then resize b 1;
  Bytes.unsafe_set b.buffer pos c;
  b.position <- pos + 1

let uchar_utf_8_byte_length_max = 4
let uchar_utf_16_byte_length_max = 4

let rec add_utf_8_uchar b u =
  let pos = b.position in
  if pos >= b.length then resize b uchar_utf_8_byte_length_max;
  let n = Bytes.set_utf_8_uchar b.buffer pos u in
  if n = 0
  then (resize b uchar_utf_8_byte_length_max; add_utf_8_uchar b u)
  else (b.position <- pos + n)

let rec add_utf_16be_uchar b u =
  let pos = b.position in
  if pos >= b.length then resize b uchar_utf_16_byte_length_max;
  let n = Bytes.set_utf_16be_uchar b.buffer pos u in
  if n = 0
  then (resize b uchar_utf_16_byte_length_max; add_utf_16be_uchar b u)
  else (b.position <- pos + n)

let rec add_utf_16le_uchar b u =
  let pos = b.position in
  if pos >= b.length then resize b uchar_utf_16_byte_length_max;
  let n = Bytes.set_utf_16le_uchar b.buffer pos u in
  if n = 0
  then (resize b uchar_utf_16_byte_length_max; add_utf_16le_uchar b u)
  else (b.position <- pos + n)

let add_substring b s offset len =
  if offset < 0 || len < 0 || offset > String.length s - len
  then invalid_arg "Buffer.add_substring/add_subbytes";
  let new_position = b.position + len in
  if new_position > b.length then resize b len;
  Bytes.unsafe_blit_string s offset b.buffer b.position len;
  b.position <- new_position

let add_subbytes b s offset len =
  add_substring b (Bytes.unsafe_to_string s) offset len

let add_string b s =
  let len = String.length s in
  let new_position = b.position + len in
  if new_position > b.length then resize b len;
  Bytes.unsafe_blit_string s 0 b.buffer b.position len;
  b.position <- new_position

let add_bytes b s = add_string b (Bytes.unsafe_to_string s)

let add_buffer b bs =
  add_subbytes b bs.buffer 0 bs.position

(* this (private) function could move into the standard library *)
let really_input_up_to ic buf ofs len =
  let rec loop ic buf ~already_read ~ofs ~to_read =
    if to_read = 0 then already_read
    else begin
      let r = input ic buf ofs to_read in
      if r = 0 then already_read
      else begin
        let already_read = already_read + r in
        let ofs = ofs + r in
        let to_read = to_read - r in
        loop ic buf ~already_read ~ofs ~to_read
      end
    end
  in loop ic buf ~already_read:0 ~ofs ~to_read:len


let unsafe_add_channel_up_to b ic len =
  if b.position + len > b.length then resize b len;
  let n = really_input_up_to ic b.buffer b.position len in
  (* The assertion below may fail in weird scenario where
     threaded/finalizer code, run asynchronously during the
     [really_input_up_to] call, races on the buffer; we don't ensure
     correctness in this case, but need to preserve the invariants for
     memory-safety (see discussion of [resize]). *)
  assert (b.position + n <= b.length);
  b.position <- b.position + n;
  n

let add_channel b ic len =
(* Test multi-line
   comment indenting
*)
  if len < 0 || len > Sys.max_string_length then   (* PR#5004 *)
    invalid_arg "Buffer.add_channel";
  let n = unsafe_add_channel_up_to b ic len in
  (* It is intentional that a consumer catching End_of_file
     will see the data written (see #6719, #7136). *)
  if n < len then raise End_of_file;
  ()

let output_buffer oc b =
  output oc b.buffer 0 b.position

let closing = function
  | '(' -> ')'
  | '{' -> '}'
  | _ -> assert false

(* opening and closing: open and close characters, typically ( and )
   k: balance of opening and closing chars
   s: the string where we are searching
   start: the index where we start the search. *)
let advance_to_closing opening closing k s start =
  let rec advance k i lim =
    if i >= lim then raise Not_found else
    if s.[i] = opening then advance (k + 1) (i + 1) lim else
    if s.[i] = closing then
      if k = 0 then i else advance (k - 1) (i + 1) lim
    else advance k (i + 1) lim in
  advance k start (String.length s)

let advance_to_non_alpha s start =
  let rec advance i lim =
    if i >= lim then lim else
    match s.[i] with
    | 'a' .. 'z' | 'A' .. 'Z' | '0' .. '9' | '_' -> advance (i + 1) lim
    | _ -> i in
  advance start (String.length s)

(* We are just at the beginning of an ident in s, starting at start. *)
let find_ident s start lim =
  if start >= lim then raise Not_found else
  match s.[start] with
  (* Parenthesized ident ? *)
  | '(' | '{' as c ->
     let new_start = start + 1 in
     let stop = advance_to_closing c (closing c) 0 s new_start in
     String.sub s new_start (stop - start - 1), stop + 1
  (* Regular ident *)
  | _ ->
     let stop = advance_to_non_alpha s (start + 1) in
     String.sub s start (stop - start), stop

(* Substitute $ident, $(ident), or ${ident} in s,
    according to the function mapping f. *)
let add_substitute b f s =
  let lim = String.length s in
  let rec subst previous i =
    if i < lim then begin
      match s.[i] with
      | '$' as current when previous = '\\' ->
         add_char b current;
         subst ' ' (i + 1)
      | '$' ->
         let j = i + 1 in
         let ident, next_i = find_ident s j lim in
         add_string b (f ident);
         subst ' ' next_i
      | current when previous == '\\' ->
         add_char b '\\';
         add_char b current;
         subst ' ' (i + 1)
      | '\\' as current ->
         subst current (i + 1)
      | current ->
         add_char b current;
         subst current (i + 1)
    end else
    if previous = '\\' then add_char b previous in
  subst ' ' 0

let truncate b len =
    if len < 0 || len > length b then
      invalid_arg "Buffer.truncate"
    else
      b.position <- len

(** {1 Iterators} *)

let to_seq b =
  let rec aux i () =
    (* Note that b.position is not a constant and cannot be lifted out of aux *)
    if i >= b.position then Seq.Nil
    else
      let x = Bytes.unsafe_get b.buffer i in
      Seq.Cons (x, aux (i+1))
  in
  aux 0

let to_seqi b =
  let rec aux i () =
    (* Note that b.position is not a constant and cannot be lifted out of aux *)
    if i >= b.position then Seq.Nil
    else
      let x = Bytes.unsafe_get b.buffer i in
      Seq.Cons ((i,x), aux (i+1))
  in
  aux 0

let add_seq b seq = Seq.iter (add_char b) seq

let of_seq i =
  let b = create 32 in
  add_seq b i;
  b

(** {6 Binary encoding of integers} *)

external unsafe_set_int8 : bytes -> int -> int -> unit = "%bytes_unsafe_set"
external unsafe_set_int16 : bytes -> int -> int -> unit = "%caml_bytes_set16u"
external unsafe_set_int32 : bytes -> int -> int32 -> unit = "%caml_bytes_set32u"
external unsafe_set_int64 : bytes -> int -> int64 -> unit = "%caml_bytes_set64u"
external swap16 : int -> int = "%bswap16"
external swap32 : int32 -> int32 = "%bswap_int32"
external swap64 : int64 -> int64 = "%bswap_int64"


let add_int8 b x =
  let new_position = b.position + 1 in
  if new_position > b.length then resize b 1;
  unsafe_set_int8 b.buffer b.position x;
  b.position <- new_position

let add_int16_ne b x =
  let new_position = b.position + 2 in
  if new_position > b.length then resize b 2;
  unsafe_set_int16 b.buffer b.position x;
  b.position <- new_position

let add_int32_ne b x =
  let new_position = b.position + 4 in
  if new_position > b.length then resize b 4;
  unsafe_set_int32 b.buffer b.position x;
  b.position <- new_position

let add_int64_ne b x =
  let new_position = b.position + 8 in
  if new_position > b.length then resize b 8;
  unsafe_set_int64 b.buffer b.position x;
  b.position <- new_position

let add_int16_le b x =
  add_int16_ne b (if Sys.big_endian then swap16 x else x)

let add_int16_be b x =
  add_int16_ne b (if Sys.big_endian then x else swap16 x)

let add_int32_le b x =
  add_int32_ne b (if Sys.big_endian then swap32 x else x)

let add_int32_be b x =
  add_int32_ne b (if Sys.big_endian then x else swap32 x)

let add_int64_le b x =
  add_int64_ne b (if Sys.big_endian then swap64 x else x)

let add_int64_be b x =
  add_int64_ne b (if Sys.big_endian then x else swap64 x)

let add_uint8 = add_int8
let add_uint16_ne = add_int16_ne
let add_uint16_le = add_int16_le
let add_uint16_be = add_int16_be

(* 1 *)
let foo =
if u then let x = v1 in v2
else let y = w1 in w2

(* 2 *)
let foo =
if u then let x = v1 in v2
else
  let y = w1 in
  w2

(* 3 *)
let foo =
if u then
  let x = v1 in
  v2
else let y = w1 in w2

(* 4 *)
let foo =
if u then
  let x = v1 in
  v2
else
  let y = w1 in
  w2

(* Test of a first-class module. *)

module type FOO = sig val foo : string end
let create foo : (module FOO) = (module struct let foo = foo end)
module Foo = (val create "Issue #106")

let _ =
  let f x l = x::l in
  f 120 [130]

let topological_sort deps =
  let to_graph _ deps graph = (deps.file, MSet.elements deps.deps)::graph in
  let graph = Hashtbl.fold to_graph deps [] in
  let rec explore path visited node =
    if List.mem node path then
      raise @@ Dep_error (CircularDependencies (node, path));
    if List.mem node visited then visited
    else
      let edges = try
          List.assoc node graph
        with
          Not_found ->
            if !ignore then []
            else
              raise
                @@ Files_legacy.Files_error (ObjectFileNotFound (mk_mident node))
                | _ -> assert false
      in node::List.fold_left (explore (node::path)) visited
        (List.map Files_legacy.get_file edges)
  in
  List.rev @@ List.fold_left (fun visited (n, _) -> explore [] visited n) [] graph

(* The even and odd functions assume that their argument is non-negative. *)
let rec odd = function
| 0 -> false
| y -> even (y - 1)

  and even y = if y = 0 then true else odd (y - 1)


module ListSetExtended = struct
  include ListSet
  let of_list lst = List.fold_right add lst empty
end

module My_types = struct
type nonrec t=t
type my_rec={my_bool: bool;}
end


module type Printer = sig
(** [print_list sep printer] returns a printer for ['a list] using [printer] as
    element printer and [sep] as separator between elements. *)
val print_list : string -> 'a printer -> 'a list printer
val print_name : name printer

val print_ident : ident printer
end

(* Let us declare an extensible variant. *)
type _ variant = ..

(* And extend it. *)
type _ variant +=
  | Basic : unit variant
  | PrettyPrinter : unit variant  (** Pretty print *)

(* Two times. *)
type _ variant += WithBuff : t variant

let add_multiline x =
  let res =
    x + x in
  res

let add_one_line x = let res = x + x in res

let add_two_lines x =
  let res = x + x in
  res

let add_three_lines x =
  let res = x + x
  in
  res

let add_as_fun_multiline = fun x ->
  x

let add_as_fun_one_line = fun x -> x

let sum_of_int n =
  let res = ref 0 in
  for i = 1 to n do res := !res + i; done

let sum_of_int_reversed n =
  let res = ref 0 in
  for i = n downto 1 do res := !res + i; done

let verbose_id = function
  | -1 -> -1
  | n -> n

let is_prime n =
  let no_divisor = ref true in
  let i = ref 1 in
  while !no_divisor && !i * !i < n do
    incr i;
    if n mod !i = 0
    then no_divisor := false;
  done;
  !no_divisor

let unbox_rec = function
  | Some My_types.{ my_bool } -> my_bool
  | _ -> false

let unbox_bool = function
  | Some true -> true
  | _ -> false

let is_some_letter = function
  | Some 'a'..'z' -> true
  | _ -> false

let is_some_some = function
  | Some Some _ -> true
  | _ -> false

let my_const :
  type a b. a: a -> b: b -> a =
  fun ~a ~b -> a

let my_id ~value = value

let into_list ~value = my_id ~value :: []

let long_function
  (long_argument_1 : int)
  (long_argument_2 : int)
  (long_argument_3 : int)
  (long_argument_4 : int) : unit =
  ()

let large_const =
  let val = 3 in
  fun
    (long_argument_1 : int)
    (long_argument_2 : int)
    (long_argument_3 : int)
    (long_argument_4 : int) : int ->
    val

let [a; _; _] = [1; 2;
  3]

let [|a; _; _|] = [|1; 2;
  3|]

let _ = (let x = 42 in
  x)

let _ =
  let foo =
    fun x -> x
  in
  foo

let _ =
  let foo =
    function true -> false | false -> true
  in
  foo

(* Showcase the usage of operator bindings *)
let greetings =
  let (let*) = Option.bind
  and (and*) a_opt b_opt =
    match (a_opt, b_opt) with
    | (Some a, Some b) -> Some (a, b)
    | _ -> None
  in
  let* msg1 =
    Option.map String.capitalize_ascii (Some "hello ")
  and* msg2 = Some "world"
  in
  Some (msg1 ^ msg2)

(* Some pattern-matching *)
let hd::_ = [1; 2; 3]

let Some message = Some "message"

let [1; snd] = [1; 2]

type a = int and
b = float

type a = int

and b = float

(* Messing around with quoted strings and curly brackets *)
type string_record = { my_string: string; }
let quoted_string =
  {|Hello|}
and quoted_string_multiline_with_id =
  {external|
World
{|!|}
|external}
let _ =
  {
    my_string = quoted_string ^ quoted_string_multiline_with_id
  }

(* Tags in pattern matching *)
type my_box = [`Foo of int | `Bar of int]
let unbox = function
  | `Foo a | `Bar a -> a

(* Type coercion *)
let _ = (`Foo 4 :> [`Foo of int])

(* function signature containing type variables *)
let my_const : 'a 'b. 'a -> 'b -> 'a = Fun.const

(* Showcase OOP *)
module Inner1 = struct
  class ['a] pushable =
    object (self)
      val mutable list = ([]: 'a list)

      method push x =
        list <- x::list
    end

  class virtual ['a] poppable =
    object (self)
      inherit ['a] pushable

      (* unsafe implementation *)
      method pop =
        let result::rest = list in
        list <- rest;
        Some result
    end
end

module Inner2 = struct
  class type ['a] stack =
    object
      inherit ['a] Inner1.poppable
    end

  class ['a] stack_impl : ['a] stack =
    object
      inherit ['a] Inner1.poppable as super

      (* safe implementation *)
      method pop =
        match list with
        | _::_ -> super#pop
        | _ -> None
    end
end

class memoized = object
  method __memo__ : __memo__ = __memo__
  initializer memoize_obj this
end

let (Some 2) =
  let my_stack = new Inner2.stack_impl in
  my_stack#push 1;
  my_stack#push 2;
  my_stack#pop

let obj_id (obj : < .. >) = obj

let obj_with_unit_id (obj : < nothing: unit; .. >) = obj

let obj_with_only_unit_id (obj : < nothing: unit >) = obj

(* Some modules and functors *)
module type T1 = sig
  type t = private int64
end

module type T2 = sig end

module M: T2 = struct end

module F (X: T1) (Y: T1 with type t := X.t) = struct
  module type S = sig
    type t = X.t

    val zero : t

    val succ : t -> t

    include T2
  end

  let zero = 0

  let succ n = n + 1

  include M
end

(* Showcase ppx usage *)
let _ =
  let lid = [%sedlex.regexp? R] in
  body

let _ = [%sedlex.regexp R]

let _ =
  [%sedlex match lexbuf with
    | R1 -> e1
    | Rn -> en
    | _  -> def
  ]

let _ = match%sedlex lexbuf with
  | R1 -> e1
  | Rn -> en
  | _  -> def

let x = foo;%lwt bar

let x =
  foo;%lwt bar;%lwt
  baz;%lwt qux

let x =
  foo
    bar;%lwt
  baz

let x = function
  | [%type: [%t? foo] option] ->
    bar

let _ =
  foo <- { slug; };
  bar

let _ = 12 [@deprecated "12 is deprecated, use 13 instead"]

let _ = "some string"
  [@@deprecated "assignment are deprecated, inline everything instead"]

[@@@deprecated "writing code is deprecated, use ai-generated code instead"]

type t =
  { verbose: int
  (** Verbosity level. *)
  ; loggers: string
  (** Loggers enabled. *)
  ; bflags: bool StrMap.t
  (** Boolean flags. *)
  }

let _ =
  { verbose = 0
  ; loggers = "foo"
  ; bflags = StrMap.empty
  }

type t = {
  foo: bool [@default false];
  bar: int
}

type t = {
  foo: bool;
  bar: int [@default 0]
}

type t = {
  foo: bool;
  bar: int [@default 0] (* comment *);
}

(* A merry mess of ppx tags, comments, and misplaced semicolons *)
type t = {
  bar: float;
  foo: bool [@default false] (* a comment *) [@other tag] (* and another one *)
     [@and again] (* and another one *) [@and again] (* and a last one *);
}
type t = {
  bar: float;
  foo: bool [@default false] (* a comment *) [@other tag] (* and another one *)
     [@and again] (* and another one *) [@and again] (* and a last one *)
}

type message = {
  raw_level : int32;
  message_counter : Z.t; [@printer Z.pp_print]
  payload : bytes;
}

let _ = {
  bar = 0.0;
  foo = True
  [@some tag]
  [@some other_tag]
}

type controller =
  | C : Slug.t -> controller

let _ =
  let open Printf in
  sprintf "hello world"

let foo : #x y z = bar

(* Playing with nested structures *)
let _ =
  ( 1, 2
  , 3
  , 4
  )

let _ =
  (1, 2, (3, 4),
  5, 6)

let x = foo || foo
  || bar || bar

let x = foo; foo;
  bar; bar

let _ = {
  foo = let bar = baz in bar
}

(* #618: spacing/softlines in local opens *)
let _ =
  Foo.(bar
    baz    baaz
  )

(* Open and let open *)
open Foo
open Bar

let _ =
  let open Baz in
  ()

(* let module *)
let x =
  let module IMap = Map.Make(Int) in
  IMap.empty

(* Multi-line functor signatures *)
module Lift
  (Credit: module type of CreditSignature)
  (Dance: module type of DanceSignature)
  (Tune: module type of TuneSignature)
  (Version: module type of VersionSignature)
= struct
  let foo = x
end

module MOption:
  functor (A: SERIALISABLE) -> SERIALISABLE with
  type t = A.t option

module MUnit: SERIALISABLE with
  type t = unit

(* Ensure labelled arguments are correctly spaced *)
let _ =
  foo ~arg :: []

(* Various spacing corner cases *)
type query = (string * string list) list
type x = ('any Slug.t -> bool) -> float
let id (type s) (x : s) : s = x
type foo = { a : 'a. ('a, mandatory) arg -> 'a; }
type foo = (int, int) result

(* types with constraints *)
type (+'meth, 'prefix, 'params, 'query, 'input, 'output) service =
  ('meth, 'prefix, 'params, 'query, 'input, 'output, error) raw
  constraint 'meth = [< meth]

type 'a x = 'a option
  constraint 'a = 'b

(* Indentation of multi-line types in PPX syntax *)
let h =
  [%madcast: float ->
    bool]

(* Indentation of function cases in PPX syntax *)
let x =
  [%expr function
    | false -> 0.
    | true -> 1.]

(* New line for structures in module definitions *)
module MFloat : SERIALISABLE
  with type t = float =
struct
  type t = float [@@deriving yojson]
  let _key = "float"
end

module Make (R : sig end)
  (S : sig end) (T : sig end)

(* Showcase usage of line number directive. Line breaks could be improved. *)
module Foo = struct
  val bar

# 1 "v1/pervasives.mli"
end

module MyTime = struct include Time end

(* Nested match cases *)
let foo = function
  | x -> (
    try foo with _ -> None )

let _ = try%foo
    bar
  with
    | Not_found -> baz
    | _ -> qux

let f = function
  | None -> ()
  | Some x ->
    (match x with None -> () | Some _ -> ())

let foo = function
  | Some _ ->
    (function true -> false | false -> true)
  | None ->
    (function true -> true | false -> false)

(* set expressions *)
let _ =
  x <-
    if foo then
      bar
    else
      baz

(* various test cases for "|" rules *)
let not = function
  true -> false
  | false -> true

let not x = match x with
  | false -> true
  | true -> false

let not =
  function | true -> false | false -> true

let not = function true -> false | false -> true

let not x = match x with | true -> false | false -> true

exception MyException of string

(* Idempotence failures from #441 *)
let _ =
  let module Lift = Mlfi_types.TypEq.Lift (struct type 'a c = 'a S.t end) in ()

let _ = function | Error s -> failwith s | Ok x -> x

(* Slight variant of #441 *)
let _ =
  let module Foo = struct let f = 0 let g = 1 end in ()

(* Indentation/newlines after non-specific infix operators *)
let _ = foo bar @@ let () = qux in
  baz

let _ =
  myBlockIntroducingFunction @@ fun x ->
    something horrible onto x

(* #617: multi-line or_pattern in match case *)
let _ =
  match foo with
  | A | B | C
  | D | E | F -> bar

(* #642: interleaving regular sequence with PPX sequence *)
let () =
  foo;
  bar;%lwt
  baz

let () = foo; bar;%lwt baz

(* #644: softlines before module definition in a module type *)
module type F = sig
  module G : sig val h : string end

  type ('state, 'proof, 'output) implementation =
    (
      module S with
      type state = 'state
      and type proof = 'proof
      and type output_proof = 'output
    )
end

(* #645: idempotency error on first-class module types *)
let peer_functor : (module Peer.F) =
  if !listening then (module Server) else (module Client)

(* #646: multi-line functor type formatted as single-line*)
module Make:
  functor (I: Interface.S) ->
  functor (_: Transport.S with type payload = I.data Payload.t) ->
  S

(* #647: missing space for pattern matching of constructor holding list *)
let _ =
  match foo with
  | Bar ["baz"] -> qux

(* #719: missing space for pattern matching of constructor holding record *)
let _ =
  match foo with
  | Bar { baz } -> qux

(* #721: unbalanced spacing around parenthesized expressions *)
let _ = (begin end)

(* #718: indentations and newlines around final-argument continuations *)
let eval_tree ~addr ~source_path (tree : Syn.tree) =
  let fm = {T.empty_frontmatter with addr; source_path} in
  Frontmatter.run ~init:fm @@ fun () ->
  Emitted_trees.run ~init:[] @@ fun () ->
  Jobs.run ~init:[] @@ fun () ->
  Heap.run ~init:Env.empty @@ fun () ->
  Lex_env.run ~env:Env.empty @@ fun () ->
  Dyn_env.run ~env:Env.empty @@ fun () ->
  let main = eval_tree_inner ~addr tree in
  let side = Emitted_trees.get () in
  let jobs = Jobs.get () in
  {main; side; jobs}

(* #660 dangling functions *)
let foo x = fun y ->
  zzzzzzzzzz

let () =
  foo @@ fun y ->
  zzzzzzzzzz

let () =
  foo >>= fun y ->
  zzzzzzzzzz

let () =
  foo x (fun y ->
    zzzzzzzzzz
  )

let foo x = function
  | y -> zzzzzzzzzz
  | u -> vvvvvvvv

let () =
  foo x @@ function
    | y -> zzzzzzzzzz
    | u -> vvvvvvvv

let () =
  foo x >>= function
    | y -> zzzzzzzzzz
    | u -> vvvvvvvv

let () =
  foo x (function
    | y -> zzzzzzzzzz
    | u -> vvvvvvvvv
  )

(* #727 proper formatting of multi-lined typed function argument *)
let foo
  (bar :
    int ->
    string ->
    unit
  )
  ~(baz :
    int ->
    string ->
    unit
  )
=
  bar baz

(* #728 missing space in module function arguments *)
let foo
    (bar : int)
    (module Baz : BAZTYPE)
  =
  Baz.foo bar

(* #729 Spacing in single-line records *)
let _ = [a; b; c]
let _ = [|a; b; c|]
let _ = { a; b; c }
type x = [`Foo | `Bar]
type x = [> `Foo | `Bar]
type x = [< `Foo | `Bar]
type x = { a: int; b: int; c: int }

(* #726 multi-line calls containing functions *)
let _ =
  foo
    bar
    (fun baz ->
      baaaaaaz
    )
    (fun qux ->
      quuuuuux
    )

(* #659 handling of the `;;` separator *)

let bonjour () = "Bonjour"

;;

print_string (bonjour ());
print_string ", "

;;

let topiary _x = "Topiary"

;;

print_string (topiary 27);
print_endline "!"

(* #730 consistency of opening brackets dangling *)
let foo = [|
  1;
  2;
  3;
|]
let foo = [
  1;
  2;
  3;
]
let foo = {
  a = 1;
  b = 2;
  c = 3;
}
type bar = [
  | `A
  | `B
  | `C
]
type bar = {
  a: int;
  b: int;
  c: int;
}

type t =
  (
    int,
    float,
    string
  ) foo

type bar = [ | `A | `B ]

type bar =
  [`A]

type bar = {
  a: int; b: int
  }

let foo =
  [1; 2;
  3]

let foo = [| 1; 2; 3; |]

let foo =
  [|1; 2; 3|]

let foo =
  {a = 1; b = 2}

(* #731 function dangling on multi-line calls *)
let _ =
  foo bar baz @@ fun x ->
  x

let _ =
  foo
    bar
    baz
    @@ fun x ->
    x

(* #546 Hanging forms *)
let _ =
  somefun @@
    fun x -> body

(* #882 Effect handling syntax *)
let _ =
  try comp1 () with
  | effect (Xchg n), k -> continue k (n+1)

(* #661 Dangling of last argument in function application *)
let _ =
  my_function {
    x = 2;
    y = 12.;
  }

let _ =
  my_function @@ [|
    2;
    12.;
  |]

(* #907 Various dangling behaviours *)
let _ = {
  x = 1;
}

let _ =
  { x = 1;}

let _ =
  {
    x = 1;
  }

let _ = {x = 1}

type t =
  (
    int,
    float,
    string
  ) foo

let foo = {
  x = 2;
  y = 12.;
}

let () =
  let foo = {
    x = 2;
    y = 12.;
  }
  in
  something_with_foo

let () =
  my_function {
    x = 2;
    y = 12.;
  }

let () =
  my_function @@ {
    x = 2;
    y = 12.;
  }

type foo = {
  bar:
    int list;
  baz:
    [`Qux | `Bar];
}

(* #914 to #917 Edge cases in dangling behaviours *)
let _ =
  match foo with
  | _ ->
    (
      match bar with
      | [] -> None
      | _ -> foo
    )

let _ =
  match foo with
  | _ ->
    (
      if path = [] then
        ()
      else
        ()
    )

let foo : (bar, baz) qux =
  ()

let _ =
  div
    ~a: [a_id "page_nav"]
    [
      some stuff
    ]

let _ :: [
  foo;
  bar;
  baz
] =
  [1;2;3;4]

let [|
  a;
  b;
|] = [|1; 2|]

let {
  a;
  b;
} = {
  a = 1;
  b = 2;
}

(* #951 More dangling behaviour issues *)
let foo =
  (
    match bar with
    | [] | [] ->
      qux
  )

  let foo = (
    match bar with
    | [], [] ->
      qux
  )

(* #956 #957 #965 Dangling behaviour issues, yet again *)
let _ =
  foo
    (
      bar {baz} qux
    )

let _ =
  foo
    (
      let%tag {bar} = baz in
      ()
    )

let _ =
  foo
    (fun {bar} ->
      bar
    )

(* #971 Dangling behaviour issues, _and_ `function_application` scope errors *)
let _ =
  foo (
    bar
      [(qux)]
  )

let foo = [
  [bar]; [baz];
]

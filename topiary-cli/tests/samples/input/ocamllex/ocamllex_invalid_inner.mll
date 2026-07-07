(* Regression test: invalid injected OCaml should pass through unchanged. *)
{ let x = } rule token = parse | eof { EOF }

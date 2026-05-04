(**************************************************************************)
(*  Taken from https://github.com/paris-branch/dancelor/                  *)
(*  Licened under GPL-3.0-or-later                                        *)
(*  Copyright belongs to the original authors                             *)
(**************************************************************************)
{
  open TextFormulaParser

  exception UnexpectedCharacter of char
  exception UnterminatedQuote

  let keywords =
    [ "true",   TRUE;
      "false",  FALSE;
      "and",    AND;
      "or",     OR;
      "not",    NOT ]
}

let alpha = ['a'-'z' 'A'-'Z']
let symbol = ['_' '-']
let digit = ['0'-'9']

let identifier = (alpha | symbol | digit)+

rule token   arg1   arg2 = parse

  | ' ' { token lexbuf }

  | '(' { LPAR }
  | ')' { RPAR }

  | ":" (identifier as id) {
      match List.assoc_opt id keywords with
      | Some kw -> kw
      | _ -> NULLARY_PREDICATE id
    }

  | (identifier as id) ":" {
      PREDICATE id
    }

  | '"' {
      let buf = Buffer.create 8 in
      LITERAL (string buf lexbuf)
    }

  (* The following pattern must exclude all the special characters matched above. *)
  | [^ ' ' '(' ')' ':' '"']+ as lit {
      LITERAL lit
    }

  | _ as c { raise (UnexpectedCharacter c) }

  | eof { EOF }

  | ('=' '.' '=') as qute_smily { some_ocaml_code }

and string buf = parse
  | '"'    { Buffer.contents buf }
  | _ as c { Buffer.add_char buf c; string buf lexbuf }
  | eof    { raise UnterminatedQuote }

and ocaml = parse "ocaml" { OCaml }

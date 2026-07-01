/* We want to implement a syntax which looks a bit (and I mean a bit)
like Prolog, at least for the accessibility rules. This is not quite
all, because we need range formulas, and the goal. So I think it will be a bit like this

    [Ranges]
    bow ∈ { "Magic bat", "Something else"}.
    "desert big key" in { "Desert chest above balls", "Something else" }.

    [Rules]
    reach: "Desert boss" :- have: bow.

  A few things to note:
  - Arbitrary strings are literals, but if not a simple identifier, must be quoted
  - Predicates are written `have: <item>` or `reach: <location>`
  - Whitespaces are irrelevant
  - Unicode is allowed but not necessary
  - Ranges define all the possible items, but maybe we need to declare the locations somewhere else
  - I don't have a syntax for comments, yet
  - Missing: a syntax to name sets of locations
*/

%{

open Grammar

let error ?loc code = raise (ParseError(loc,code))
%}

%token EOF
%token IN // ∈ or in
%token EQ // =
%token OPENBRACE CLOSEBRACE COMMA
%token REVIMPL // :-
%token RANGESSECTION
%token RULESSECTION
%token GOALSECTION
%token COLON PERIOD
%token <String.t> IDENT
%token HAVEPREDICATE
%token REACHPREDICATE

%type <Grammar.section list> program
%start program

%%

program:
| p=list(section); EOF { p }
| loc=loc(error) { error ~loc "Syntax error" }

section:
| RULESSECTION; s=list(rule) { Rules s }
| RANGESSECTION; s=list(range) { Ranges s }
| GOALSECTION; goal=goal { Goal goal }

rule:
| a=atom; REVIMPL; rhs=separated_list(COMMA, atom); PERIOD { (a, rhs) }

range:
| item=IDENT; IN; locs=range_expr; PERIOD
  { RangeDecl (item, locs) }
| range=IDENT; EQ; locs=range_expr; PERIOD
  { RangeDef (range, locs) }

goal:
| goal=atom; PERIOD { goal }

range_expr:
| OPENBRACE; locs=separated_nonempty_list(COMMA, IDENT); CLOSEBRACE { RangeLiteral locs }
| id=IDENT { RangeIdent id }

atom:
| HAVEPREDICATE; COLON; thing=IDENT { Have thing }
| REACHPREDICATE; COLON; thing=IDENT { Reach thing }

%inline loc(X):
| X { $loc }

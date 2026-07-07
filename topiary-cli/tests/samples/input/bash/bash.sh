#!/usr/bin/env bash

# Here is a comment
do_a_thing
produce | consume # here's a comment
something

# comment
a=123 # comment

# a different comment
# that spans multiple lines
if some_command
then
# indented multi-
# line comment
do_something
another_thing --foo    --bar
else
do_something_else
fi


if [[ -e "/some/file" ]]|| true; then
  foo
elif !((1==0))
then
  bar
  baz
else
    baz \
  && quux || xyzzy&
fi

multi \
| line |& pipeline

for thing in foo bar quux
do
  echo $thing
  rm -rf /
done

select thing in foo bar quux; do
  echo $thing
  break
done

for (( i=0; i<10; i++ )); do
  echo $i
done

while true
do
  echo "Hello world!"
done

until true
do
  echo "Hello world!"
done

case "${foo}" in
  single) line --mode ;;
  multi)
    line && mode
    ;;&
  bar|quux)
    xyzzy
  do_something | least_expected
    ;;
  *)
    exit 1
esac

{
  here
  is
  { a; nested; compound; }
}

if { foo; }; then
  echo
fi

(
  here
  is
  ( a; nested; subshell )
)

if ( foo; bar ); then
  echo
fi

{ one; (inside; the); other; }

( one
  { inside
    the
  }
  other
)

function foo  () {
  local x=1
  x=2
  bar
  quux || xyzzy
}

quux() { xyzzy; }

export a b=1 c
declare x=$foo
x=123
echo "${x:-something}"
echo "${x/foo/bar}"
ENV_VAR=123 ANOTHER=456 some_command

cat <<-HEREDOC
	Here is
	a
	  heredoc
	HEREDOC

some_command > output < input
another_thing <<< herestring

if foo 2>/dev/null; then
  exit 1
fi

{
  cat <<EOF
This shouldn't be indented ${foo}
...nor this
EOF
}

readonly a="$(foo | bar || baz --quux 2>&1)"
foo <(bar||baz --something) | tee >(quux)

export xyzzy=$(
  something
  another_thing --foo
)

declare {a,b,c}=1
declare -a an_array=(a b c)
echo "${an_array[@]}"

multi_line_array=(
  a
  b
  [1]+=foo
)

[[ ! "foo" ]] && echo foo
(( 0 )) && echo foo

echo $( (foo); (bar) )

# Rewrite testing
echo `date`
[ "foo" ] && bar
echo $[1+2]
function foo { :; }

# NOTE This MUST be on the last line of this file
foo # bar

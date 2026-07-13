// ================================================================================
// Regression examples
// ================================================================================

// Issue #997

/*
Here is a
block comment
*/

module foo() {
  /*
  Here is another
  block comment
  */
}

// https://github.com/Leathong/openscad-LSP/issues/48
echo(bisector_angle_offset=bisector_angle_offset);
prev_angle = atan2((prev - point).y, (prev - point).x);
echo(prev_angle=prev_angle);

// ================================================================================
// Variables/Imports
// ================================================================================
include <my_path/my_lib.scad>;
use <my_path/my_lib.scad>; include <my_path/my_lib.scad> use <my_path/my_lib.scad>
// variables
rr = a_vector[2]; // member of vector
range1 = [-1.5:0.5:3]; // for() loop range
xx = [0:5]; // alternate for() loop range
$fn = 360; // special variable
E = 2.71828182845904523536028747135266249775724709369995; // constant
cond_var = "is_true" ? true : false;
let (a = 1, b = 2,) {};

// ================================================================================
// Functions
// ================================================================================
function line(point1, point2, width = 1,) =
  let (angle = 90 - atan((point2[1] - point1[1]) / (point2[0] - point1[0])))
  let (offset_x = 0.5 * width * cos(angle), offset_y = 0.5 * width * sin(angle))
  let (offset1 = [ -offset_x, offset_y], offset2 = [offset_x, -offset_y])
  [point1 + offset1, point2 + offset1, point2 + offset2, point1 + offset2];

eager = (function(z) identity)(4);

function null(a, b,) = 0;

echo( - a );
scale( - z ) {}

// ================================================================================
// Transformations
// ================================================================================
cylinder();
cylinder( d = 5, h=100,);
rotate([90, 0, 0])
cylinder();
translate([1, 0, 0]) {
    difference() { translate([0, 1, 0])
        translate([1, 0, 0]) rotate([0, 90, 0])
        cylinder(); cube(); } }

// ================================================================================
// Nested Items
// ================================================================================
module big_module() { function inner_function() = undef; module inner_module() cube();
}

module extern_module() include <other_file.scad>

// ================================================================================
// Control Flow
// ================================================================================
for (i = [10:50]) { let (angle = i*360/20, r= i*2, distance = r*5,) {
    rotate(angle, [1, 0, 0]) translate([0, distance, 0])
    sphere(r = r); }
}

// newline indent propagates from innermost if_block
if ($preview) if(true) foo(); else if(true) if
    (false) foo(false); else translate([2,0,0]) foo(); else bar();

// format propagates from first union_block
if (true) {
    } else if (fn(true)) { foo();} else if (false) { bar();} else { baz(); }


for(i = [1:2:7]) { let (x = i ^ 2,y = x - 1) {
    translate([x,y,0]) sphere(r=i); } }

intersection_for(i = [1,2,3]) { if (i > 1) {
    translate([0,i,0]) cube(); } else { translate([0,i,0]) cube(); } }

// ================================================================================
// Comments
// ================================================================================

/* ignored [Customizer Group] ignored */
/* Multiline
comment
here
*/
my_parameter = /*inline block*/ 5;

function math(x) = /*do math stuff*/ x + 2 // done with math
;

module my_cylinder() {
  // here we create a cylinder
  cylinder(); /* done ! */
  cube();
}

list1 = [
  1,
  2 // comment
];

list2 = [
  1, 2 /* block comment */ ];
arguments1= foo(1,
(point2[1] - point1[1]) / (point2[0] - point1[0]) // comment
);
arguments2= foo(1,
2 // comment
);

// ================================================================================
// Modifiers
// ================================================================================
!cylinder();
* linear_extrude(4) text("Hello");
rotate([0,90,0])# cylinder();
%cube();
// multi modifier
translate(1) # ! cube();
rotate([90]) %
translate()
#
cube();

// ================================================================================
// Assertions/Echoes
// ================================================================================
assert();
assert(10 < 20) cube();
for(y = [3:5]) assert(assert() y < x, "message")
  cylinder();
assert(true) assert() cylinder();
val = assert(true, "strut must be positive")
assert(true, "frame must be nonnegative")
undef;

function foo() =
echo("this can precede an expression") true;


fn = function(x) echo("this is x") x;
echo(fn ? "truthy" : "falsey");
echo(function(y) y ? "first" : "second");


// ================================================================================
// Lists/Ternaries
// ================================================================================
  list1 = [
    1,
    2, 3
  ];
  my_fn = fn1([
      1,
      2,
      3,
    ],
    true
  );

function affine3d_rot_from_to(from, to) =
  assert(is_vector(from))
  assert(is_vector(to))
  assert(len(from) == len(to))
  let (
    from = unit(point3d(from)),
    to = unit(point3d(to))
  )approx(from, to) ? affine3d_identity()
  : from.z == 0 && to.z == 0 ? affine3d_zrot(v_theta(point2d(to)) - v_theta(point2d(from)))
  : let (u = vector_axis(from, to),
    ang = vector_angle(from, to),
    c = cos(ang),
    c2 = 1 - c,
    s = sin(ang)
    // double indent a list preceded by list expression
  ) [
    [1, 0, 0, 0],
    [0, 1, 0, 0],
    [0, 0, 1, 0],
    [0, 0, 0, 1],
  ];

// Prettier style ternaries https://prettier.io/blog/2023/11/13/curious-ternaries
x = foo() ? bar()
  : baz() ?
  qux()
  : true;

// ================================================================================
// Comprehensions
// ================================================================================
conditionless = [for (x = [1:10]) x];
conditioned = [for (x = [1:10]) if ($preview) x];
ifelse = [for (x = [1:10]) if ($preview) x else ln(x)];
if_for_ifelse = [for (x = 0) if (x < 0) for (y = 2) if (y == 2) y else x];
complex_condition = [
    for (x = [1:10]) if (x % 2 == 0) x else if (x < 5) x - 1 else 0
];
spliced = [for (x = [1:10]) x, for (y = [1, 2, 3]) y, for (z = [4, 5, 6]) z];
nested = [for (x = [1:10]) for (y = [1, 2, 3]) for (z = [4, 5, 6]) x * y * z];
grouped = [if (x < 7) (for (y = [1:10]) if (y > x) y) else x];
let_each = [for(i = [0:1]) let(a=90) each arc(angle=a)];
let_for = [let (i=[0:1]) for(i = i) let(a=90) each arc(angle=a)];
let_if = [for(i = [0:1]) let(a=360) if (is_def(isect)) isect];
fn_list = [each function ()10];

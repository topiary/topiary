unsafe impl Send for Language {}
unsafe impl Sync for Language {}

/// Sample doc comment
pub fn node_kind_for_id(&self, id: u16) -> &'static str {
    unsafe { CStr::from_ptr(ffi::ts_language_symbol_name(self.0, id)) }
    .to_str()
    .unwrap();
    "foo"
}

// Comments
// at the beginning.

// More comments.

enum OneLine { Leaf { content: String, /* comment */ id: usize /* another comment */, size: usize, }, Hardline { content: String, id: usize, }, Space, } // End of line comment

enum ExpandEnum {
    Leaf { content: String, /* Comment between fields. */ id: usize, size: usize, },
    Hardline { content: String, id: usize, },
    Space,
}
enum NoFinalComma {
    Space,
}

enum ExpandTwoLevels {
    Leaf {
        /*
         * Multi-line
         * comment
         */
        content: String,
        //   Comment after field declaration in enum variant.
        id: usize,

        size: usize,
    },
    Hardline { content: String, id: usize, },

    // comment between enum items
    Space,
}

enum Mode1 {
    Open, // open
    Closed, // closed
    Either, // just leaving the current mode unchanged
}

enum Mode2 {
    Open,
    /// Doc comment
    /// about Closed.
    Closed,
    // just leaving the current mode unchanged
    Either,
}

enum Mode3 {
    Open,
    Closed,
    Either, /* just leaving the current mode unchanged */
}

enum Mode4 {
    Open,
    Closed,
    /* just leaving the current mode unchanged */
    Either,
}

enum Mode5 {
    Open,
    Closed,
    // just leaving the current
    // mode unchanged
    Either,
}

enum Mode6 {
    Open,
    Closed,
    /* just leaving the current
       mode unchanged */
    Either,
}

// Inline with let stmt
fn inline_let() { let hi = 1; }

// While loop spacing
while i == true {
    let i = 42;
}

// Scoped blocks
{
    let i = 42;
}
{
    let i = 43;
}

// Empty block inside of impl function
impl MyTrait for MyStruct {
    fn foo() {
        // ... logic ...
    }
}

const JSON: Foo = json!({"code": 200,
"success": true,
"payload": {"features": ["serde", "json"],
"homepage": null}});

const JSON2: Value = json!([
  "foo",
  "bar",
]);

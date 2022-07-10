This crate provides procedural macrs to generate functions that return static slices
of struct field names or enum variant names.

# Examples
```rust
use fievar::Fields;

#[derive(Fields)]
struct File {
    id: String,
    name: String,
    mime_type: String,
}

assert_eq!(&["id", "name", "mime_type"], File::fields());
```

You can also rename fields.
```rust
use fievar::Fields;

#[derive(Fields)]
struct File {
    id: String,
    name: String,
    #[fievar(name = "mimeType")]
    mime_type: String,
}

assert_eq!(&["id", "name", "mimeType"], File::fields());
```

# Transformations
## Syntax
*Expression:*  
&ensp;&ensp; [[*T*][`|`*Sep*]]

*Sep:* Word separator.  
&ensp;&ensp; Can be any text.

*T:*
Determines how to transform field/variant.  
&ensp;&ensp; [[*TrCase*][` `*NumAlign*]]

*NumAlign:* Controls alignment of numerals.  
&ensp;&ensp; `1__` | `__1` | `_1_`  

*TrCase:* Controls the case of letters.  
&ensp;&ensp; [*TrWord*[` `*TrWord*[` `*TrWord*]]]  

*TrWord:* Controls the case of individual words.  
&ensp;&ensp; [*TrChar*[*TrChar*[*TrChar*]]]

*TrChar:* Controls the case of individual characters in words.  
&ensp;&ensp; `c` | `C`  

*TrCase* consists of upto three *TrWord*s separated by a space. If there is
only one *TrWord* then it is used to transform all words in field/variant. If
there are two *TrWord*s then the first *TrWord* is applied to the first word of
the field/variant and the second *TrWord* is applied to the rest of the words.
If there are three *TrWord*s then the first and last *TrWord*s are applied to
the first and last words of the field/variant and the second *TrWord* is
applied to the rest of the words. *TrChar*s work similarly on characters in a
word.

## Examples
```rust
use fievar::Variants;

#[derive(Variants)]
enum E {
    #[fievar(transform = "c")] // lowercase
    AVeryLong0Variant,

    #[fievar(transform = "C")] // uppercase
    AVeryLong1Variant,

    #[fievar(transform = "1__|_")] // align numeral left
    AVeryLong2Variant,

    #[fievar(transform = "__1|_")] // align numeral right
    AVeryLong3Variant,

    #[fievar(transform = "_1_|_")] // align numeral middle
    AVeryLong4Variant,

    #[fievar(transform = "c Cc")] // camelCase
    AVeryLong5Variant,

    #[fievar(transform = "c|_")] // snake_case
    AVeryLong6Variant,

    #[fievar(transform = "CcC cCc CcC _1_|*-*")] // something different
    LastVeryLong7Variant,
}

let v = &[
    "averylong0variant",
    "AVERYLONG1VARIANT",
    "A_Very_Long2_Variant",
    "A_Very_Long_3Variant",
    "A_Very_Long_4_Variant",
    "aVeryLong5Variant",
    "a_very_long6_variant",
    "LasT*-*vERy*-*lONg*-*7*-*VarianT"
];
assert_eq!(v, E::variants());
```

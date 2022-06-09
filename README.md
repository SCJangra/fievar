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

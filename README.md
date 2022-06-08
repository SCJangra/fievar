This crate provides procedural macrs to generate functions that return static slices
of struct field names or enum variant names.

# Examples
```rust
use fievar::Fields;

#[derive(Fields)]
struct Token {
    access_token: String,
    refresh_token: String,
}

assert_eq!(&["access_token", "refresh_token"], Token::fields());
```

You can also rename fields.
```rust
use fievar::Fields;

#[derive(Fields)]
struct Token {
    #[fievar(name = "accessToken")]
    access_token: String,
    refresh_token: String,
}

assert_eq!(&["accessToken", "refresh_token"], Token::fields());
```

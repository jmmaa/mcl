# Mark's Configuration Language (MCL)

Its an easy to use config toy language written in rust. I implemented this solely for practicing rust lifetimes. 


**Note: Still in development**

# Introduction

Generally, you can think of this config language as `json`, except without commas and colons.

This `mcl` object
```

mcl {
    repository "https://github.com/jmmaa/mcl"
    stars 0
}

```
is equivalent to `json` object below

```
"mcl": {
    "repository": "https://github.com/jmmaa/mcl",
    "stars": 0
}

```

Unlike `json`, you can use template string literals in here by enclosing the string with backticks

```
mcl {
    template_string `MY TEMPLATE STRING`
    normal_string   "hello I am simple string"
}
```
TODO

# How to use

```rust
use mcl;

fn main() {

    let output = mcl::from_str(
        r#"
        
        foo {
    
            bar "baz"
        }
        
        "#,
    )
    .unwrap();

    // getting value of "bar"
    let val = &output["foo"]["bar"].as_str();

    assert!(val == &Some("baz"));
}

```

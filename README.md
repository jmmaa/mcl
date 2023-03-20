# Marky's Configuration Language (MCL)

Its an easy to use config toy language written in rust.


**Note: Still in development**



# Syntax

This 

```
jmmaa {

	github "https://github.com/jmmaa"

	repositories 26

    about "\nJust Me!\n"
}

languages {

	low_level {

        rust "little"

    }

	high_level {

        python "fluent"

        typescript "fluent"
    }

}

```

translates to 

```rust
// its unordered!

Table({
    Str("languages"): Table({
        Str("low_level"): Table({
            Str("rust"): Str("little")
        }), 
        Str("high_level"): Table({
            Str("typescript"): Str("fluent"), 
            Str("python"): Str("fluent")
        })
    }), 
    Str("jmmaa"): Table({
        Str("about"): String("\nJust Me!\n"),
        Str("repositories"): Integer(26),
        Str("github"): Str("https://github.com/jmmaa")
    })
})
```
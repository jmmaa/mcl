# Marky's Configuration Language (MCL)

Its an easy to use config toy language written in rust. I implemented this solely for practicing rust lifetimes.


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

Table(
    {
        "jmmaa": Table(
            {
                "about": String("\nJust Me!\n"),
                "repositories": Integer(26),
                "github": String("https://github.com/jmmaa")
            }
        ),
        "languages": Table(
            {
                "low_level": Table(
                    {
                        "rust": String("little")
                    }
                ),
                "high_level": Table(
                    {
                        "typescript": String("fluent"),
                        "python": String("fluent")
                    }
                )
            }
        )
    }
)
```
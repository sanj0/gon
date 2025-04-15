# General Object Notation

GON is JSON but not quite. It requires no quotes around keys, allows trailing
commas but doesn't require any at all and supports multiline strings.

# Format Example

```
[
   {
       name: "The Count of Monte Cristo",
       author: "Alexandre Dumas (père)",
       year: 1844,
       ratings: [
           {
                from: "***REMOVED***"
                rating: 5
                comment: "best book ever ..."
           }
       ],
   },
   "There is also a random string in this list",
   -99,
   "And a random number",
]
```

# Spelling

The rust library (and the CLI binary) can also spell Gon either minimally or
pretty and convert from and to JSON (using serde_json).

**Minimal** (`gon min` or `value.min_spell())
```
{nested:{list:[1,2,3,4,5],another_list:[{inner:"one"},"two",3,[4],["five"]]},list:["Small","list","without objects/lists"],number:-3.14,optional:None}
```

**Pretty** (`gon min <--indent-width 4 --indent-char ' ' --trailing-commas>` or `value.spell(config)`)

```
{
    nested: {
        list: [1, 2, 3, 4, 5],
        another_list: [
            {
                inner: "one",
            },
            "two",
            3,
            [4],
            ["five"],
        ],
    },
    list: ["Small", "list", "without objects/lists"],
    number: -3.14,
    optional: None,
}
```

# JSON-Conversion

Gon is compatible with JSON[^Because JSON-keys are quoted but Gon-keys not, I
assume that Gon is subset of JSON]. Conversion can be done using
`serde_json::Value::from` and `gon::Value::from` or `gon into` and `gon from`.

# Known issues/TODOs

1. Keys in objects cannot contain dashes, dollar signs and other characters that count as
   separators in klex. Maybe add a klex feature that allows dollar signs and
   dashes inside symbols

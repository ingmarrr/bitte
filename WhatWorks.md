
# Files

```ti
file test: "test.txt" {"hello world"};

file test: "test.txt" "hello world";

file test: "test.txt" {"hello world"};

file test: "test.txt" "hello world";

file foo: "voo.a"(bar: str) {"Hello {$bar$}"};

file foo(bar: str): "voo.a" {$bar$};
```

# Directories

```ti
dir foo {
    bar {
        baz
    }
}
```

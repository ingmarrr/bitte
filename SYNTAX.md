
# Strings

```ti
dir foo = baz {
    barbaz {
    foo.txt: {"# Hello World."},
    bar.md: {"# barbar
        ## baz
    "}}
}

file "bar.txt" { 
    "Hello World" 
}




let hello: str =  {" hello world "}



```

With types

```ti
let hello = {"Hello World"}
```

With input

```ti
let hello(name: str) =
    {"# Hello {$ name $}."}

let hey(req names: list<str>) =
    {$ for name in names {"
        # Hello {$ name $}.
    "}$}
```

# Files

```ti
file hello: "hello.txt" {" Hello World "}

file hello.txt {$ name $}

file "hello.txt" {" Hello World "}
```

With input for files:

```ti
file hello(name: string): "hello.txt" {"
    Hello {$ name $}
"}
```

# Directories

```ti
dir foo {
    bar {
        baz
    }
}
```

# Combining it

```ti

let prog(name: str) {"Hello {$ name $}"}

file hello: "hello.txt" = {"# Running: {$ prog $} "}

dir foo: "foobarbazdirectory" {
    bar {
        baz {
            README.md: {"# Foobar"}
        }
    }
}

dir main {
    bar.txt {"# Hello {$ prog $}.
        
        This is a test.

        ## Running {$ prog $}
    "},
    @hello,
    #foo
}
```

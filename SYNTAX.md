
# Strings

```tipis
let hello = "Hello World"
```

# Lists

```tipis
let list = ["Hello", "World"]
let list = [("John", 42), ("Jane", 24)]
```

# Insertions

```tipis
let john = "John"
let hello = "Hello $john$"
```

# Formats

```tipis
let hello(name: str) = "Hello $name$"
hello("John")
```

# For loops in insertions

```tipis
let names = ["John", "Jane", "Gustav"]
let hello(name: str) = "Hello $name"
let greeting = "Greetings: $hello(name) for name in names$"

let names_ages = [("John", 42), ("Jane", 24)]
let hello(name: str, age: int) = "Hello $name, you are $age years old"
let greetings = "Greetings: $hello(name, age) for name, age in names_ages$"
```

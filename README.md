
# Bitte

Bitte is a cli tool for generating boilerplate templates.

## The `.tp` files

Templates are defined in files ending on `.tp`. Those files contain the structure of the template, including directories, files and file content.

To make the template include unique names and dynamic additional features, `tp` supports variables.

### Variables

Currently, `tp` only supports two types, `string`'s and `list<string>`. Strings are identified with a `$` and List of strings via a `#`.

Strings are defined with single quotation marks (`'`) and list via square brackets (`[]`).

```tp
$name = 'Anakin'
#jedi = ['Obi Wan', 'Luke Skywalker']
```

Multiline strings are possible and start from the opening `'` so this:

```tp
$body = '<div>
    Hello World
</div>'
```

Will output this:

```html
<div>
    Hello World
</div>
```

### Inserting variables

To use the previously defined variables in other strings, there are to options. 

For String types, simply use the `$` followed by the name and either a whitespace or curly braces. 
The curly braces are required if you want to have append something to the variable or simply have a character following directly behind the insertion that is NOT a special character. 
Any non-alphabetic character will end the variable insertion, since variables are only allowed to have alphabetic characters. Sadly this enforces camelCase... but its the easiest option just to forbid all
non-alphabetic characters so deal with it :D.

```tp
$content = 'Hello $name'
$moreContent = 'Hello ${name}!'
```

For Lists however it gets slightly more tricky. The `#` symbol can be used to iterate over the list and then insert the each element into the same content, defined after the the variable insertion call within curly brackets.
To tell the 'compiler' where to insert the element, put a `$`. Any content behind it will be interpreted as a string. You can put multiple `$` signs to use the element multiple times.

```tp
$more = 'List of all the jedi: #jedi{'Mr. $. Its a pleasure to have you $.'}'
```

#### !Important:

When creating string content, `'`, `$`, `#`, `!` have to be escaped if you want to have those symbols in your template. You can also insert newliens `\n` and tabs `\t` (tabs are interpreted as 4 spaces).

### Initialization

If the template is supposed to support custom naming, there must be an `init` section present in the `.tp` file, which can be indicated like via `[init]`:

In the `init` section, the author can specify `required` and `optional` parameters.

```tp
[init]
required $name
optional $do_this
```

The `required` parameter needs to be passed in when creating creating an instance of the template while the `optional` parameters can be left out.

### Let Section

For `.tp` file can contain a `[let]` section in which the author can define `elements` that can take inputs and return a new !`string`! output.

In comparison to variables, elements can take inputs that can be used multiple times in the content. 

The naming is local, so if names collide, it will take the local name before the global variable and if it can find neither an error will be displayed.


```tp
[let]
!greet $greeting, $name = '<div>${greeting}, Mr. ${name}!</div>'
!greetins #names = #names{'!greet{'Hello', $}'}
```


### Structure Section

In this section, the author can define the file structure of the template. A new folder is defined by a name followed by curly braces `{}`. 

Folders/Files are comma seperated, so it doesnt really make a difference if you put them on a single line. 
Although this is ok and for small folders good, for larger/more complex structures it is recommended to actually put folder contents on a new line. 

```tp
[structure]

src { main.rs }, Cargo.toml

===

src { main.rs },
Cargo.toml

===

src {
    main.rs
},
Cargo.toml
```

The above described structure only creates empty files. 
If you want to insert content into the files, you can do that by putting a `:` behind the file name and after that either the string content directly or a variable/element.

```tp

[let]
$mainContent = 'fn main() {
    println!("Hello World.");
}'

[structure]

src { main.rs: $mainContent },
Cargo.toml
```

Of course, you can also have folders in folders:

```tp
[init]
required $projectName

[let]
$mainContent = 'fn main() {
    println!("Hello World.");
}'

$addContent = 'pub fn add(x: usize, y: usize) -> usize {
    x + y
}'

$toml_content = '[package]
name = "$projectName"
version = "0.1.0"
edition = "2021"

[dependencies]'

[structure]

src {
    math {
        add.rs: $addContent,
        sub.rs: 'pub fn sub(x: usize, y: usize) -> usize {\n\tx - y\n}',
        mod.rs: 'pub mod add;\npub mod sub;'
    },
    lib.rs: 'pub mod math;'
    main.rs: $mainContent
}
Cargo.toml: $tomlContent
```

### Partials

Partial templates are similar to `elements` in the way that they take inputs, but instead they `can` create/insert into file/folder structures.

```tp
[let]
!serviceContent #funcs = #funcs{'pub fn $_service() -> String {
    "$".to_owned()
}'}

!viewContent #funcs = #funcs{'pub fn $_view() -> String {
    <div>$</div>
}'}

[partials]
@feature $name, #functions = {
src {
    services {
        ${name}_service.rs: !serviceContent{funcs: #functions},
        mod.rs: 'pub mod $name_service;'
    }
    views {
        ${name}_view.rs: !viewContent{funcs: #functions}
        mod.rs: 'pub mod $name_view;'
    }
}}
```

This allows the easy generation of boilerplate features that repeat itself in codebases.

### Reading files

### Reading folder structures














# Files

```ti

let hey(names: list) = {{

    @for name in names {{
        @if name == "kenobi" {{ General Kenobi!  }} 
        @else {{ Hello There {$name$} }}
    }}

}}

```

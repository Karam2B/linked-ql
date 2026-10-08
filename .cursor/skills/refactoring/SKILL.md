---
name: refactoring
description: >-
    In Rust, when intructed to work following refactoring best practices, you should edit rust
    following this skill
---

# "cfg" conditional compilation
working on a refactor in rust is centered around compile gate "refactoring", to 
enable refactoring include it in "default" like this:
```Cargo.toml
[features]
default = ["refactor", ...]
refactor = []
```

to work on the stable version remove 'refactor" like this:

```Cargo.toml
[features]
default = []
refactor = []
```

while working on refactor, no changes should be ever introduced to stable, if changes needed on any part of the code, please keep old code under `cfg(not(featuren = "refactoring"))`, and new code under `cfg(not(featuren = "refactoring"))`, like this

Code in question

```rust
use crate::context::Context;

fn use_context(ctx: &mut Context) {
    ctx.method()
}

```

if for example, the prompt or refactoring instruction are telling you to deprecate "Context" in favor of "i32" and "method" in favor of incrementing, changes should be done like this:

```rust
#[cfg(feature = "refacotring")]
mod refactoring_use_context {
    fn use_context(ctx: &mut i32) {
        ctx++
    }
}

#[cfg(not(feature = "refacotring"))]
mod old_use_context {
    use crate::context::Context;

    fn use_context(ctx: &mut Context) {
        ctx.method()
    }

}


#[cfg(not(feature = "refacotring"))]
pub use old_use_context::use_context;
#[cfg(feature = "refacotring")]
pub use refacoring_use_context::use_context;

```

# Follow comments
items may contain "refactoring/refactor todo(s)" comments that should be followed to fulfill the prompt correctly. Only use comments that are subject to the propmpt, general prompt like "finish the refactor" may take all comment into consideration, while more specific prompt take only what is relavant. if instructed to complete the refactor by deleting old code, these comments should be deleted as well. Documentation not preceded by "refactoring/refactor todo(s)" should be kept.


# Stable stay unchanged
if instructed to work on the refactor
1. all `cargo test` and `cargo check` should work in stable version (default = []), while problems may exist in refactored version.
2. no changes should be ever introduced to stable.


# introducing new changes
introducing a non-breaking refactoring should be gated behind 
`cfg[feature = "refactoring"]`

# Replacement old code
introcuding a breaking change of old code or replace old code like `impl`: follow

old code should be gated under `cfg(not(feature = "refactoring"))` and new code 
behind `cfg(feature = "refactoring")`, 

do not introduce #[deprecate] attribute to the old code, even if the prompt mentions the word "deprecate"

# Use module
using cfg for `impl` or `fn` items can be messy. introdue new module for 
cleaner refactor, example:

```Rust
#[cfg(feature = "refactoring")]
// refactoring todos:
// - using new dependency
// - signature of Trait::method is kept
// - comment for this module should be comment for "crate::trait"
mod refactoring_trait {
    //! Trait 
    use crate::new_dep::Context;

    pub use trait Trait {
        fn method(self, ctx: &mut Context);
    }
}

#[cfg(not(feature = "refactoring"))]
mod stable_trait {
    // old dependencies are deprecated and gated under `cfg(not(feature = "refactoring"))`
    use crate::stable_base_trait::BaseTrait;

    pub use trait Trait: BaseTrait {
        fn method(self)
    }
}

#[cfg(feature = "refactoring")]
use refactoring_trait::*; 

#[cfg(not(feature = "refactoring"))]
use stable_trait::*;

```


# Finishing the refactor
when instructerd "finish the refactor by deleting old code" (with mention for deletion), no code should be gated under `cfg(not(feature = "refactoring"))` or `cfg(feature = "refactoring")`. like this

Before: 
```Rust
#[cfg(feature = "refactoring")]
// refactoring todos:
// - using new dependency
// - signature of Trait::method is kept
// - comment for this module should be kept
mod refactoring_trait {
    //! Trait 
    use crate::new_dep::Context;

    pub use trait Trait {
        fn method(self, ctx: &mut Context);
    }
}

#[cfg(not(feature = "refactoring"))]
mod stable_trait {
    // old dependencies are deprecated and gated under `cfg(not(feature = "refactoring"))`
    use crate::stable_base_trait::BaseTrait;

    pub use trait Trait: BaseTrait {
        fn method(self)
    }
}

#[cfg(feature = "refactoring")]
use refactoring_trait::*; 

#[cfg(not(feature = "refactoring"))]
use stable_trait::*;

```

After (src/trait):
```Rust
//! Trait 
use crate::new_dep::Context;

pub use trait Trait {
    fn method(self, ctx: &mut Context);
}
```

Note that "delete old code" has to mentioned. if just saying "finish refactoring" the meaning is that there are missing `cfg(feature = "refactoring")` items needed to fulfill the purpose of the refactor, i.e. follow all "refactoring todos"
---
name: refactoring
description: >-
    when intructed to work following refactoring best practices, you should edit rust
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

# Stable should work
all `cargo test` and `cargo check` should work in stable version (default = []), while problems may exist in refactored version.

# introducing new changes
introducing a non-breaking refactoring should be gated behind 
`cfg[feature = "refactoring"]`

# Replacement old code
sometime refactoing involves change of old code, in that case old code
should be gated under `cfg[not(feature = "refactoring")]` and new code 
behind

# Use module
using cfg for `impl` or `fn` items can be messy. introdue new module for 
cleaner refactor

```Rust
#[cfg(feature = "refactoring")]
mod refactoring_trait {
    // new dependency
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

# Follow comments
module gated under `#[cfg(feature = "refactoring")]` usually contain "refactoring todo" comments that should be followed to fulfill the prompt correctly. Only use comments that are subject to the propmpt, general prompt like "finish the refactor" may take all comment into consideration, while more specific prompt take only what is relavant

# Finishing the refactor
when instructerd "finish the refactor by deleting old code" (with mention for deletion), no code should be gated under `cfg(not(feature = "refactoring"))` or `cfg(feature = "refactoring")`. like this

Before: 
```Rust
#[cfg(feature = "refactoring")]
mod refactoring_trait {
    // new dependency
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

After:
```Rust
use crate::new_dep::Context;

pub use trait Trait {
    fn method(self, ctx: &mut Context);
}
```

Note that "delete old code" has to mentioned. if just saying "finish refactoring" the meaning is that there are missing `cfg(feature = "refactoring")` items needed to fulfill the purpose of the refactor, i.e. follow all "refactoring todos"
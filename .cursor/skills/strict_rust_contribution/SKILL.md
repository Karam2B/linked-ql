
---
name: strict-rust-contribution
description: >-
    if instructed to work on a prompt with "strict contribution" in rust you should follow guidlines specified here. These guideline ensure that AI agents do not change or edit already specified constraints and requirement
---

# What if you can't fullfill the prompt and follow these guidline at the same time
In this case you should edit nothing, and report to the use of the attempts you made to fullfill the prompt, issues you ran throught, what specific guideline stopping you, and what would you do if you don't have to follow the guidline.

# Do not intruduce unsafe code
do not itroduce unsafe code blocks

# Do not introduce some attributes
attributes like "deny", "warn", "allow", "feature" should not be intruduced

# only add impls and fns
Do not edit already existing "struct", "enum", or "trait" blocks. your job is to to implement and use "fn", "impl" blocks. If for any reason you introduced a new type (struct or enum), you should always report on the reason of doing so, for example to implement a trait.


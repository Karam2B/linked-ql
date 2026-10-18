/// User facing trait to define a link between two collections, Implemented by the
/// user to declare a link between two collections. Used internally
/// in operations::*::*Link traits.
pub trait Link<Base> {
    type Spec;
    fn spec(self) -> Self::Spec;
}

pub struct SetNew<L, D> {
    pub link: L,
    pub data: D,
}

pub struct SetId<L, Id> {
    pub link: L,
    pub id: Id,
}

pub struct Unset<L> {
    pub link: L,
}

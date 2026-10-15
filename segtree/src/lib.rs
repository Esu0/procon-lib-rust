pub mod lazy;

pub trait Monoid {
    type S: Copy;

    fn binary_operation(&self, lhs: Self::S, rhs: Self::S) -> Self::S;
    fn identity(&self) -> Self::S;
}

pub trait MapMonoid {
    type F: Copy;
    type M: Monoid;

    fn mapping(&self, f: Self::F, x: <Self::M as Monoid>::S) -> <Self::M as Monoid>::S;
    fn composition(&self, f: Self::F, g: Self::F) -> Self::F;
    fn identity_map(&self) -> Self::F;
}

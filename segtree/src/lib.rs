pub mod lazy;

pub trait Monoid {
    type S: Copy;

    fn binary_operation(&mut self, lhs: Self::S, rhs: Self::S) -> Self::S;
    fn identity(&mut self) -> Self::S;
}

pub trait MapMonoid {
    type F: Copy;
    type M: Monoid;

    fn mapping(&mut self, f: Self::F, x: <Self::M as Monoid>::S) -> <Self::M as Monoid>::S;
    fn composition(&mut self, f: Self::F, g: Self::F) -> Self::F;
    fn identity_map(&mut self) -> Self::F;
}

impl<F: Copy, M: Monoid> MapMonoid for Box<dyn MapMonoid<F = F, M = M>> {
    type F = F;
    type M = M;

    fn mapping(&mut self, f: Self::F, x: <Self::M as Monoid>::S) -> <Self::M as Monoid>::S {
        self.as_mut().mapping(f, x)
    }

    fn composition(&mut self, f: Self::F, g: Self::F) -> Self::F {
        self.as_mut().composition(f, g)
    }

    fn identity_map(&mut self) -> Self::F {
        self.as_mut().identity_map()
    }
}

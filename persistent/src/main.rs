struct A(u32);

impl A {
    fn foo(&self) -> impl Fn() {
        || {
            println!("Hello {}!", self.0);
        }
    }

    fn bar(&self) -> impl Fn() + Copy {
        || {
            println!("Hello {} bar!", self.0);
        }
    }
}

fn main() {
    let a = A(10);
    let mut foo = a.foo();
    foo = a.foo();

    let x = a.bar();
    println!("size: {}", size_of_val(&foo));
}

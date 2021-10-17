use std::fmt::Display;

pub enum Cry {
    Wouaf,
    Miaou,
}

impl Display for Cry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cry::Wouaf => write!(f, "wouaf"),
            Cry::Miaou => write!(f, "miaou"),
        }
    }
}

pub trait Animal {
    fn cry(&self) -> Cry;
    fn mov(&mut self, pos: (i32, i32));
}

pub struct Dog {
    pos: (i32, i32),
}
pub struct Cat {
    pos: (i32, i32),
}

impl Animal for Dog {
    fn cry(&self) -> Cry {
        Cry::Wouaf
    }

    fn mov(&mut self, pos: (i32, i32)) {
        self.pos = pos;
    }
}

impl Animal for Cat {
    fn cry(&self) -> Cry {
        Cry::Miaou
    }

    fn mov(&mut self, pos: (i32, i32)) {
        self.pos = pos;
    }
}

#[cfg(test)]
mod tests {
    use crate::Animal;
    use crate::Dog;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn test_cry() {
        let dog = Dog { pos: (0, 0) };
        assert_eq!(format!("{}", dog.cry()), "wouaf");
    }

    #[test]
    fn test_mov() {
        let mut dog = Dog { pos: (0, 0) };
        assert_eq!(dog.pos, (0, 0));
        dog.mov((42, 0));
        assert_eq!(dog.pos, (42, 0));
    }
}

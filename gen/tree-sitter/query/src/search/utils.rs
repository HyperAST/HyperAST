#[derive(PartialEq, Eq, Debug)]
pub struct CaptureId(u32);

#[derive(Default)]
pub struct CaptureNames(Vec<String>);

impl CaptureNames {
    pub fn intern(&mut self, name: &str) -> CaptureId {
        let p = find(&self.0, name);
        let Some(p) = p else {
            let len = self.0.len();
            self.0.push(name.to_string());
            return CaptureId(len as u32);
        };
        CaptureId(p as u32)
    }
}

fn find(s: &[String], name: &str) -> Option<u32> {
    s.iter().position(|x| x == name).map(|x| x as u32)
}

pub mod arc {
    use super::{find, CaptureId};
    use std::sync::Arc;

    pub struct CaptureNames(Arc<[String]>);
    impl CaptureNames {
        pub fn resolve(&self, name: &str) -> Option<CaptureId> {
            find(&self.0, name).map(CaptureId)
        }
    }
    impl super::CaptureNames {
        pub fn into_arc(self) -> CaptureNames {
            CaptureNames(self.0.into())
        }
    }
}

pub mod opt {
    use super::CaptureId;

    #[derive(Default)]
    pub struct CaptureNames {
        offsets: Vec<u16>,
        intern: Vec<u8>,
    }

    impl CaptureNames {
        pub fn intern(&mut self, name: &str) -> CaptureId {
            let capture = name.as_bytes();
            let i = find(&self.offsets, &self.intern, capture);
            if i == self.offsets.len() {
                self.intern.extend(capture);
                if self.intern.len() > u16::MAX as usize {
                    panic!("too long interned capture names")
                }
                self.offsets.push(self.intern.len() as u16);
            }
            CaptureId(i as u32)
        }
    }

    fn find(offsets: &[u16], intern: &[u8], name: &[u8]) -> usize {
        let mut i = 0;
        let mut o = 0u16;
        loop {
            if i == offsets.len() {
                break;
            }
            let oo = offsets[i];
            if &intern[(o as usize)..(oo as usize)] == name {
                break;
            }
            i += 1;
            if i == u32::MAX as usize {
                panic!("too many interned capture names")
            }
            o = oo;
        }
        i
    }

    pub mod arc {
        use super::{find, CaptureId};
        use std::sync::Arc;

        pub struct CaptureNames {
            offsets: Arc<[u16]>,
            intern: Arc<[u8]>,
        }
        impl CaptureNames {
            pub fn resolve(&self, name: &str) -> Option<CaptureId> {
                let i = find(&self.offsets, &self.intern, name.as_bytes());
                if i == self.offsets.len() {
                    return None;
                }
                Some(CaptureId(i as u32))
            }
        }
        impl super::CaptureNames {
            pub fn into_arc(self) -> CaptureNames {
                CaptureNames {
                    offsets: self.offsets.into(),
                    intern: self.intern.into(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let mut capture_names = CaptureNames::default();
        let id_name = capture_names.intern("name");
        assert_eq!(id_name, capture_names.intern("name"));
        let id_body = capture_names.intern("body");
        assert_eq!(id_body, capture_names.intern("body"));
        assert_eq!(id_name, capture_names.intern("name"));
        let capture_names = capture_names.into_arc();
        assert_eq!(Some(id_name), capture_names.resolve("name"));
        assert_eq!(Some(id_body), capture_names.resolve("body"));
    }

    #[test]
    fn opt() {
        let mut capture_names = opt::CaptureNames::default();
        let id_name = capture_names.intern("name");
        assert_eq!(id_name, capture_names.intern("name"));
        let id_body = capture_names.intern("body");
        assert_eq!(id_body, capture_names.intern("body"));
        assert_eq!(id_name, capture_names.intern("name"));
        let capture_names = capture_names.into_arc();
        assert_eq!(Some(id_name), capture_names.resolve("name"));
        assert_eq!(Some(id_body), capture_names.resolve("body"));
    }
}

use super::{
    estimate::Estimate, fs_container::FsContainer, max::Max, mean::Mean, min::Min,
    named_container::NamedContainer, quantile::Quantile, stats::Stats,
};
use rhai::Dynamic;

pub(crate) trait Finalize {
    type Output;
    fn finalize(self) -> Self::Output;
}

impl Finalize for Dynamic {
    type Output = Self;
    fn finalize(mut self) -> Self::Output {
        if self.is_map() {
            dbg!(&self);
            finalize_map(self.cast::<rhai::Map>())
        } else if self.is_array() {
            dbg!(&self);
            finalize_array(self.cast::<rhai::Array>())
        } else {
            dbg!(&self);
            finalize_value(&mut self);
            self
        }
    }
}

impl<T> super::finalize::Finalize for T
where
    T: Estimate,
{
    type Output = T::Output;
    fn finalize(self) -> Self::Output {
        self.estimate()
    }
}

fn finalize_value(v: &mut Dynamic) {
    if v.is::<Mean>() {
        let x: Mean = v.clone_cast();
        let x = x.finalize();
        dbg!(x);
        *v = rhai::Dynamic::from_float(x);
    } else if v.is::<Max>() {
        let x: Max = v.clone_cast();
        let x = x.finalize();
        dbg!(x);
        *v = rhai::Dynamic::from_int(x);
    } else if v.is::<Min>() {
        let x: Min = v.clone_cast();
        let x = x.finalize();
        dbg!(x);
        *v = rhai::Dynamic::from_int(x);
    } else if v.is::<Quantile>() {
        let x: Quantile = v.clone_cast();
        let x = x.finalize();
        dbg!(&x);
        match x {
            Err(e) => *v = e.into(),
            Ok(i) => *v = rhai::Dynamic::from_int(i),
        };
    } else if v.is::<Stats>() {
        let x: Stats = v.clone_cast();
        let x = x.finalize().finalize();
        dbg!(&x);
        *v = x;
    } else if v.is::<NamedContainer<Dynamic>>() {
        let x: NamedContainer<Dynamic> = v.clone_cast();
        let x = x.finalize();
        dbg!(&x);
        *v = x.into();
    } else if v.is::<FsContainer<Dynamic>>() {
        let x: FsContainer<Dynamic> = v.clone_cast();
        let x = x.finalize();
        dbg!(&x);
        *v = x.into();
    } else if v.is_array() {
        let x = std::mem::replace(v, 0.into());
        let mut x = finalize_array(x.cast::<rhai::Array>());
        std::mem::swap(&mut x, v);
    } else if v.is_map() {
        let x = std::mem::replace(v, 0.into());
        let mut x = finalize_map(x.cast::<rhai::Map>());
        std::mem::swap(&mut x, v);
    }
}

fn finalize_map(mut map: rhai::Map) -> Dynamic {
    for v in map.values_mut() {
        finalize_value(v);
    }
    map.into()
}

fn finalize_array(mut arr: rhai::Array) -> Dynamic {
    for v in arr.iter_mut() {
        finalize_value(v);
    }
    arr.into()
}

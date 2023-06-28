use rhai::Dynamic;

use super::{estimate::Estimate, max::Max, mean::Mean, min::Min, quantile::Quantile};

pub(crate) trait Finalize {
    type Output;
    fn finalize(self) -> Self::Output;
}

impl Finalize for Dynamic {
    type Output = Self;
    fn finalize(self) -> Self::Output {
        if self.is_map() {
            dbg!(&self);
            finalize_map(self.cast::<rhai::Map>())
        } else {
            dbg!(&self);
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

fn finalize_map(mut map: rhai::Map) -> Dynamic {
    for v in map.values_mut() {
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
        } else if v.is::<rhai::Map>() {
            let x = std::mem::replace(v, 0.into());
            let mut x = finalize_map(x.cast::<rhai::Map>());
            std::mem::swap(&mut x, v);
        }
    }
    map.into()
}

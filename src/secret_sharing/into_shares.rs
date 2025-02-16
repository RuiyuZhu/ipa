use crate::{
    rand::{thread_rng, Rng},
    secret_sharing::BitDecomposed,
};

pub trait IntoShares<T>: Sized {
    fn share(self) -> [T; 3] {
        self.share_with(&mut thread_rng())
    }
    fn share_with<R: Rng>(self, rng: &mut R) -> [T; 3];
}

fn vec_shares<I, U, T, R>(values: I, rng: &mut R) -> [Vec<T>; 3]
where
    I: IntoIterator<Item = U>,
    U: IntoShares<T>,
    R: Rng,
{
    let (i0, (i1, i2)) = values
        .into_iter()
        .map(|v| {
            let [v0, v1, v2] = v.share_with(rng);
            (v0, (v1, v2))
        })
        .unzip();
    [i0, i1, i2]
}

impl<I, U, T> IntoShares<Vec<T>> for I
where
    I: Iterator<Item = U>,
    U: IntoShares<T>,
{
    fn share_with<R: Rng>(self, rng: &mut R) -> [Vec<T>; 3] {
        vec_shares(self, rng)
    }
}

impl<U, T> IntoShares<BitDecomposed<T>> for BitDecomposed<U>
where
    U: IntoShares<T>,
{
    fn share_with<R: Rng>(self, rng: &mut R) -> [BitDecomposed<T>; 3] {
        vec_shares(self, rng).map(BitDecomposed::new)
    }
}

// TODO: make a macro so we can use arbitrary-sized tuples
impl IntoShares<()> for () {
    fn share_with<R: Rng>(self, _rng: &mut R) -> [(); 3] {
        [(), (), ()]
    }
}

impl<T, U, V, W> IntoShares<(T, U)> for (V, W)
where
    T: Sized,
    U: Sized,
    V: IntoShares<T>,
    W: IntoShares<U>,
{
    fn share_with<R: Rng>(self, rng: &mut R) -> [(T, U); 3] {
        let [a0, a1, a2] = self.0.share_with(rng);
        let [b0, b1, b2] = self.1.share_with(rng);
        [(a0, b0), (a1, b1), (a2, b2)]
    }
}

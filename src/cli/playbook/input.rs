use crate::ff::{Field, Fp31, Fp32BitPrime};
use crate::secret_sharing::IntoShares;
use std::any::type_name;
use std::fs::File;
use std::io;
use std::io::{stdin, BufRead, BufReader, Read};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::str::FromStr;

trait InputItem: Sized {
    fn from_str(s: &str) -> Self;
}

impl<F: Field> InputItem for F {
    fn from_str(s: &str) -> Self {
        let int_v = s.parse::<u128>().unwrap();
        F::from(int_v)
    }
}

impl InputItem for u64 {
    fn from_str(s: &str) -> Self {
        s.parse::<u64>().unwrap()
    }
}

impl<I: InputItem> InputItem for (I, I) {
    fn from_str(s: &str) -> Self {
        let mut iter = s.split(',');
        match (iter.next(), iter.next()) {
            (Some(left), Some(right)) => (I::from_str(left), I::from_str(right)),
            _ => panic!(
                "{s} is not a valid tuple of input elements: {}",
                type_name::<I>()
            ),
        }
    }
}

struct InputSource {
    inner: Box<dyn BufRead>,
}

impl InputSource {
    pub fn from_file(path: &PathBuf) -> Self {
        Self {
            inner: Box::new(BufReader::new(File::open(path).unwrap())),
        }
    }

    pub fn from_stdin() -> Self {
        Self {
            // TODO: this is suboptimal, better to use stdinlock
            inner: Box::new(BufReader::new(stdin())),
        }
    }

    #[cfg(test)]
    pub fn from_static_str(input: &'static str) -> Self {
        Self {
            inner: Box::new(BufReader::new(input.as_bytes())),
        }
    }

    pub fn iter<T: InputItem>(&mut self) -> impl Iterator<Item = T> + '_ {
        self.lines()
            .filter_map(|line| line.map(|l| T::from_str(&l)).ok())
    }
}

impl Read for InputSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl BufRead for InputSource {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.inner.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.inner.consume(amt)
    }
}

#[cfg(test)]
mod tests {
    use crate::cli::playbook::input::InputItem;
    use crate::ff::{Fp31, Fp32BitPrime};
    use crate::secret_sharing::IntoShares;
    use crate::test_fixture::Reconstruct;

    #[test]
    fn from_str() {
        assert_eq!(Fp31::from(1_u128), Fp31::from_str("1"));
        assert_eq!(Fp32BitPrime::from(0_u128), Fp32BitPrime::from_str("0"));
        assert_eq!(6_u64, u64::from_str("6"));
    }

    #[test]
    #[should_panic]
    fn parse_negative() {
        Fp31::from_str("-1");
    }

    #[test]
    #[should_panic]
    fn parse_empty() {
        Fp31::from_str("");
    }

    #[test]
    fn tuple() {
        let input = "20,27";
        let tp = <(Fp31, Fp31)>::from_str(input);
        let shares = tp.share();
        assert_eq!(
            (Fp31::from(20_u128), Fp31::from(27_u128)),
            shares.reconstruct()
        );
    }

    #[test]
    #[should_panic]
    fn tuple_parse_error() {
        <(Fp31, Fp31)>::from_str("20,");
    }

    mod input_source {
        use super::*;
        use crate::cli::playbook::input::InputSource;
        use crate::ff::Field;
        use std::io::stdin;

        #[test]
        fn multiline() {
            let expected = vec![(1_u128, 2_u128), (3, 4)];

            let mut source = InputSource::from_static_str("1,2\n3,4");
            let actual = source
                .iter::<(Fp31, Fp31)>()
                .map(|(l, r)| (l.as_u128(), r.as_u128()))
                .collect::<Vec<_>>();

            assert_eq!(expected, actual);
        }
    }
}

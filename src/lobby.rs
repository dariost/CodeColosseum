use crate::tuning::RANDOM_ID_SIZE;
use data_encoding::BASE32_DNSSEC;
use rand::Rng;

pub(crate) fn gen_random_id<T: Rng>(rng: &mut T) -> String {
    BASE32_DNSSEC.encode(&rng.gen::<[u8; RANDOM_ID_SIZE]>())
}

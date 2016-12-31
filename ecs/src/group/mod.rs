pub mod filter;

use bit_set::BitSet;
use policy::IdSet;

pub struct Group {
    entities: BitSet
}
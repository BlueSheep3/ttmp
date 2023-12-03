use serde::{Serialize, Serializer};
use std::{
	collections::{BTreeMap, HashMap, HashSet},
	hash::Hash,
};

pub fn sorted_hashmap<S, T, U>(value: &HashMap<T, U>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
	T: Ord + Hash + Serialize,
	U: Serialize,
{
	let ordered: BTreeMap<_, _> = value.iter().collect();
	ordered.serialize(serializer)
}

pub fn sorted_hashset<S, T>(map: &HashSet<T>, serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
	T: Ord + Hash + Serialize,
{
	let mut sorted_entries: Vec<_> = map.iter().collect();
	sorted_entries.sort();
	sorted_entries.serialize(serializer)
}

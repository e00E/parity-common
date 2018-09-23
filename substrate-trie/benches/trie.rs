// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

#[macro_use]
extern crate criterion;
use criterion::{Criterion, black_box, Fun};
criterion_group!(benches, trie_insertions_32_mir_1k, trie_iter, trie_insertions_32_ran_1k, trie_insertions_six_high, trie_insertions_six_mid, trie_insertions_random_mid, trie_insertions_six_low, nibble_common_prefix);
criterion_main!(benches);

extern crate memorydb;
extern crate patricia_trie as trie;
extern crate patricia_trie_ethereum as ethtrie;
extern crate substrate_trie;
extern crate keccak_hasher;
extern crate trie_standardmap;
extern crate hashdb;

use memorydb::MemoryDB;
use trie::{DBValue, NibbleSlice, TrieMut, Trie};
use ethtrie::RlpNodeCodec;
use trie_standardmap::{Alphabet, ValueMode, StandardMap};
use substrate_trie::ParityNodeCodec;
use keccak_hasher::KeccakHasher;
use hashdb::Hasher;

//use ethtrie::{TrieDB, TrieDBMut};

type H256 = <KeccakHasher as Hasher>::Out;
type TrieDB<'a> = trie::TrieDB<'a, KeccakHasher, ParityNodeCodec<KeccakHasher>>;
type TrieDBMut<'a> = trie::TrieDBMut<'a, KeccakHasher, ParityNodeCodec<KeccakHasher>>;
type RlpTrieDB<'a> = trie::TrieDB<'a, KeccakHasher, RlpNodeCodec<KeccakHasher>>;
type RlpTrieDBMut<'a> = trie::TrieDBMut<'a, KeccakHasher, RlpNodeCodec<KeccakHasher>>;

fn random_word(alphabet: &[u8], min_count: usize, diff_count: usize, seed: &mut H256) -> Vec<u8> {
	assert!(min_count + diff_count <= 32);
	*seed = KeccakHasher::hash(seed.as_ref());
	let r = min_count + (seed[31] as usize % (diff_count + 1));
	let mut ret: Vec<u8> = Vec::with_capacity(r);
	for i in 0..r {
		ret.push(alphabet[seed[i] as usize % alphabet.len()]);
	}
	ret
}

fn random_bytes(min_count: usize, diff_count: usize, seed: &mut H256) -> Vec<u8> {
	assert!(min_count + diff_count <= 32);
	*seed = KeccakHasher::hash(seed.as_ref());
	let r = min_count + (seed[31] as usize % (diff_count + 1));
	seed[0..r].to_vec()
}

fn random_value(seed: &mut H256) -> Vec<u8> {
	*seed = KeccakHasher::hash(seed.as_ref());
	match seed[0] % 2 {
		1 => vec![seed[31];1],
		_ => seed.to_vec(),
	}
}

struct TrieInsertionList(Vec<(Vec<u8>, Vec<u8>)>);
impl ::std::fmt::Display for TrieInsertionList {
	fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(fmt, "{} items", self.0.len())
	}
}

fn bench_insertions(b: &mut Criterion, name: &str, d: Vec<(Vec<u8>, Vec<u8>)>) {
	let funs = vec![
		Fun::new("Rlp", |b, d: &TrieInsertionList| b.iter(&mut ||{
			let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new();
			let mut root = H256::default();
			let mut t = RlpTrieDBMut::new(&mut memdb, &mut root);
			for i in d.0.iter() {
				t.insert(&i.0, &i.1).unwrap();
			}
		})),
		Fun::new("Codec", |b, d: &TrieInsertionList| b.iter(&mut ||{
			let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new_codec();
			let mut root = H256::default();
			let mut t = TrieDBMut::new(&mut memdb, &mut root);
			for i in d.0.iter() {
				t.insert(&i.0, &i.1).unwrap();
			}
		})),
		Fun::new("-", |b, _d: &TrieInsertionList| b.iter(&mut ||{}))
	];

	b.bench_functions(name, funs, &TrieInsertionList(d));
}

fn trie_insertions_32_mir_1k(b: &mut Criterion) {
	let st = StandardMap {
		alphabet: Alphabet::All,
		min_key: 32,
		journal_key: 0,
		value_mode: ValueMode::Mirror,
		count: 1000,
	};
	bench_insertions(b, "trie_ins_32_mir_1k", st.make());
}

fn trie_insertions_32_ran_1k(b: &mut Criterion) {
	let st = StandardMap {
		alphabet: Alphabet::All,
		min_key: 32,
		journal_key: 0,
		value_mode: ValueMode::Random,
		count: 1000,
	};
	bench_insertions(b, "trie_ins_32_ran_1k", st.make());
}

fn trie_insertions_six_high(b: &mut Criterion) {
	let mut d: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
	let mut seed = H256::default();
	for _ in 0..1000 {
		let k = random_bytes(6, 0, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}
	bench_insertions(b, "trie_ins_six_high", d);
}

fn trie_insertions_six_mid(b: &mut Criterion) {
	let alphabet = b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_";
	let mut d: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
	let mut seed = H256::default();
	for _ in 0..1000 {
		let k = random_word(alphabet, 6, 0, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}
	bench_insertions(b, "trie_ins_six_mid", d);
}

fn trie_insertions_random_mid(b: &mut Criterion) {
	let alphabet = b"@QWERTYUIOPASDFGHJKLZXCVBNM[/]^_";
	let mut d: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
	let mut seed = H256::default();
	for _ in 0..1000 {
		let k = random_word(alphabet, 1, 5, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}
	bench_insertions(b, "trie_ins_random_mid", d);
}

fn trie_insertions_six_low(b: &mut Criterion) {
	let alphabet = b"abcdef";
	let mut d: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
	let mut seed = H256::default();
	for _ in 0..1000 {
		let k = random_word(alphabet, 6, 0, &mut seed);
		let v = random_value(&mut seed);
		d.push((k, v))
	}
	bench_insertions(b, "trie_ins_six_low", d);
}

fn trie_iter(b: &mut Criterion) {
	let st = StandardMap {
		alphabet: Alphabet::All,
		min_key: 32,
		journal_key: 0,
		value_mode: ValueMode::Mirror,
		count: 1000,
	};
	let d = st.make();
	let mut memdb = MemoryDB::<KeccakHasher, DBValue>::new_codec();
	let mut root = H256::default();
	{
		let mut t = TrieDBMut::new(&mut memdb, &mut root);
		for i in d.iter() {
			t.insert(&i.0, &i.1).unwrap();
		}
	}

	b.bench_function("trie_iter", |b| b.iter(&mut ||{
		let t = TrieDB::new(&memdb, &root).unwrap();
		for n in t.iter().unwrap() {
			black_box(n).unwrap();
		}
	}));
}

fn nibble_common_prefix(b: &mut Criterion) {
	let st = StandardMap {
		alphabet: Alphabet::Custom(b"abcdef".to_vec()),
		min_key: 32,
		journal_key: 0,
		value_mode: ValueMode::Mirror,
		count: 999,
	};
	let (keys, values): (Vec<_>, Vec<_>) = st.make().iter().cloned().unzip();
	let mixed: Vec<_> = keys.iter().zip(values.iter().rev()).map(|pair| {
		(NibbleSlice::new(pair.0), NibbleSlice::new(pair.1))
	}).collect();
	b.bench_function("nibble_common_prefix", |b| b.iter(&mut ||{
		for (left, right) in mixed.iter() {
			let _ = left.common_prefix(&right);
		}
	}));
}
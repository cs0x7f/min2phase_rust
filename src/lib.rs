#[macro_use(lazy_static)]
extern crate lazy_static;

use rand::Rng;

#[derive(Clone, Copy)]
struct Cubie {
	ca: [u8; 8],
	ea: [u8; 12],
}

#[derive(Clone, Copy)]
struct Coord {
	twst: u16,
	tsym: u16,
	flip: u16,
	fsym: u16,
	slice: u16,
	prun: i8,
}

#[derive(Clone, Copy)]
struct Coord2 {
	edge: u16,
	esym: u16,
	corn: u16,
	csym: u16,
	mid: u16
}

#[repr(C)]
struct Solution {
	depth1: i8,
	verbose: u8,
	urf_idx: u8,
	premv_len: i8,
	length: i8,
	moves: [u8; 31],
}

const INVERSE_SOLUTION: u8 = 0x01;
const USE_SEPARATOR: u8 = 0x02;
const APPEND_LENGTH: u8 = 0x04;
const MAX_PREMV_LEN: i8 = 20;
const MIN_P1PRE_LEN: i8 = 7;

const N_FLIP     : usize =  2048;
const N_FLIP_SYM : usize =   336;
const N_TWST     : usize =  2187;
const N_TWST_SYM : usize =   324;
const N_SLICE    : usize =   495;
const N_PERM     : usize = 40320;
const N_PERM_SYM : usize =  2768;
const N_MPERM    : usize =    24;
const N_CCOMB    : usize =    70;

const N_MOVES_P1 : usize =    18;
const N_MOVES_P2 : usize =    10;
const MAX_DEPTH2 : usize =    13;

static MOVE2STR: [&str; 18] = ["U ", "U2", "U'", "R ", "R2", "R'", "F ", "F2", "F'", "D ", "D2", "D'", "L ", "L2", "L'", "B ", "B2", "B'"];
static URF_MOVE: [[u8; 18]; 6] = [
	[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
	[6, 7, 8, 0, 1, 2, 3, 4, 5, 15, 16, 17, 9, 10, 11, 12, 13, 14],
	[3, 4, 5, 6, 7, 8, 0, 1, 2, 12, 13, 14, 15, 16, 17, 9, 10, 11],
	[2, 1, 0, 5, 4, 3, 8, 7, 6, 11, 10, 9, 14, 13, 12, 17, 16, 15],
	[8, 7, 6, 2, 1, 0, 5, 4, 3, 17, 16, 15, 11, 10, 9, 14, 13, 12],
	[5, 4, 3, 8, 7, 6, 2, 1, 0, 14, 13, 12, 17, 16, 15, 11, 10, 9]
];

const U: u8 = 0;
const R: u8 = 9;
const F: u8 = 18;
const D: u8 = 27;
const L: u8 = 36;
const B: u8 = 45;

static CORNER_FACELET: [[u8; 3]; 8] = [
	[ U + 8, R + 0, F + 2 ], [ U + 6, F + 0, L + 2 ], [ U + 0, L + 0, B + 2 ], [ U + 2, B + 0, R + 2 ],
	[ D + 2, F + 8, R + 6 ], [ D + 0, L + 8, F + 6 ], [ D + 6, B + 8, L + 6 ], [ D + 8, R + 8, B + 6 ]
];

static EDGE_FACELET: [[u8; 2]; 12] = [
	[ U + 5, R + 1 ], [ U + 7, F + 1 ], [ U + 3, L + 1 ], [ U + 1, B + 1 ], [ D + 5, R + 7 ], [ D + 1, F + 7 ],
	[ D + 3, L + 7 ], [ D + 7, B + 7 ], [ F + 5, R + 3 ], [ F + 3, L + 5 ], [ B + 5, L + 3 ], [ B + 3, R + 5 ]
];


static P2MOVES: [u8; 18] = [0, 1, 2, 4, 7, 9, 10, 11, 13, 16, 3, 5, 6, 8, 12, 14, 15, 17];

impl Solution {
	fn append_move(&mut self, cur_move: u8) {
		if self.length == 0 {
			self.moves[self.length as usize] = cur_move;
			self.length += 1;
			return;
		}

		let cur_axis = cur_move / 3;
		let last_axis = self.moves[(self.length - 1) as usize] / 3;

		if cur_axis == last_axis {
			let pow = (cur_move % 3 + self.moves[(self.length - 1) as usize] % 3 + 1) % 4;
			if pow == 3 {
				self.length -= 1;
			} else {
				self.moves[(self.length - 1) as usize] = cur_axis * 3 + pow;
			}
			return;
		}

		if self.length > 1
			&& cur_axis % 3 == last_axis % 3
			&& cur_axis == self.moves[(self.length - 2) as usize] / 3
		{
			let pow = (cur_move % 3 + self.moves[(self.length - 2) as usize] % 3 + 1) % 4;
			if pow == 3 {
				self.moves[(self.length - 2) as usize] = self.moves[(self.length - 1) as usize];
				self.length -= 1;
			} else {
				self.moves[(self.length - 2) as usize] = cur_axis * 3 + pow;
			}
			return;
		}

		self.moves[self.length as usize] = cur_move;
		self.length += 1;
	}

	fn to_string(&self) -> String {
		let mut buf = String::new();
		let urf = if self.verbose & INVERSE_SOLUTION != 0 {
			(self.urf_idx + 3) % 6
		} else {
			self.urf_idx
		};

		if urf < 3 {
			for (s, &mv) in self.moves.iter().enumerate().take(self.length as usize) {
				if self.verbose & USE_SEPARATOR != 0 && s == self.depth1 as usize {
					buf.push_str(".  ");
				}
				buf.push_str(MOVE2STR[URF_MOVE[urf as usize][mv as usize] as usize]);
				buf.push_str(" ");
			}
		} else {
			for (s, &mv) in self.moves.iter().enumerate().take(self.length as usize).rev() {
				buf.push_str(MOVE2STR[URF_MOVE[urf as usize][mv as usize] as usize]);
				buf.push_str(" ");
				if self.verbose & USE_SEPARATOR != 0 && s == self.depth1 as usize {
					buf.push_str(".  ");
				}
			}
		}

		if self.verbose & APPEND_LENGTH != 0 {
			buf.push_str(&format!("({}f)", self.length));
		}

		buf
	}
}

#[repr(C)]
struct IdaContext {
	mv: [u8; 30],
	allow_shorter: bool,
	depth1: i8,
	length1: i8,
	valid1: i8,
	urf_idx: u8,
	p1_cubies: [Cubie; 20],
	urf_cubies: [Cubie; 6],
	premv: [u8; 15],
	premv_len: i8,
	max_depth2: i8,
	target_length: i8,
	probes: u64,
	min_probes: u64,
	solution: Solution,
}

impl Cubie {
	fn new() -> Self {
		let mut cc = Cubie {ca: [0; 8], ea: [0; 12]};
		cc.reset();
		cc
	}

	fn reset(&mut self) {
		for i in 0..8 {
			self.ca[i] = i as u8;
		}
		for i in 0..12 {
			self.ea[i] = i as u8 * 2;
		}
	}

	fn cmp(&self, other: &Cubie) -> i8 {
		for i in 0..8 {
			if self.ca[i] != other.ca[i] {
				return (self.ca[i] as i8) - (other.ca[i] as i8);
			}
		}
		for i in 0..12 {
			if self.ea[i] != other.ea[i] {
				return (self.ea[i] as i8) - (other.ea[i] as i8);
			}
		}
		0
	}

	fn corn_mult(a: &Cubie, b: &Cubie, prod: &mut Cubie) {
		for cn in 0..8 {
			let ori_a = (a.ca[(b.ca[cn] & 0x7) as usize] >> 3) as u8;
			let ori_b = (b.ca[cn] >> 3) as u8;
			let mut ori = ori_a + if ori_a < 3 { ori_b } else { 6 - ori_b };
			ori = ori % 3 + if (ori_a < 3) == (ori_b < 3) { 0 } else { 3 };
			prod.ca[cn] = (a.ca[(b.ca[cn] & 0x7) as usize] & 0x7) | (ori << 3);
		}
	}

	fn edge_mult(a: &Cubie, b: &Cubie, prod: &mut Cubie) {
		for ed in 0..12 {
			prod.ea[ed] = a.ea[(b.ea[ed] >> 1) as usize] ^ (b.ea[ed] & 1);
		}
	}

	fn inv(src: &Cubie, inv: &mut Cubie) {
		for ed in 0..12 {
			inv.ea[(src.ea[ed] >> 1) as usize] = (ed as u8 * 2) | (src.ea[ed] & 0x1);
		}
		for cn in 0..8 {
			inv.ca[(src.ca[cn] & 0x7) as usize] = cn as u8 | (((0x20 >> (src.ca[cn] >> 3)) & 0x18) as u8);
		}
	}
}

fn get_nparity(mut idx: i32, n: i32) -> i32 {
	let mut p = 0;
	let mut i = n - 2;
	while i >= 0 {
		p ^= idx % (n - i);
		idx /= n - i;
		i -= 1;
	}
	p & 1
}

fn get_nperm(arr: &[u8], n: i32) -> i32 {
	let mut idx = 0;
	let mut val = 0x76543210;
	for i in 0..(n - 1) {
		let v = arr[i as usize] << 2;
		idx = (n - i) * idx + ((val >> v) & 0xf) as i32;
		val -= 0x11111110 << v;
	}
	idx
}

fn set_nperm(arr: &mut [u8], mut idx: u16, n: u16) {
	let mut extract = 0;
	let mut val = 0x76543210;
	for i in 2..=n {
		extract = (extract << 4) | (idx % i) as u32;
		idx /= i;
	}
	for i in 0..(n - 1) {
		let v = (extract & 0xf) << 2;
		extract >>= 4;
		arr[i as usize] = ((val >> v) & 0xf) as u8;
		let m = (1 << v) - 1;
		val = (val & m) | ((val >> 4) & !m);
	}
	arr[(n - 1) as usize] = (val & 0xf) as u8;
}

fn get_comb(arr: &[u8], n: i32, mask: i32) -> i32 {
	let mut idx_c = 0;
	let mut r = 4;
	let mut cnk = if n == 12 { 330 } else { 35 };
	for i in (0..n).rev() {
		if (arr[i as usize] & 0xc) == mask as u8 {
			idx_c += cnk;
			cnk = cnk * r / std::cmp::max(1, i - r + 1);
			r -= 1;
		}
		cnk = cnk * (i - r) / std::cmp::max(1, i);
	}
	idx_c
}

fn set_comb(arr: &mut [u8], mut idx_c: i32, n: i32, mask: i32) {
	let mut r = 4;
	let mut fill = n - 1;
	let mut cnk = if n == 12 { 330 } else { 35 };
	for i in (0..n).rev() {
		if idx_c >= cnk {
			idx_c -= cnk;
			cnk = cnk * r / std::cmp::max(1, i - r + 1);
			r -= 1;
			arr[i as usize] = (r | mask) as u8;
		} else {
			if (fill & 0xc) == mask {
				fill -= 4;
			}
			arr[i as usize] = fill as u8;
			fill -= 1;
		}
		cnk = cnk * (i - r) / std::cmp::max(1, i);
	}
}

impl Cubie {
	fn get_flip(&self) -> i32 {
		let mut idx = 0;
		for i in 0..11 {
			idx = (idx << 1) | (self.ea[i] & 1) as i32;
		}
		idx
	}

	fn set_flip(&mut self, mut idx: u16) {
		let mut parity = 0;
		for i in (0..11).rev() {
			let val = idx & 1;
			idx >>= 1;
			parity ^= val;
			self.ea[i] = (self.ea[i] & !1) | (val as u8);
		}
		self.ea[11] = (self.ea[11] & !1) | (parity as u8);
	}

	fn get_twst(&self) -> i32 {
		let mut idx = 0;
		for i in 0..7 {
			idx += (idx << 1) + (self.ca[i] >> 3) as i32;
		}
		idx
	}

	fn set_twst(&mut self, mut idx: u16) {
		let mut twst = 15;
		for i in (0..7).rev() {
			let val = idx % 3;
			idx /= 3;
			twst -= val;
			self.ca[i] = (self.ca[i] & 0x7) | (val << 3) as u8;
		}
		self.ca[7] = (self.ca[7] & 0x7) | ((twst % 3) << 3) as u8;
	}

	fn get_slice(&self) -> u16 {
		let mut arr = [0u8; 12];
		for i in 0..12 {
			arr[i] = self.ea[i] >> 1;
		}
		494 - get_comb(&arr, 12, 8) as u16
	}

	fn set_slice(&mut self, idx: i32) {
		let mut arr = [0u8; 12];
		set_comb(&mut arr, 494 - idx, 12, 8);
		for i in 0..12 {
			self.ea[i] = (self.ea[i] & 1) | (arr[i] << 1);
		}
	}

	fn get_cperm(&self) -> i32 {
		let mut arr = [0u8; 8];
		for i in 0..8 {
			arr[i] = self.ca[i] & 0x7;
		}
		get_nperm(&arr, 8)
	}

	fn set_cperm(&mut self, idx: u16) {
		let mut arr = [0u8; 8];
		set_nperm(&mut arr, idx, 8);
		for i in 0..8 {
			self.ca[i] = (self.ca[i] & !0x7) | arr[i];
		}
	}

	fn get_eperm(&self) -> i32 {
		let mut arr = [0u8; 8];
		for i in 0..8 {
			arr[i] = self.ea[i] >> 1;
		}
		get_nperm(&arr, 8)
	}

	fn set_eperm(&mut self, idx: u16) {
		let mut arr = [0u8; 8];
		set_nperm(&mut arr, idx, 8);
		for i in 0..8 {
			self.ea[i] = (self.ea[i] & 1) | (arr[i] << 1);
		}
	}

	fn get_mperm(&self) -> i32 {
		let mut arr = [0u8; 4];
		for i in 8..12 {
			arr[i - 8] = (self.ea[i] >> 1) & 0x3;
		}
		get_nperm(&arr, 4)
	}

	fn set_mperm(&mut self, idx: u16) {
		let mut arr = [0u8; 4];
		set_nperm(&mut arr, idx, 4);
		for i in 8..12 {
			self.ea[i] = (self.ea[i] & 1) | ((arr[i - 8] + 8) << 1);
		}
	}

	fn get_ccomb(&self) -> i32 {
		let mut arr = [0u8; 8];
		for i in 0..8 {
			arr[i] = self.ca[i] & 0x7;
		}
		get_comb(&arr, 8, 0)
	}

	fn set_ccomb(&mut self, idx: i32) {
		let mut arr = [0u8; 8];
		set_comb(&mut arr, idx, 8, 0);
		for i in 0..8 {
			self.ca[i] = (self.ca[i] & !0x7) | arr[i];
		}
	}
}

fn esym2csym(esym: u16) -> u16 {
	esym ^ (0x00dddd00u32 >> ((esym & 0xf) << 1) & 3) as u16
}

struct StaticContext {
	movecube: [Cubie; 18],
	symcube: [Cubie; 16],
	symmult: [[u8; 16]; 16],
	symmuli: [[u8; 16]; 16],
	symmove: [[u8; 8]; N_MOVES_P1],
	symmove2: [[u8; 16]; N_MOVES_P1],
	canon_masks2: [u16; 11],
	symurf: Cubie,
	symurfi: Cubie,
}

impl StaticContext {
	fn box_new() -> Box<Self> {
		let mut sctx = Box::new(StaticContext {
			movecube: [Cubie::new(); 18],
			symcube: [Cubie::new(); 16],
			symmult: [[0; 16]; 16],
			symmuli: [[0; 16]; 16],
			symmove: [[0; 8]; 18],
			symmove2: [[0; 16]; 18],
			canon_masks2: [0; 11],
			symurf: Cubie::new(),
			symurfi: Cubie::new(),
		});
		sctx.init();
		sctx
	}

	fn init(&mut self) {
		let movebase: [Cubie; 6] = [
			Cubie {ca: [3, 0, 1, 2, 4, 5, 6, 7], ea: [6, 0, 2, 4, 8, 10, 12, 14, 16, 18, 20, 22]},
			Cubie {ca: [20, 1, 2, 8, 15, 5, 6, 19], ea: [16, 2, 4, 6, 22, 10, 12, 14, 8, 18, 20, 0]},
			Cubie {ca: [9, 21, 2, 3, 16, 12, 6, 7], ea: [0, 19, 4, 6, 8, 17, 12, 14, 3, 11, 20, 22]},
			Cubie {ca: [0, 1, 2, 3, 5, 6, 7, 4], ea: [0, 2, 4, 6, 10, 12, 14, 8, 16, 18, 20, 22]},
			Cubie {ca: [0, 10, 22, 3, 4, 17, 13, 7], ea: [0, 2, 20, 6, 8, 10, 18, 14, 16, 4, 12, 22]},
			Cubie {ca: [0, 1, 11, 23, 4, 5, 18, 14], ea: [0, 2, 4, 23, 8, 10, 12, 21, 16, 18, 7, 15]}
		];
		for i in 0..18 {
			if i % 3 == 0 {
				self.movecube[i] = movebase[i / 3];
			} else {
				let mut cc = Cubie::new();
				Cubie::corn_mult(&self.movecube[i - 1], &movebase[i / 3], &mut cc);
				Cubie::edge_mult(&self.movecube[i - 1], &movebase[i / 3], &mut cc);
				self.movecube[i] = cc;
			}
		}

		let u4 = Cubie {
			ca: [3, 0, 1, 2, 7, 4, 5, 6],
			ea: [6, 0, 2, 4, 14, 8, 10, 12, 23, 17, 19, 21]
		};
		let lr2 = Cubie {
			ca: [25, 24, 27, 26, 29, 28, 31, 30],
			ea: [4, 2, 0, 6, 12, 10, 8, 14, 18, 16, 22, 20]
		};
		let f2 = Cubie {
			ca: [5, 4, 7, 6, 1, 0, 3, 2],
			ea: [12, 10, 8, 14, 4, 2, 0, 6, 18, 16, 22, 20]
		};

		let mut cc = Cubie::new();
		let mut cd = Cubie::new();

		for i in 0..16 {
			self.symcube[i] = cc;
			Cubie::corn_mult(&cc, &u4, &mut cd);
			Cubie::edge_mult(&cc, &u4, &mut cd);
			cc = cd;
			if i % 4 == 3 {
				Cubie::corn_mult(&cc, &lr2, &mut cd);
				Cubie::edge_mult(&cc, &lr2, &mut cd);
				cc = cd;
			}
			if i % 8 == 7 {
				Cubie::corn_mult(&cc, &f2, &mut cd);
				Cubie::edge_mult(&cc, &f2, &mut cd);
				cc = cd;
			}
		}

		self.symurf = Cubie{ca: [8, 20, 13, 17, 19, 15, 22, 10], ea: [3, 16, 11, 18, 7, 22, 15, 20, 1, 9, 13, 5]};
		Cubie::corn_mult(&self.symurf, &self.symurf, &mut self.symurfi);
		Cubie::edge_mult(&self.symurf, &self.symurf, &mut self.symurfi);

		for i in 0..16 {
			for j in 0..16 {
				Cubie::corn_mult(&self.symcube[i], &self.symcube[j], &mut cc);
				Cubie::edge_mult(&self.symcube[i], &self.symcube[j], &mut cc);
				for k in 0..16 {
					if Cubie::cmp(&cc, &self.symcube[k]) == 0 {
						self.symmult[i][j] = k as u8;
						self.symmuli[k][j] = i as u8;
					}
				}
			}
		}

		let mut p2moves_imap = [0; 18];
		for i in 0..18 {
			p2moves_imap[P2MOVES[i] as usize] = i;
		}

		for i in 0..18 {
			for j in 0..16 {
				Cubie::corn_mult(&self.symcube[j], &self.movecube[i], &mut cc);
				Cubie::corn_mult(&cc, &self.symcube[self.symmuli[0][j] as usize], &mut cd);
				Cubie::edge_mult(&self.symcube[j], &self.movecube[i], &mut cc);
				Cubie::edge_mult(&cc, &self.symcube[self.symmuli[0][j] as usize], &mut cd);
				for k in 0..18 {
					if Cubie::cmp(&self.movecube[k], &cd) == 0 {
						self.symmove2[p2moves_imap[i] as usize][j] = p2moves_imap[k] as u8;
						if j % 2 == 0 {
							self.symmove[i][j / 2] = k as u8;
						}
						break;
					}
				}
			}
		}

		for i in 0..10 {
			let ix = P2MOVES[i] as usize / 3;
			self.canon_masks2[i] = 0;
			for j in 0..10 {
				let jx = P2MOVES[j] as usize / 3;
				self.canon_masks2[i] |= if (ix == jx) || ((ix % 3 == jx % 3) && (ix >= jx)) { 1 } else { 0 } << j;
			}
		}
		self.canon_masks2[10] = 0;
	}
}

fn init_sym2raw(
	sctx: &StaticContext, n_raw: usize, coord: usize,
	sym2raw: &mut [u16], raw2sym: &mut [u16], selfsym: &mut [u16],
) -> usize {
	let mut c = Cubie::new();
	let mut e = Cubie::new();
	let mut d = Cubie::new();
	let sym_inc = if coord >= 2 { 1 } else { 2 };
	let sym_shift = if coord >= 2 { 0 } else { 1 };
	for i in 0..n_raw {
		raw2sym[i] = 0;
	}
	let mut count = 0;
	for i in 0..n_raw {
		if raw2sym[i] != 0 {
			continue;
		}
		match coord {
			0 => c.set_flip(i as u16),
			1 => c.set_twst(i as u16),
			2 => c.set_eperm(i as u16),
			_ => unreachable!(),
		}
		for s in (0..16).step_by(sym_inc) {
			if coord == 1 {
				Cubie::corn_mult(&sctx.symcube[sctx.symmuli[0][s as usize] as usize], &c, &mut e);
				Cubie::corn_mult(&e, &sctx.symcube[s as usize], &mut d);
			} else {
				Cubie::edge_mult(&sctx.symcube[sctx.symmuli[0][s as usize] as usize], &c, &mut e);
				Cubie::edge_mult(&e, &sctx.symcube[s as usize], &mut d);
			}
			let idx = match coord {
				0 => d.get_flip(),
				1 => d.get_twst(),
				2 => d.get_eperm(),
				_ => unreachable!(),
			};
			if idx == i as i32 {
				selfsym[count] |= 1 << (s >> sym_shift);
			}
			raw2sym[idx as usize] = ((count << 4 | s) >> sym_shift) as u16;
		}
		sym2raw[count] = i as u16;
		count += 1;
	}
	#[cfg(debug_assertions)]
	println!("init sym2raw coord={} count={}", coord, count);
	count
}

struct StaticTables {
	perm_sym_inv : [u16; N_PERM_SYM],
	cperm2comb   : [u8; N_PERM_SYM],
	flip_sym2raw : [u16; N_FLIP_SYM],
	flip_raw2sym : [u16; N_FLIP],
	flip_selfsym : [u16; N_FLIP_SYM],
	twst_sym2raw : [u16; N_TWST_SYM],
	twst_raw2sym : [u16; N_TWST],
	twst_selfsym : [u16; N_TWST_SYM],
	eperm_sym2raw: [u16; N_PERM_SYM],
	eperm_raw2sym: [u16; N_PERM],
	eperm_selfsym: [u16; N_PERM_SYM],
	flip_move    : [u16; N_FLIP_SYM * N_MOVES_P1],
	twst_move    : [u16; N_TWST_SYM * N_MOVES_P1],
	slice_move   : [u16; N_SLICE * N_MOVES_P1],
	slice_conj   : [u16; N_SLICE * 8],
	cperm_move   : [u16; N_PERM_SYM * N_MOVES_P2],
	eperm_move   : [u16; N_PERM_SYM * N_MOVES_P2],
	mperm_move   : [u16; N_MPERM * N_MOVES_P2],
	mperm_conj   : [u16; N_MPERM * 16],
	ccomb_move   : [u16; N_CCOMB * N_MOVES_P2],
	ccomb_conj   : [u16; N_CCOMB * 16],
	slice_flip_prun : [u32; N_SLICE * N_FLIP_SYM / 8 + 1],
	slice_twst_prun : [u32; N_SLICE * N_TWST_SYM / 8 + 1],
	ccomb_eperm_prun: [u32; N_CCOMB * N_PERM_SYM / 8 + 1],
	mperm_cperm_prun: [u32; N_MPERM * N_PERM_SYM / 8 + 1],
}

fn init_move_tables(sctx: &StaticContext, stbl: &mut StaticTables) {
	let mut c = Cubie::new();
	c.reset();

	for i in 0..N_FLIP_SYM {
		c.set_flip(stbl.flip_sym2raw[i]);
		for j in 0..N_MOVES_P1 {
			let mut d = Cubie::new();
			Cubie::edge_mult(&c, &sctx.movecube[j], &mut d);
			stbl.flip_move[i * N_MOVES_P1 + j] = stbl.flip_raw2sym[d.get_flip() as usize];
		}
	}

	for i in 0..N_TWST_SYM {
		c.set_twst(stbl.twst_sym2raw[i]);
		for j in 0..N_MOVES_P1 {
			let mut d = Cubie::new();
			Cubie::corn_mult(&c, &sctx.movecube[j], &mut d);
			stbl.twst_move[i * N_MOVES_P1 + j] = stbl.twst_raw2sym[d.get_twst() as usize];
		}
	}

	for i in 0..N_SLICE {
		c.set_slice(i as i32);
		for j in 0..N_MOVES_P1 {
			let mut d = Cubie::new();
			Cubie::edge_mult(&c, &sctx.movecube[j], &mut d);
			stbl.slice_move[i * N_MOVES_P1 + j] = d.get_slice();
		}
		for j in 0..8 {
			let mut e = Cubie::new();
			let mut d = Cubie::new();
			Cubie::edge_mult(&sctx.symcube[j << 1], &c, &mut e);
			Cubie::edge_mult(&e, &sctx.symcube[j << 1], &mut d);
			stbl.slice_conj[i * 8 + j] = d.get_slice();
		}
	}

	c.reset();
	for i in 0..N_PERM_SYM {
		c.set_cperm(stbl.eperm_sym2raw[i]);
		c.set_eperm(stbl.eperm_sym2raw[i]);
		for j in 0..N_MOVES_P2 {
			let mut d = Cubie::new();
			Cubie::corn_mult(&c, &sctx.movecube[P2MOVES[j] as usize], &mut d);
			Cubie::edge_mult(&c, &sctx.movecube[P2MOVES[j] as usize], &mut d);
			stbl.cperm_move[i * N_MOVES_P2 + j] = esym2csym(stbl.eperm_raw2sym[d.get_cperm() as usize]);
			stbl.eperm_move[i * N_MOVES_P2 + j] = stbl.eperm_raw2sym[d.get_eperm() as usize];
		}
		let mut d = Cubie::new();
		Cubie::inv(&c, &mut d);
		stbl.perm_sym_inv[i] = stbl.eperm_raw2sym[d.get_eperm() as usize];
		stbl.cperm2comb[i] = c.get_ccomb() as u8;
	}

	for i in 0..N_MPERM {
		c.set_mperm(i as u16);
		for j in 0..N_MOVES_P2 {
			let mut d = Cubie::new();
			Cubie::edge_mult(&c, &sctx.movecube[P2MOVES[j] as usize], &mut d);
			stbl.mperm_move[i * N_MOVES_P2 + j] = d.get_mperm() as u16;
		}
		for j in 0..16 {
			let mut e = Cubie::new();
			let mut d = Cubie::new();
			Cubie::edge_mult(&sctx.symcube[j], &c, &mut e);
			Cubie::edge_mult(&e, &sctx.symcube[sctx.symmuli[0][j] as usize], &mut d);
			stbl.mperm_conj[i * 16 + j] = d.get_mperm() as u16;
		}
	}

	for i in 0..N_CCOMB {
		c.set_ccomb(i as i32);
		for j in 0..N_MOVES_P2 {
			let mut d = Cubie::new();
			Cubie::corn_mult(&c, &sctx.movecube[P2MOVES[j] as usize], &mut d);
			stbl.ccomb_move[i * N_MOVES_P2 + j] = d.get_ccomb() as u16;
		}
		for j in 0..16 {
			let mut e = Cubie::new();
			let mut d = Cubie::new();
			Cubie::corn_mult(&sctx.symcube[j], &c, &mut e);
			Cubie::corn_mult(&e, &sctx.symcube[sctx.symmuli[0][j] as usize], &mut d);
			stbl.ccomb_conj[i * 16 + j] = d.get_ccomb() as u16;
		}
	}
}

fn set_pruning(table: &mut [u32], index: usize, value: u32) {
	table[index >> 3] ^= value << ((index & 7) << 2);
}

fn get_pruning(table: &[u32], index: usize) -> u32 {
	(table[index >> 3] >> ((index & 7) << 2)) & 0xf
}

fn init_raw_sym_prun(
	prun_table: &mut [u32],
	raw_move: &[u16],
	raw_conj: &[u16],
	sym_move: &[u16],
	sym_selfsym: &[u16],
	n_raw: usize,
	n_sym: usize,
	prun_flag: usize,
) {
	let sym_shift: usize = prun_flag & 0xf;
	let sym_e2c_magic: usize = if (prun_flag >> 4) & 1 == 1 { 0x00DDDD00 } else { 0 };
	let is_phase2: usize = if (prun_flag >> 5) & 1 == 1 { 1 } else { 0 };
	let inv_depth: usize = (prun_flag >> 8) & 0xf;
	let max_depth: usize = (prun_flag >> 12) & 0xf;

	let sym_mask: usize = (1 << sym_shift) - 1;
	let n_entries: usize = n_raw * n_sym;
	let n_moves: usize = if is_phase2 != 0 { N_MOVES_P2 } else { N_MOVES_P1 };

	let mut depth: usize = 0;
	let mut _done: usize = 1;

	for i in 0..(n_entries / 8 + 1) {
		prun_table[i] = 0xffffffff;
	}
	set_pruning(prun_table, 0, 0xf);

	while depth < max_depth as usize {
		let inv = depth > inv_depth;
		let select = (if inv { 0xf } else { depth }) as u32;
		let check = (if inv { depth } else { 0xf }) as u32;
		depth += 1;
		let xor_val = (depth ^ 0xf) as u32;
		let mut val: u32 = 0;
		let mut i = 0;
		while i < n_entries {
			if (i & 7) == 0 {
				val = prun_table[i >> 3];
				if !inv && val == 0xffffffff {
					i += 8;
					continue;
				}
			}

			if (val & 0xf) != select {
				i += 1;
				val >>= 4;
				continue;
			}

			let raw = i % n_raw;
			let sym = i / n_raw;

			for m in 0..n_moves {
				let symx = sym_move[sym * n_moves + m] as usize;
				let rawx = raw_conj[(raw_move[raw * n_moves + m] as usize) << sym_shift | (symx & sym_mask)] as usize;
				let symx = symx >> sym_shift;
				let idx = symx * n_raw + rawx;
				let prun = get_pruning(prun_table, idx);

				if prun != check {
					continue;
				}

				_done += 1;
				if inv {
					set_pruning(prun_table, i, xor_val);
					break;
				}

				set_pruning(prun_table, idx, xor_val);
				let idx = idx - rawx;

				for j in 1..=15 {
					let ssmask = sym_selfsym[symx as usize];
					if (ssmask >> j) & 1 == 0 {
						continue;
					}
					let idxx = idx + raw_conj[((rawx << sym_shift) | (j ^ (sym_e2c_magic >> (j << 1) & 3))) as usize] as usize;
					if get_pruning(prun_table, idxx) == check {
						set_pruning(prun_table, idxx, xor_val);
						_done += 1;
					}
				}
			}
			i += 1;
			val >>= 4;
		}
		#[cfg(debug_assertions)]
		println!("depth={:2} entry_cnt={:10}", depth, _done);
	}
}

impl StaticTables {
	fn box_new(sctx: &StaticContext) -> Box<Self> {
		let mut stbl = Box::new(StaticTables {
			perm_sym_inv : [0u16; N_PERM_SYM],
			cperm2comb   : [0u8; N_PERM_SYM],
			flip_sym2raw : [0u16; N_FLIP_SYM],
			flip_raw2sym : [0u16; N_FLIP],
			flip_selfsym : [0u16; N_FLIP_SYM],
			twst_sym2raw : [0u16; N_TWST_SYM],
			twst_raw2sym : [0u16; N_TWST],
			twst_selfsym : [0u16; N_TWST_SYM],
			eperm_sym2raw: [0u16; N_PERM_SYM],
			eperm_raw2sym: [0u16; N_PERM],
			eperm_selfsym: [0u16; N_PERM_SYM],
			flip_move    : [0u16; N_FLIP_SYM * N_MOVES_P1],
			twst_move    : [0u16; N_TWST_SYM * N_MOVES_P1],
			slice_move   : [0u16; N_SLICE * N_MOVES_P1],
			slice_conj   : [0u16; N_SLICE * 8],
			cperm_move   : [0u16; N_PERM_SYM * N_MOVES_P2],
			eperm_move   : [0u16; N_PERM_SYM * N_MOVES_P2],
			mperm_move   : [0u16; N_MPERM * N_MOVES_P2],
			mperm_conj   : [0u16; N_MPERM * 16],
			ccomb_move   : [0u16; N_CCOMB * N_MOVES_P2],
			ccomb_conj   : [0u16; N_CCOMB * 16],
			slice_flip_prun : [0u32; N_SLICE * N_FLIP_SYM / 8 + 1],
			slice_twst_prun : [0u32; N_SLICE * N_TWST_SYM / 8 + 1],
			ccomb_eperm_prun: [0u32; N_CCOMB * N_PERM_SYM / 8 + 1],
			mperm_cperm_prun: [0u32; N_MPERM * N_PERM_SYM / 8 + 1],
		});
		stbl.init(&sctx);
		stbl
	}

	fn init(&mut self, sctx: &StaticContext) {
		init_sym2raw(sctx, N_FLIP, 0, &mut self.flip_sym2raw, &mut self.flip_raw2sym, &mut self.flip_selfsym);
		init_sym2raw(sctx, N_TWST, 1, &mut self.twst_sym2raw, &mut self.twst_raw2sym, &mut self.twst_selfsym);
		init_sym2raw(sctx, N_PERM, 2, &mut self.eperm_sym2raw, &mut self.eperm_raw2sym, &mut self.eperm_selfsym);
		init_move_tables(sctx, self);
		init_raw_sym_prun(&mut self.slice_twst_prun, &self.slice_move, &self.slice_conj, &self.twst_move, &self.twst_selfsym, N_SLICE, N_TWST_SYM, 0x69603);
		init_raw_sym_prun(&mut self.slice_flip_prun, &self.slice_move, &self.slice_conj, &self.flip_move, &self.flip_selfsym, N_SLICE, N_FLIP_SYM, 0x69603);
		init_raw_sym_prun(&mut self.ccomb_eperm_prun, &self.ccomb_move, &self.ccomb_conj, &self.eperm_move, &self.eperm_selfsym, N_CCOMB, N_PERM_SYM, 0x7c824);
		init_raw_sym_prun(&mut self.mperm_cperm_prun, &self.mperm_move, &self.mperm_conj, &self.cperm_move, &self.eperm_selfsym, N_MPERM, N_PERM_SYM, 0x8ea34);
	}
}

fn get_perm_sym_inv(sctx: &StaticContext, stbl: &StaticTables, idx: u16, sym: u16, is_corner: i32) -> u16 {
	let idxi = stbl.perm_sym_inv[idx as usize];
	let mut result = if is_corner != 0 {
		esym2csym(idxi)
	} else {
		idxi
	};
	result = (result & 0xfff0) | (sctx.symmult[result as usize & 0xf][sym as usize] as u16);
	result
}

impl Coord {
	fn new() -> Self {
		Coord {
			twst: 0u16,
			tsym: 0u16,
			flip: 0u16,
			fsym: 0u16,
			slice: 0u16,
			prun: 0i8
		}
	}

	fn from_cubie(&mut self, stbl: &StaticTables, src: &Cubie) -> i8 {
		self.slice = src.get_slice();
		self.flip = stbl.flip_raw2sym[src.get_flip() as usize];
		self.fsym = self.flip & 7;
		self.flip >>= 3;
		self.twst = stbl.twst_raw2sym[src.get_twst() as usize];
		self.tsym = self.twst & 7;
		self.twst >>= 3;
		self.prun = std::cmp::max(
			get_pruning(&stbl.slice_twst_prun, self.twst as usize * N_SLICE + stbl.slice_conj[(self.slice * 8 + self.tsym) as usize] as usize),
			get_pruning(&stbl.slice_flip_prun, self.flip as usize * N_SLICE + stbl.slice_conj[(self.slice * 8 + self.fsym) as usize] as usize)
		) as i8;
		self.prun
	}

	fn move_prun(&mut self, sctx: &StaticContext, stbl: &StaticTables, src: &Coord, mv: usize) -> i8 {
		self.slice = stbl.slice_move[src.slice as usize * N_MOVES_P1 + mv];
		self.flip = stbl.flip_move[src.flip as usize * N_MOVES_P1 + sctx.symmove[mv][src.fsym as usize] as usize];
		self.fsym = (self.flip & 7) ^ src.fsym;
		self.flip >>= 3;
		self.twst = stbl.twst_move[src.twst as usize * N_MOVES_P1 + sctx.symmove[mv][src.tsym as usize] as usize];
		self.tsym = (self.twst & 7) ^ src.tsym;
		self.twst >>= 3;
		self.prun = std::cmp::max(
			get_pruning(&stbl.slice_twst_prun, self.twst as usize * N_SLICE + stbl.slice_conj[(self.slice * 8 + self.tsym) as usize] as usize),
			get_pruning(&stbl.slice_flip_prun, self.flip as usize * N_SLICE + stbl.slice_conj[(self.slice * 8 + self.fsym) as usize] as usize)
		) as i8;
		self.prun
	}
}

impl Coord2 {
	fn new() -> Self {
		Coord2 {
			edge: 0u16,
			esym: 0u16,
			corn: 0u16,
			csym: 0u16,
			mid: 0u16
		}
	}

	fn from_cubie(&mut self, sctx: &StaticContext, stbl: &StaticTables, src: &Cubie) -> i8 {
		self.corn = esym2csym(stbl.eperm_raw2sym[src.get_cperm() as usize]) as u16;
		self.csym = self.corn & 0xf;
		self.corn = self.corn >> 4;
		self.edge = stbl.eperm_raw2sym[src.get_eperm() as usize] as u16;
		self.esym = self.edge & 0xf;
		self.edge = self.edge >> 4;
		self.mid = src.get_mperm() as u16;
		let edgei = get_perm_sym_inv(sctx, stbl, self.edge, self.esym, 0);
		let corni = get_perm_sym_inv(sctx, stbl, self.corn, self.csym, 1);
		std::cmp::max(
			get_pruning(&stbl.ccomb_eperm_prun, (edgei >> 4) as usize * N_CCOMB + stbl.ccomb_conj[stbl.cperm2comb[corni as usize >> 4] as usize * 16 + sctx.symmuli[edgei as usize & 0xf][corni as usize & 0xf] as usize] as usize),
			std::cmp::max(
				get_pruning(&stbl.ccomb_eperm_prun, self.edge as usize * N_CCOMB + stbl.ccomb_conj[stbl.cperm2comb[self.corn as usize] as usize * 16 + sctx.symmuli[self.esym as usize][self.csym as usize] as usize] as usize),
				get_pruning(&stbl.mperm_cperm_prun, self.corn as usize * N_MPERM + stbl.mperm_conj[self.mid as usize * 16 + self.csym as usize] as usize)
			)
		) as i8
	}
}

impl IdaContext {
	pub fn new() -> Self {
		IdaContext {
			mv: [0; 30],
			allow_shorter: false,
			depth1: 0,
			length1: 0,
			valid1: 0,
			urf_idx: 0,
			p1_cubies: [Cubie::new(); 20],
			urf_cubies: [Cubie::new(); 6],
			premv: [0; 15],
			premv_len: 0,
			max_depth2: 0,
			target_length: 0,
			probes: 0,
			min_probes: 0,
			solution: Solution {
				depth1: 0,
				verbose: 0,
				urf_idx: 0,
				premv_len: 0,
				length: 0,
				moves: [0; 31],
			},
		}
	}

	pub fn solve_cubie(&mut self, sctx: &StaticContext, stbl: &StaticTables, cc: &Cubie, target_length: i8) -> String {
		let mut cc1 = *cc;
		let mut cc2 = Cubie::new();
		self.target_length = target_length + 1;
		self.probes = 0;
		for i in 0..6 {
			self.urf_cubies[i] = cc1;
			Cubie::corn_mult(&sctx.symurfi, &cc1, &mut cc2);
			Cubie::edge_mult(&sctx.symurfi, &cc1, &mut cc2);
			Cubie::corn_mult(&cc2, &sctx.symurf, &mut cc1);
			Cubie::edge_mult(&cc2, &sctx.symurf, &mut cc1);
			if i == 2 {
				Cubie::inv(&cc1, &mut cc2);
				cc1 = cc2;
			}
		}
		for length1 in 0..21 {
			self.length1 = length1;
			self.max_depth2 = std::cmp::min(MAX_DEPTH2 as i8, self.target_length as i8 - self.length1 as i8 - 1);
			self.depth1 = self.length1 - self.premv_len;
			self.allow_shorter = false;
			for urf_idx in 0..6 {
				self.urf_idx = urf_idx;
				let cc = self.urf_cubies[self.urf_idx as usize];
				let ret = self.phase1_pre_moves(sctx, stbl, MAX_PREMV_LEN as i8, -30, &cc, 0);
				if ret == 0 {
					let solbuf = self.solution.to_string();
					#[cfg(debug_assertions)]
					println!("solution found in {:2}+{:2} moves urf={} premv={} probe={:5}: {}",
						self.length1, self.solution.length - self.length1,
						self.solution.urf_idx, self.solution.premv_len, self.probes, solbuf);
					return solbuf;
				}
			}
		}
		String::from("Error 8")
	}

	fn phase1_pre_moves(&mut self, sctx: &StaticContext, stbl: &StaticTables,
			maxl: i8, lm: i8, cc: &Cubie, _ssym: i32) -> i32 {
		self.premv_len = MAX_PREMV_LEN - maxl;
		if self.premv_len == 0 || ((0o667667 >> lm) & 1) == 0 {
			self.depth1 = self.length1 - self.premv_len;
			self.allow_shorter = self.depth1 == MIN_P1PRE_LEN && self.premv_len != 0;
			self.p1_cubies[0] = *cc;
			let mut node = Coord::new();
			if node.from_cubie(stbl, &self.p1_cubies[0]) <= self.depth1 {
				let ret = self.phase1(sctx, stbl, &node, 0, self.depth1, -1);
				if ret == 0 {
					return 0;
				}
			}
		}
		if maxl == 0 || self.premv_len + MIN_P1PRE_LEN >= self.length1 {
			return 1;
		}
		let skip_moves = if maxl == 1 || self.premv_len + 1 + MIN_P1PRE_LEN >= self.length1 {
			0o227227
		} else {
			0
		};
		let mut cd = Cubie::new();
		let lm = lm / 3;
		for m in 0..18 {
			if m / 3 == lm || m / 3 == lm - 3 || m / 3 == lm + 3 {
				continue;
			}
			if (skip_moves & (1 << m)) != 0 {
				continue;
			}
			Cubie::corn_mult(&sctx.movecube[m as usize], cc, &mut cd);
			Cubie::edge_mult(&sctx.movecube[m as usize], cc, &mut cd);
			self.premv[MAX_PREMV_LEN as usize - maxl as usize] = m as u8;
			let ret = self.phase1_pre_moves(sctx, stbl, maxl - 1, m, &cd, 0);
			if ret == 0 {
				return 0;
			}
		}
		1
	}

	fn phase1(&mut self, sctx: &StaticContext, stbl: &StaticTables,
			node: &Coord, _ssym: i32, maxl: i8, lm: i8) -> i8 {
		let mut next_node: Coord = Coord::new();
		if node.prun == 0 && maxl < 5 {
			if self.allow_shorter || maxl == 0 {
				self.depth1 -= maxl;
				let ret = self.init_phase2(sctx, stbl);
				self.depth1 += maxl;
				return ret;
			} else {
				return 1;
			}
		}
		for axis in (0..N_MOVES_P1 as i8).step_by(3) {
			if axis == lm || axis == lm - 9 {
				continue;
			}
			for power in 0..3 {
				let m = axis + power;
				let prun = next_node.move_prun(sctx, stbl, node, m as usize);
				if prun > maxl {
					break;
				} else if prun == maxl {
					continue;
				}
				self.mv[self.depth1 as usize - maxl as usize] = m as u8;
				self.valid1 = self.valid1.min(self.depth1 - maxl);
				let ret = self.phase1(sctx, stbl, &next_node, 0, maxl - 1, axis);
				if ret == 0 {
					return 0;
				} else if ret >= 2 {
					break;
				}
			}
		}
		1
	}

	fn init_phase2(&mut self, sctx: &StaticContext, stbl: &StaticTables) -> i8 {
		self.probes += 1;
		let mut cc = Cubie::new();
		for i in self.valid1 as usize..self.depth1 as usize {
			Cubie::corn_mult(&self.p1_cubies[i], &sctx.movecube[self.mv[i] as usize], &mut cc);
			Cubie::edge_mult(&self.p1_cubies[i], &sctx.movecube[self.mv[i] as usize], &mut cc);
			self.p1_cubies[i + 1] = cc;
		}
		self.valid1 = self.depth1;
		let mut node1 = Coord2::new();
		let mut prun = node1.from_cubie(sctx, stbl, &cc);
		let mut node2 = Coord2::new();
		if self.premv_len > 0 {
			let m = self.premv[self.premv_len as usize - 1] as usize / 3 * 3 + 1;
			let mut cd = Cubie::new();
			Cubie::corn_mult(&sctx.movecube[m], &cc, &mut cd);
			Cubie::edge_mult(&sctx.movecube[m], &cc, &mut cd);
			prun = prun.min(node2.from_cubie(sctx, stbl, &cd));
		}
		if prun > self.max_depth2 {
			return prun - self.max_depth2;
		}
		let mut depth2 = self.max_depth2;
		while depth2 >= prun {
			let mut sol_src = 0;
			let mut ret = self.phase2(sctx, stbl, &node1, depth2, self.depth1, 10);
			if ret < 0 && self.premv_len > 0 {
				sol_src = 1;
				ret = self.phase2(sctx, stbl, &node2, depth2, self.depth1, 10);
			}
			if ret < 0 {
				break;
			}
			depth2 -= ret;
			self.target_length = 0;
			self.solution.length = 0;
			self.solution.urf_idx = self.urf_idx;
			self.solution.depth1 = self.depth1;
			self.solution.premv_len = self.premv_len;
			for i in 0..self.depth1 + depth2 {
				self.solution.append_move(self.mv[i as usize]);
			}
			if sol_src == 1 {
				self.solution.append_move(self.premv[self.premv_len as usize - 1] / 3 * 3 + 1);
			}
			for i in (0..self.premv_len).rev() {
				self.solution.append_move(self.premv[i as usize]);
			}
			self.target_length = self.solution.length;
			depth2 -= 1;
		}

		if depth2 != self.max_depth2 {
			self.max_depth2 = std::cmp::min(MAX_DEPTH2 as i8, self.target_length as i8 - self.length1 - 1);
			return if self.probes >= self.min_probes { 0 } else { 1 };
		}
		1
	}

	fn phase2(&mut self, sctx: &StaticContext, stbl: &StaticTables,
			node: &Coord2, maxl: i8, depth: i8, lm: i8) -> i8 {
		if node.edge == 0 && node.corn == 0 && node.mid == 0 {
			return maxl;
		}
		let move_mask = sctx.canon_masks2[lm as usize];
		let mut nodex = Coord2::new();
		for m in 0..N_MOVES_P2 {
			if (move_mask >> m & 1) != 0 {
				continue;
			}
			nodex.mid = stbl.mperm_move[node.mid as usize * N_MOVES_P2 + m];
			nodex.corn = stbl.cperm_move[node.corn as usize * N_MOVES_P2 + sctx.symmove2[m][node.csym as usize] as usize];
			nodex.csym = sctx.symmult[nodex.corn as usize & 0xf][node.csym as usize] as u16;
			nodex.corn = nodex.corn >> 4;
			nodex.edge = stbl.eperm_move[node.edge as usize * N_MOVES_P2 + sctx.symmove2[m][node.esym as usize] as usize];
			nodex.esym = sctx.symmult[nodex.edge as usize & 0xf][node.esym as usize] as u16;
			nodex.edge = nodex.edge >> 4;
			let edgei = get_perm_sym_inv(sctx, stbl, nodex.edge, nodex.esym, 0) as usize;
			let corni = get_perm_sym_inv(sctx, stbl, nodex.corn, nodex.csym, 1) as usize;
			let prun = get_pruning(&stbl.ccomb_eperm_prun,
				(edgei >> 4) as usize * N_CCOMB +
				stbl.ccomb_conj[stbl.cperm2comb[corni as usize >> 4] as usize * 16 + sctx.symmuli[edgei as usize & 0xf][corni as usize & 0xf] as usize] as usize) as i8;
			if prun > maxl + 1 {
				return maxl - prun + 1;
			} else if prun >= maxl {
				continue;
			}
			let prun = std::cmp::max(
				get_pruning(&stbl.mperm_cperm_prun, nodex.corn as usize * N_MPERM + stbl.mperm_conj[nodex.mid as usize * 16 + nodex.csym as usize] as usize),
				get_pruning(&stbl.ccomb_eperm_prun, nodex.edge as usize * N_CCOMB + stbl.ccomb_conj[stbl.cperm2comb[nodex.corn as usize] as usize * 16 + sctx.symmuli[nodex.esym as usize][nodex.csym as usize] as usize] as usize)
			) as i8;
			if prun >= maxl {
				continue;
			}
			let ret = self.phase2(sctx, stbl, &nodex, maxl - 1, depth + 1, m as i8);
			if ret >= 0 {
				self.mv[depth as usize] = P2MOVES[m];
				return ret;
			} else if ret < -2 {
				break;
			}
		}
		-1
	}
}

impl Cubie {
	pub fn verify(&self) -> i32 {
		let mut sum = 0;
		let mut edge_mask = 0;
		for e in 0..12 {
			edge_mask |= 1 << (self.ea[e] >> 1);
			sum ^= self.ea[e] & 1;
		}
		if edge_mask != 0xfff {
			return -2;
		} else if sum != 0 {
			return -3;
		}
		let mut corn_mask = 0;
		for c in 0..8 {
			corn_mask |= 1 << (self.ca[c] & 7);
			sum += self.ca[c] >> 3;
		}
		if corn_mask != 0xff {
			return -4;
		} else if sum % 3 != 0 {
			return -5;
		}
		let mut parity = get_nparity(self.get_cperm(), 8);
		let mut ea: [u8; 12] = self.ea;
		for i in 0..12 {
			while ((ea[i] as usize) >> 1) != i {
				let j = (ea[i] as usize) >> 1;
				ea.swap(i, j);
				parity ^= 1;
			}
		}
		if parity != 0 {
			return -6;
		}
		0
	}

	fn random_reset(&mut self) {
		let mut rng = rand::thread_rng();
		let cperm = rng.gen_range(0..N_PERM) as u16;
		let mut parity = get_nparity(cperm as i32, 8);
		self.reset();
		self.set_cperm(cperm);
		self.set_twst(rng.gen_range(0..N_TWST) as u16);
		self.set_flip(rng.gen_range(0..N_FLIP) as u16);
		for i in 0..10 {
			let j = i + rng.gen_range(0..12 - i);
			if i != j {
				self.ea.swap(i, j);
				parity ^= 1;
			}
		}
		if parity != 0 {
			self.ea.swap(10, 11);
		}
	}

	fn from_facelet(&mut self, facelet: &String) -> i32 {
		if facelet.len() < 54 {
			return -1;
		}
		let fstr: &[u8] = facelet.as_bytes();
		let mut f: [u8; 54] = [0; 54];
		let colors: [u8; 6] = [fstr[4], fstr[13], fstr[22], fstr[31], fstr[40], fstr[49]];
		let mut count: i32 = 0;
		for i in 0..54 {
			if let Some(fidx) = colors.iter().position(|&s| s == fstr[i]) {
				f[i] = fidx as u8;
				count += 1 << (fidx * 4);
			} else {
				return -1;
			}
		}
		if count != 0x999999 {
			return -1;
		}
		self.reset();
		let mut ori: usize;
		for i in 0..8 {
			ori = 0;
			while ori < 3 {
				if f[CORNER_FACELET[i][ori] as usize] == 0 || f[CORNER_FACELET[i][ori] as usize] == 3 {
					break;
				}
				ori += 1
			}
			let col1 = f[CORNER_FACELET[i][(ori + 1) % 3] as usize];
			let col2 = f[CORNER_FACELET[i][(ori + 2) % 3] as usize];
			for j in 0..8 {
				if col1 == CORNER_FACELET[j][1] / 9 && col2 == CORNER_FACELET[j][2] / 9 {
					self.ca[i] = (ori as u8 % 3) << 3 | j as u8;
					break;
				}
			}
		}
		for i in 0..12 {
			for j in 0..12 {
				if f[EDGE_FACELET[i][0] as usize] == EDGE_FACELET[j][0] / 9
					&& f[EDGE_FACELET[i][1] as usize] == EDGE_FACELET[j][1] / 9
				{
					self.ea[i] = (j as u8) << 1;
					break;
				}
				if f[EDGE_FACELET[i][0] as usize] == EDGE_FACELET[j][1] / 9
					&& f[EDGE_FACELET[i][1] as usize] == EDGE_FACELET[j][0] / 9
				{
					self.ea[i] = ((j as u8) << 1) | 1;
					break;
				}
			}
		}
		0
	}

	fn to_facelet(&self) -> String {
		let colors: [char; 6] = ['U', 'R', 'F', 'D', 'L', 'B'];
		let mut f: [u8; 54] = [0; 54];
		for i in 0..54 {
			f[i] = (i as u8) / 9;
		}
		for c in 0..8 {
			let j = (self.ca[c] & 0x7) as usize;
			let ori = (self.ca[c] >> 3) as usize;
			for n in 0..3 {
				f[CORNER_FACELET[c][(n + ori) % 3] as usize] = CORNER_FACELET[j][n] / 9;
			}
		}
		for e in 0..12 {
			let j = (self.ea[e] >> 1) as usize;
			let ori = (self.ea[e] & 1) as usize;
			for n in 0..2 {
				f[EDGE_FACELET[e][(n + ori) % 2] as usize] = EDGE_FACELET[j][n] / 9;
			}
		}
		let mut buf = String::new();
		for i in 0..54 {
			buf.push(colors[f[i] as usize]);
		}
		buf
	}
}

lazy_static! {
	static ref global_sctx: Box<StaticContext> = StaticContext::box_new();
	static ref global_stbl: Box<StaticTables> = StaticTables::box_new(&global_sctx);
}


/// Solve a Rubik's cube represented in facelet
///
/// # Arguments
///
/// * `facelet` - the Rubik's cube to be solved, represented in facelet
/// * `maxl` - number of moves to solve the cube, included. 21 or 20 is recommended.
///
/// Facelet for the rubik's cube:
/// ```
///          +--------+
///          |U1 U2 U3|
///          |U4 U5 U6|
///          |U7 U8 U9|
/// +--------+--------+--------+--------+
/// |L1 L2 L3|F1 F2 F3|R1 R2 R3|B1 B2 B3|
/// |L4 L5 L6|F4 F5 F6|R4 R5 R6|B4 B5 B6|
/// |L7 L8 L9|F7 F8 F9|R7 R8 R9|B7 B8 B9|
/// +--------+--------+--------+--------+
///          |D1 D2 D3|
///          |D4 D5 D6|
///          |D7 D8 D9|
///          +--------+
/// ```
/// should be: U1U2...U9R1R2...R9F1..F9D1..D9L1..L9B1..B9
/// Example, facelet of solved cube is UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB
///
/// Return solution moves on success, return "Error " + error_code on failure
pub fn solve(facelet: &String, maxl: u8) -> String {
	let mut cc = Cubie::new();
	if cc.from_facelet(facelet) < 0 {
		return String::from("Error 1");
	}
	let verify = cc.verify();
	if verify < 0 {
		return String::from("Error ") + &(-verify).to_string();
	}
	let mut ctx = IdaContext::new();
	return ctx.solve_cubie(&global_sctx, &global_stbl, &cc, std::cmp::min(25, maxl) as i8)
}

/// Generate a random cube represented in facelet
pub fn random_cube() -> String {
	let mut cc = Cubie::new();
	cc.random_reset();
	cc.to_facelet()
}

/// Apply moves to a solved Rubik's cube
///
/// # Arguments
///
/// * `cube_moves` - should match ```([URFDLB][123'] ?)*```
///
/// Return ```facelet``` on success
pub fn from_moves(cube_moves: &String) -> Option<String> {
	apply_moves(&Cubie::new().to_facelet(), cube_moves)
}

/// Apply moves to a Rubik's cube represented by facelet
///
/// # Arguments
///
/// * `facelet` - the Rubik's cube to be moved, must be a solvable Rubik's cube
/// * `cube_moves` - should match ```([URFDLB][123'] ?)*```
///
/// Return ```facelet``` of the moved cube on success
pub fn apply_moves(facelet: &String, cube_moves: &String) -> Option<String> {
	let mut cc = Cubie::new();
	if cc.from_facelet(facelet) < 0 {
		return None;
	}
	let verify = cc.verify();
	if verify < 0 {
		return None;
	}
	let mut s = cube_moves.trim().chars().peekable();
	let mut axis = 0;
	let mut pow = 0;
	let mut cd = Cubie::new();
	while let Some(c) = s.next() {
		match c {
			'U' | 'R' | 'F' | 'D' | 'L' | 'B' => {
				if pow != 0 {
					Cubie::corn_mult(&cc, &global_sctx.movecube[axis * 3 + pow - 1], &mut cd);
					Cubie::edge_mult(&cc, &global_sctx.movecube[axis * 3 + pow - 1], &mut cd);
					cc = cd;
				}
				pow = 1;
				match c {
					'U' => { axis = 0; },
					'R' => { axis = 1; },
					'F' => { axis = 2; },
					'D' => { axis = 3; },
					'L' => { axis = 4; },
					'B' => { axis = 5; },
					_ => (),
				}
			},
			'\'' | '-' => pow = (4 - pow) % 4,
			'3' => pow = pow * 3 % 4,
			'2' => pow = pow * 2 % 4,
			'+' | '1' | ' ' | '\t' => (),
			_ => {
				return None;
			}
		}
	}

	if pow != 0 {
		Cubie::corn_mult(&cc, &global_sctx.movecube[axis * 3 + pow - 1], &mut cd);
		Cubie::edge_mult(&cc, &global_sctx.movecube[axis * 3 + pow - 1], &mut cd);
		cc = cd;
	}
	Some(cc.to_facelet())
}

/// Generate a random move sequence in specific number of moves
///
/// # Arguments
///
/// * `n_moves` - number of moves
///
/// Return moves, ensure no redaudant moves exists, e.g. "R R", "R L R", etc.
///
/// Call ```from_moves(cube_moves)``` to obtain the scrambled cube
pub fn random_moves(n_moves: u16) -> String {
	let mut rng = rand::thread_rng();
	let mut last_axis = 18;
	let mut scramble = String::new();
	let mut i = 0;
	while i < n_moves {
		let mv = rng.gen_range(0..18);
		let axis = mv / 3;
		if axis == last_axis || (axis % 3 == last_axis % 3 && axis > last_axis) {
			continue;
		}
		last_axis = axis;
		scramble.push_str(MOVE2STR[mv]);
		scramble.push_str(" ");
		i += 1
	}
	scramble
}

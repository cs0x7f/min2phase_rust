use min2phase::{random_cube, random_moves, from_moves, apply_moves, solve};
use std::time::Instant;

const MAX_SOL_LEN: u8 = 20;

#[test]
fn random_move_solve() {
	let now = Instant::now();
	let n_test = 1000;
	let n_moves = 100;
	for _ in 0..n_test {
		let scramble: String = random_moves(n_moves);
		assert!(scramble.len() == (n_moves as usize * 3));
		if let Some(cube) = from_moves(&scramble) {
			let solution = solve(&cube, MAX_SOL_LEN);
			if let Some(solved_cubie) = apply_moves(&cube, &solution) {
				assert!(solved_cubie == "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB");
			} else {
				assert!(false);
			}
		} else {
			println!("scramble={}", scramble);
			println!("move parse error!!");
			assert!(false);
		}
	}
	let elapsed = now.elapsed() / n_test;
	println!("Random move solve avg: {:.3?}", elapsed);
}

#[test]
fn random_state_solve() {
	let now = Instant::now();
	let n_test = 1000;
	for _ in 0..n_test {
		let cube = random_cube();
		let solution = solve(&cube, MAX_SOL_LEN);
		if let Some(solved_cubie) = apply_moves(&cube, &solution) {
			assert!(solved_cubie == "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB");
		} else {
			assert!(false);
		}
	}
	let elapsed = now.elapsed() / n_test;
	println!("Random state solve avg: {:.3?}", elapsed);
}
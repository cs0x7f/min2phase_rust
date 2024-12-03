use min2phase::{random_cube, random_moves, from_moves, solve};

fn main() {
	println!("Generate a random cube:"); 
	let cube: String = random_cube();
	println!("Generated: {}", cube);

	println!("Solve it in 21 moves..."); 
	let solution: String = solve(&cube, 21);
	println!("Result: {}", solution); 

	println!("Generate 30 random moves:"); 
	let moves: String = random_moves(30);
	println!("Generated: {}", moves);

	println!("Get scrambled cube:"); 
	let cube: Option<String> = from_moves(&moves);
	if let Some(cube) = cube {
		println!("Generated: {}", cube);
		println!("Solve it in 21 moves..."); 
		let solution = solve(&cube, 21);
		println!("Result: {}", solution); 
	} else {
		println!("Parse scramble error: {}", moves);
	}
/*
	let n_moves = 100;
	for _ in 0..100 {
		let scramble: String = random_moves(n_moves);
		if let Some(cube) = from_moves(&scramble) {
			println!("cubie={}", cube);
			println!("scramble={}", scramble);
			println!("solution={}", solve(&cube, 20));
		} else {
			println!("scramble={}", scramble);
			println!("move parse error!!");
		}
	}
*/
}

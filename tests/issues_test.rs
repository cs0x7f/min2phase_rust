use min2phase::{apply_moves, solve};

const MAX_SOL_LEN: u8 = 20;

#[test]
fn test_issue1_solve() {
    let cube = "UDUDUDUDURLRLRLRLRFBFBFBFBFDUDUDUDUDLRLRLRLRLBFBFBFBFB".to_string();
    let solution = solve(&cube, MAX_SOL_LEN);
    let solved_cubie = apply_moves(&cube, &solution);
    assert!(
        solved_cubie == Some("UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB".to_string()),
        "unexpected result: solution={solution} solved_cubie={solved_cubie:?}"
    );
}

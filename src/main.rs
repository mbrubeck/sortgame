
mod logic;

use logic::*;

fn main() {
    const SLICE_COUNT : i32 = MAX_SLICES as i32;
    const COLOR_COUNT : i32 = 8;
    let mut search_dir : i32 = -1;
    let mut direction : i32;

    const SS_COUNT : usize = 1024*1024;
    let mut ss = vec![SliceStack::new(); SS_COUNT];


    for s in ss.iter_mut() {
        ss_init_unsafe(s, SLICE_COUNT, COLOR_COUNT);
    }

    let mut iters : usize = 0;
    for s in &mut ss[..] {
        while !ss_iscomplete_unsafe(s) {
            iters += 1;

            direction = 0;
            let mut index = ss_find_single_joining_move2(s, &mut direction);
            if index == -1 { index = ss_find_first_double_move(s, search_dir, &mut direction); }
            //assert!(index != -1);
            index += direction;
            //ss_flip_rsslice(s, index, direction);
            ss_flip(s, index, direction);
            //search_dir = !search_dir;
        }
    }

    println!("total iterations: {}", iters);
}

/*
    Copyright (C) 2016  Erik Beran

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

pub const MAX_SLICES : usize = 32;

#[derive(Copy,Clone)]
pub struct SliceStack {
    count : i32,
    color_count : i32,
    slice_type : [u8; MAX_SLICES]
}

impl SliceStack {
    pub fn new() -> SliceStack { SliceStack { count:0, color_count:0, slice_type:[0;MAX_SLICES] } }
}

/* START PRNG HELPERS */
pub struct PrngCtxGaloisLsfw {
    value : u32
}
pub fn prng_seed_galois_lsfw(ctx : &mut PrngCtxGaloisLsfw, s : u32) {
    ctx.value = s;
}

pub fn prng_galois_lsfw(ctx : &mut PrngCtxGaloisLsfw) -> u32 {
    let shifted = ctx.value >> 1;
    let negated = (-((ctx.value & 1) as i32)) as u32;
    let anded = negated & 0x80200003;
    ctx.value = shifted ^ anded;
    return ctx.value;
}

pub fn prng_galois_lsfw_int_minmax(ctx : &mut PrngCtxGaloisLsfw, min : i32,
                                   max : i32) -> i32 {
    use std::u32;
    const INV_INT_MAX : f32 = 1.0 / ((u32::MAX) as f32);
    let f : f32 = (prng_galois_lsfw(ctx) as f32) * INV_INT_MAX;
    let frange : f32 = (max-min) as f32;
    return ((f * frange + 0.5) as i32) + min;
}

// global; not thread-safe etc
static mut l_ctx : PrngCtxGaloisLsfw = PrngCtxGaloisLsfw {value:34};
fn fake_rand(min : i32, max : i32) -> i32 {
    unsafe {
        return prng_galois_lsfw_int_minmax(&mut l_ctx, min, max);
    }
}
/* END PRNG HELPERS */

pub fn ss_init(ss : &mut SliceStack, count : i32, color_count : i32) {
    ss.count = count;
    ss.color_count = color_count;
    for t in ss.slice_type[0..(count as usize)].iter_mut() {
        *t = fake_rand(0,color_count) as u8;
    }
}

fn ss_swapslices(ss : &mut SliceStack, i0 : i32, i1 : i32) {
    ss.slice_type.swap(i0 as usize, i1 as usize);
}

pub fn ss_flip(ss : &mut SliceStack, index : i32, direction : i32) {
    if direction > 0 {
        let slice : &mut [u8] = &mut ss.slice_type[(index as usize)..(ss.count as usize)];
        slice.reverse();
    } else {
        let slice : &mut [u8] = &mut ss.slice_type[0..((index+1) as usize)];
        slice.reverse();
    }
}

// Fragmentation value of stack; minimum is different slice types
// i.e. if fragmentation - slice_types == 0, it's completed
pub fn ss_fragmentation(ss : &SliceStack) -> i32 {
    let mut last_type : u8 = 0xFF;
    let mut frag : i32 = 0;
    for t in ss.slice_type[0..(ss.count as usize)].iter() {
        if last_type != *t {
            frag += 1;
        }
        last_type = *t;
    }
    return frag;
}

pub fn ss_fragmentation2(ss : &SliceStack) -> i32 {
    let mut frag : i32 = 1;
    for i in 1..ss.count as usize {
        if ss.slice_type[i-1] != ss.slice_type[i] {
            frag += 1;
        }
    }
    return frag;
}

// Returns bool, true on stack/level is complete
pub fn ss_iscomplete(ss : &SliceStack) -> bool {
    let mut used_type_flags : u32 = 0;
    let mut last_type : u8 = 0xFF;
    for t in ss.slice_type[0..(ss.count as usize)].iter() {
        let type_flag : u32 = 1 << *t;
        if (used_type_flags & type_flag) != 0
            && last_type != *t {
                return false;
        }
        used_type_flags |= type_flag;
        last_type = *t;
    }
    return true;
}

// This only works if color_count is correct (i.e. if you tally up the
// different colors in the color_type array, it will match color_count)
pub fn ss_iscomplete2(ss : &SliceStack) -> bool {
    return (ss_fragmentation2(ss) - ss.color_count) <= 0;
}

// Success: Valid index and direction
// Failure: -1 (Cannot find valid move)
pub fn ss_find_single_joining_move(ss : &SliceStack, dir : &mut i32)
    -> i32 {
    let mut found_diff_type : bool;
    // left edge
    {
        found_diff_type = false;
        let outer_type = ss.slice_type[0];
        for (i,t) in ss.slice_type[0..(ss.count as usize)].iter().enumerate() {
            if *t == outer_type {
                if found_diff_type {
                    *dir = -1;
                    return i as i32;
                }
            } else {
                found_diff_type = true;
            }
        }
    }

    // We haven't found a matching type, try from the other end and direction
    // right edge
    {
        found_diff_type = false;
        let outer_type = ss.slice_type[(ss.count - 1) as usize];
        for (i,t) in ss.slice_type[1..(ss.count as usize)].iter().enumerate().rev() {
            if *t == outer_type {
                if found_diff_type {
                    *dir = 1;
                    return (i as i32) + 1;
                }
            } else {
                found_diff_type = true;
            }
        }
    }
    return -1;
}

// Success: Valid index and direction
// Failure: -1 (Cannot find valid move)
pub fn ss_find_single_joining_move2(ss : &SliceStack, dir : &mut i32)
    -> i32 {
    let mut last_index : i32 = 0;
    // left edge
    {
        for i in 1..ss.count {
            if (ss.slice_type[i as usize] == ss.slice_type[0])
                && (ss.slice_type[i as usize] != ss.slice_type[last_index as usize]) {
                *dir = -1;
                return i;
            }
            last_index = i;
        }
    }

    // We haven't found a matching type, try from the other end and direction
    // right edge
    {
        last_index = ss.count-1;
        for i in (0..(ss.count-1)).rev() {
            if (ss.slice_type[i as usize] == ss.slice_type[(ss.count-1) as usize])
                && (ss.slice_type[i as usize] != ss.slice_type[last_index as usize]) {
                *dir = 1;
                return i;
            }
            last_index = i;
        }
    }
    return -1;
}

// Success: Valid index and direction
// Failure: -1 (Cannot find valid move, should not happen)
pub fn ss_find_first_double_move(ss : &SliceStack, search_dir : i32, dir : &mut i32)
    -> i32 {
    let mut c_index : [u8; MAX_SLICES] = [0xFF; MAX_SLICES];
    let mut last_type : u8 = 0xFF;

    if search_dir <= 0 {
        for (i,t) in ss.slice_type[0..ss.count as usize].iter().enumerate() {
            // Color hasn't been recorded yet
            if c_index[*t as usize] == 0xFF {
                c_index[*t as usize] = i as u8;
                last_type = *t
            // non-contiguous color found
            // last_type will have been initialized by first iteration; don't check
            // for 0xff. If last_type == current_type we don't need to update it
            } else if *t != last_type {
                *dir = 1;
                return (i as i32)-1;
            }
        }
    } else {
        for (i,t) in ss.slice_type[1..ss.count as usize].iter().enumerate().rev() {
            // Color hasn't been recorded yet
            if c_index[*t as usize] == 0xFF {
                c_index[*t as usize] = i as u8;
                last_type = *t;
            // non-contiguous color found
            // last_type will have been initialized by first iteration; don't check
            // for 0xff. If last_type == current_type we don't need to update it
            } else if *t != last_type {
                *dir = -1;
                return (i as i32)+2;
            }
        }
    }

    // This should only happen when level is complete; As long as there is
    // fragmentation, there will be a way to move one section to the edge
    // to start the double move.
    unreachable!();
    return -1;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prng_galois_lsfw_test() {
        let mut ctx : PrngCtxGaloisLsfw = PrngCtxGaloisLsfw { value:0 };
        // check seeding values
        {
            let expected_values : [u32 ; 10] = [2149580803, 1, 2149580802, 2,
                2149580801, 3, 2149580800, 4, 2149580807, 5];
            for u in 0..10 as u32 {
                prng_seed_galois_lsfw(&mut ctx, u+1);
                assert_eq!(prng_galois_lsfw(&mut ctx), expected_values[u as usize]);
            }
        }
        // check consecutive calls after initial seeding
        {
            let expected_values : [u32 ; 10] = [2149580803, 3224371202,
                1612185601, 2955673603, 3627417602, 1813708801, 3056435203,
                3677798402, 1838899201, 3069030403];
            prng_seed_galois_lsfw(&mut ctx, 1);
            for u in 0..10 as usize {
                assert_eq!(prng_galois_lsfw(&mut ctx), expected_values[u]);
            }
        }
    }

    #[test]
    fn prng_galois_lsfw_int_minmax_test() {
         let mut ctx : PrngCtxGaloisLsfw = PrngCtxGaloisLsfw { value:0x10293847 };
        // check seeding values
        {
            let expected_values : [i32 ; 10] = [ 532, 266, 133, 67, 33, 17,
                508, 754, 377, 689 ];
            for u in 0..10 as usize {
                assert_eq!(prng_galois_lsfw_int_minmax(&mut ctx, 0, 1000),
                    expected_values[u]);
            }
        }
    }

    #[test]
    fn basic_init_test() {
        let mut ss : SliceStack = SliceStack{
            count : 0,
            color_count : 0,
            slice_type : [0; MAX_SLICES]
        };
        ss_init(&mut ss, 4, 2);
        assert_eq!(4, ss.count);
        assert_eq!(2, ss.color_count);
        println!("someothing!");
    }

    #[test]
    fn ss_is_complete_test() {
        let mut ss : SliceStack = SliceStack {
            count : 4,
            color_count : 3,
            slice_type : [0; MAX_SLICES]
        };
        ss.slice_type[0] = 0;
        ss.slice_type[1] = 2;
        ss.slice_type[2] = 1;
        ss.slice_type[3] = 1;
        assert!(ss_iscomplete(&ss));
        assert!(ss_iscomplete2(&ss));
        ss.slice_type[0] = 1;
        ss.slice_type[1] = 0;
        ss.slice_type[2] = 2;
        ss.slice_type[3] = 1;
        assert!(!ss_iscomplete(&ss));
        assert!(!ss_iscomplete2(&ss));
    }

    #[test]
    fn ss_fragmentation_test() {
        let mut ss : SliceStack = SliceStack {
            count : 4,
            color_count : 3,
            slice_type : [0; MAX_SLICES]
        };
        ss.slice_type[0] = 0;
        ss.slice_type[1] = 2;
        ss.slice_type[2] = 1;
        ss.slice_type[3] = 1;
        assert_eq!(3, ss_fragmentation(&ss));
        assert_eq!(3, ss_fragmentation2(&ss));
    }

    #[test]
    fn ss_fragmentation_equiv_test() {
        const SLICE_COUNT : i32 = MAX_SLICES as i32;
        const COLOR_COUNT : i32 = 8;

        const SS_COUNT : usize = 1;//024*1024;
        let mut ss = vec![SliceStack::new(); SS_COUNT];

        for s in ss.iter_mut() {
            ss_init(s, SLICE_COUNT, COLOR_COUNT);
            let f1 = ss_fragmentation(s);
            let f2 = ss_fragmentation2(s);
            assert!(f1 <= s.count);
            assert!(f2 <= s.count);
            assert_eq!(f1, f2);
            assert_eq!(ss_iscomplete(s), ss_iscomplete2(s));
        }
    }

    #[test]
    fn ss_find_single_joining_move_test() {
        // TODO
    }

   #[test]
    fn ss_find_first_double_move_test() {
        // TODO
    }

    #[test]
    fn lots_of_solutions() {
        const SLICE_COUNT : i32 = MAX_SLICES as i32;
        const COLOR_COUNT : i32 = 8;
        let mut search_dir : i32 = -1;
        let mut direction : i32;

        const SS_COUNT : usize = 1024*1024;
        let mut ss = vec![SliceStack::new(); SS_COUNT];

        for s in ss.iter_mut() {
            ss_init(s, SLICE_COUNT, COLOR_COUNT);
        }

        for s in &mut ss[..] {
            while !ss_iscomplete(s) {
            //loop {
                //if ss_iscomplete2(s) { break; }
                //let mut index = ss_find_single_joining_move(s, &mut direction).unwrap_or_else(
                //    || ss_find_first_double_move(s, search_dir, &mut direction).unwrap_or(
                //        SLICE_COUNT));
                direction = 0;
                let mut index = ss_find_single_joining_move(s, &mut direction);
                if index == -1 { index = ss_find_first_double_move(s, search_dir, &mut direction); }
                assert!(index != -1);
                index += direction;
                ss_flip(s, index, direction);
                //search_dir = !search_dir;
            }
        }
    }

}

/* vim: set ts=4 sts=4 sw=4 et : */

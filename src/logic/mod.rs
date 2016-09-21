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

pub const MAX_SLICES : usize = 16;

use prng::*;

use std::ptr::*;

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
    let mut i : usize = 0;
    for i in 0..ss.count as usize {
        ss.slice_type[i] = fake_rand(0, color_count-1) as u8;
    }
}

pub fn ss_init_unsafe(ss : &mut SliceStack, count : i32, color_count : i32) {
    ss.count = count;
    ss.color_count = color_count;
    let mut i : usize = 0;
    while i < ss.count as usize {
        unsafe { *ss.slice_type.get_unchecked_mut(i) = fake_rand(0,color_count-1) as u8 };
        i += 1;
    }
}

fn ss_swapslices(ss : &mut SliceStack, i0 : i32, i1 : i32) {
    ss.slice_type.swap(i0 as usize, i1 as usize);
}

fn ss_swapslices_unsafe(ss : &mut SliceStack, i0 : i32, i1 : i32) {
    // This could also be done with raw pointers, but this shoudl be
    // equivalent in code generation
    unsafe {
        swap(ss.slice_type.get_unchecked_mut(i0 as usize),
            ss.slice_type.get_unchecked_mut(i1 as usize));
    }
}

// Manually flipping
pub fn ss_flip(ss : &mut SliceStack, index : i32, direction : i32) {
    if direction > 0 {
        let end = ss.count - 1;
        let count = (ss.count - index) / 2;
        for i in 0..count {
            let mirror = end - i;
            let pos = index + i;
            //ss_swapslices(ss, pos, mirror);
            ss_swapslices_unsafe(ss, pos, mirror);
        }
    } else {
        let count = (index + 1) / 2;
        for i in 0..count {
            let mirror = i;
            let pos = index - i;
            //ss_swapslices(ss, pos, mirror);
            ss_swapslices_unsafe(ss, pos, mirror);
        }
    }
}

// Flipping using Rust slices and API's
pub fn ss_flip_rsslice(ss : &mut SliceStack, index : i32, direction : i32) {
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
    let mut i : usize = 0;
    while i < (ss.count as usize) {
        if last_type != ss.slice_type[i] {
            frag += 1;
        }
        last_type = ss.slice_type[i];
    }
    return frag;
}

pub fn ss_fragmentation_unsafe(ss : &SliceStack) -> i32 {
    let mut last_type : u8 = 0xFF;
    let mut frag : i32 = 0;
    let mut i : usize = 0;
    while i < (ss.count as usize) {
        let t = unsafe { *ss.slice_type.get_unchecked(i) };
        if last_type != t {
            frag += 1;
        }
        last_type = t;
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
    let mut type_flag : u32;
    for t in ss.slice_type[0..ss.count as usize].iter() {
        type_flag = 1 << *t;
        if ((used_type_flags & type_flag) != 0) && (last_type != *t) {
            return false;
        }
        used_type_flags |= type_flag;
        last_type = *t;
    }
    return true;
}

pub fn ss_iscomplete_unsafe(ss : &SliceStack) -> bool {
    let mut used_type_flags : u32 = 0;
    let mut last_type : u32 = 0x000000FF;
    let mut i : i32 = 0;
    let mut type_flag : u32;
    while i < ss.count {
        let t = unsafe { *ss.slice_type.get_unchecked(i as usize) as u32 };
        type_flag = 1 << t;
        if ((used_type_flags & type_flag) != 0) && (last_type != t) {
            return false;
        }
        used_type_flags |= type_flag;
        last_type = t;
        i += 1;
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

pub fn ss_find_single_joining_move_unsafe(ss : &SliceStack, dir : &mut i32)
    -> i32 {
    let mut found_diff_type : bool;
    let mut i : i32;
    // left edge
    unsafe {
        found_diff_type = false;
        let outer_type = *ss.slice_type.get_unchecked(0);
        i = 0;
        while i < ss.count {
            let t = *ss.slice_type.get_unchecked(i as usize);
            if t == outer_type {
                if found_diff_type {
                    *dir = -1;
                    return i;
                }
            } else {
                found_diff_type = true;
            }
            i += 1;
        }
    }

    // We haven't found a matching type, try from the other end and direction
    // right edge
    unsafe {
        found_diff_type = false;
        let outer_type = *ss.slice_type.get_unchecked((ss.count - 1) as usize);
        i = ss.count-1;
        while i > 0 {
            let t = *ss.slice_type.get_unchecked(i as usize);
            if t == outer_type {
                if found_diff_type {
                    *dir = 1;
                    return i;
                }
            } else {
                found_diff_type = true;
            }
            i -= 1;
        }
    }
    return -1;
}

// Success: Valid index and direction
// Failure: -1 (Cannot find valid move)
pub fn ss_find_single_joining_move2(ss : &SliceStack, dir : &mut i32)
    -> i32 {
    let mut last_index : isize = 0;
    let mut i : isize;
    // left edge
    unsafe {
        i = 1;
        let array = ss.slice_type.as_ptr();
        let left_type = *array.offset(0);
        while i < ss.count as isize {
            if (*array.offset(i) == left_type)
                && (*array.offset(i) != *array.offset(last_index)) {
                *dir = -1;
                return i as i32;
            }
            last_index = i;
            i += 1;
        }
    }

    // We haven't found a matching type, try from the other end and direction
    // right edge
    unsafe {
        let array = ss.slice_type.as_ptr();
        last_index = (ss.count-1) as isize;
        let right_type = *array.offset(ss.count as isize - 1);
        i = (ss.count - 1) as isize;
        while i >= 0 {
            if (*array.offset(i) == right_type)
                && (*array.offset(i) != *array.offset(last_index)) {
                *dir = 1;
                return i as i32;
            }
            last_index = i;
            i -= 1;
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
    let mut i : i32;

    if search_dir <= 0 {
        i = 0;
        while i < ss.count {
            // Color hasn't been recorded yet
            let t = ss.slice_type[i as usize];
            let ci : &mut u8 = &mut c_index[t as usize];
            if *ci == 0xFF {
                *ci = i as u8;
                last_type = t
            // non-contiguous color found
            // last_type will have been initialized by first iteration; don't check
            // for 0xff. If last_type == current_type we don't need to update it
            } else if t != last_type {
                *dir = 1;
                return i - 1;
            }
            i += 1;
        }
    } else {
        i = ss.count - 1;
        while i > 0 {
            // Color hasn't been recorded yet
            let t = ss.slice_type[i as usize];
            let ci : &mut u8 = &mut c_index[t as usize];
            if *ci == 0xFF {
                *ci = i as u8;
                last_type = t;
            // non-contiguous color found
            // last_type will have been initialized by first iteration; don't check
            // for 0xff. If last_type == current_type we don't need to update it
            } else if t != last_type {
                *dir = -1;
                return i + 1;
            }
            i -= 1;
        }
    }

    // This should only happen when level is complete; As long as there is
    // fragmentation, there will be a way to move one section to the edge
    // to start the double move.
    unreachable!();
    return -1;
}

pub fn ss_find_first_double_move_unsafe(ss : &SliceStack, search_dir : i32, dir : &mut i32)
    -> i32 {
    let mut c_index : [u8; MAX_SLICES] = [0xFF; MAX_SLICES];
    let mut last_type : u8 = 0xFF;
    let mut i : i32;

    if search_dir <= 0 {
        i = 0;
        while i < ss.count {
            let t = unsafe { *ss.slice_type.get_unchecked(i as usize) };
            let ci = unsafe { c_index.get_unchecked_mut(t as usize) };
            // Color hasn't been recorded yet
            if *ci == 0xFF {
                *ci = i as u8;
                last_type = t
            // non-contiguous color found
            // last_type will have been initialized by first iteration; don't check
            // for 0xff. If last_type == current_type we don't need to update it
            } else if t != last_type {
                *dir = 1;
                return i - 1;
            }
            i += 1;
        }
    } else {
        i = ss.count - 1;
        while i > 0 {
            let t = unsafe { *ss.slice_type.get_unchecked(i as usize) };
            let ci = unsafe { c_index.get_unchecked_mut(t as usize) };
            // Color hasn't been recorded yet
            if *ci == 0xFF {
                *ci = i as u8;
                last_type = t;
            // non-contiguous color found
            // last_type will have been initialized by first iteration; don't check
            // for 0xff. If last_type == current_type we don't need to update it
            } else if t != last_type {
                *dir = -1;
                return i + 1;
            }
            i -= 1;
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
    }

    #[test]
    fn ss_iscomplete_test() {
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
                let mut index = ss_find_single_joining_move_unsafe(s, &mut direction);
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

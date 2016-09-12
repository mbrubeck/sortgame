
pub const MAX_SLICES : usize = 32;

#[derive(Copy,Clone)]
pub struct SliceStack {
    count : i32,
    color_count : i32,
    slice_type : [u8; MAX_SLICES]
}

/*
impl Default for SliceStack {
    fn default() -> SliceStack {
        SliceStack { count:0, color_count:0, slice_type:[0; MAX_SLICES] }
    }
}
*/

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
    for i in 0..count as usize {
        ss.slice_type[i] = fake_rand(0,color_count) as u8;
    }
}

fn ss_swapslices(ss : &mut SliceStack, i0 : usize, i1 : usize) {
    let stype : u8;
    stype = ss.slice_type[i0];
    ss.slice_type[i0] = ss.slice_type[i1];
    ss.slice_type[i1] = stype;
}

pub fn ss_flip(ss : &mut SliceStack, index : usize, direction : i32) {
    if direction > 0 {
        let end = ss.count - 1;
        let count = (ss.count - (index as i32)) / 2;
        for i in 0..count as usize {
            let mirror = (end as usize) - i;
            let pos = index + i;
            ss_swapslices(ss, pos, mirror);
        }
    } else {
        let count = (index + 1) / 2;
        for i in 0..count as usize {
            let mirror = i;
            let pos = index - i;
            ss_swapslices(ss, pos, mirror);
        }
    }
}

pub fn ss_fragmentation(ss : &SliceStack) -> i32 {
    let mut last_type : u8 = 0xFF;
    let mut frag : i32 = 0;
    for i in 0..ss.count as usize {
        if last_type != ss.slice_type[i] {
            frag += 1;
        }
        last_type = ss.slice_type[i];
    }
    return frag;
}

pub fn ss_fragmentation2(ss : &SliceStack) -> i32 {
    let mut last_index : usize = 0;
    let mut frag : i32 = 1;
    for i in 1..ss.count as usize {
        if ss.slice_type[last_index] != ss.slice_type[i] {
            frag += 1;
        }
        last_index = i;
    }
    return frag;
}

pub fn ss_iscomplete(ss : &SliceStack) -> bool {
    let mut used_type_flags : u32 = 0;
    let mut last_type : u8 = 0xFF;
    for i in 0..ss.count as usize {
        let type_flag : u32 = 1 << ss.slice_type[i];
        if (used_type_flags & type_flag) != 0
            && last_type != ss.slice_type[i] {
                return false;
        }
        used_type_flags |= type_flag;
        last_type = ss.slice_type[i];
    }
    return true;
}

pub fn ss_iscomplete2(ss : &SliceStack) -> bool {
    return (ss_fragmentation2(ss) - ss.color_count) <= 0
}

pub fn ss_find_single_joining_move(ss : &SliceStack, direction : &mut i32) -> i32 {
    let mut found_diff_type : bool;
    // left edge
    {
        found_diff_type = false;
        let outer_type = ss.slice_type[0];
        for i in 0..ss.count as usize {
            if ss.slice_type[i] == outer_type {
                if found_diff_type {
                    *direction = 0;
                    return i as i32;
                }
            } else {
                found_diff_type = true;
            }
        }
    }

    // right edge
    {
        found_diff_type = false;
        let outer_type = ss.slice_type[(ss.count - 1) as usize];
        for i in (0..(ss.count-1) as usize).rev() {
            if ss.slice_type[i] == outer_type {
                if found_diff_type {
                    *direction = 1;
                    return i as i32;
                }
            } else {
                found_diff_type = true;
            }
        }
    }
    return -1;
}

pub fn ss_find_single_joining_move2(ss : &SliceStack, direction : &mut i32) -> i32 {
    let mut last_index : usize = 0;
    // left edge
    {
        for i in 1..ss.count as usize {
            if (ss.slice_type[i] == ss.slice_type[0])
                && (ss.slice_type[i] != ss.slice_type[last_index]) {
                *direction = 0;
                return i as i32;
            }
            last_index = i;
        }
    }

    // right edge
    {
        last_index = (ss.count-1) as usize;
        for i in (0..(ss.count-2) as usize).rev() {
            if (ss.slice_type[i] == ss.slice_type[(ss.count-1) as usize])
                && (ss.slice_type[i] != ss.slice_type[last_index]) {
                *direction = 1;
                return i as i32;
            }
            last_index = i;
        }
    }
    return -1;
}

pub fn ss_find_first_double_move(ss : &SliceStack, search_dir : i32,
                                 direction : &mut i32) -> i32 {
    let mut c_index : [u8; MAX_SLICES] = [0xFF; MAX_SLICES];
    let mut last_type : u8 = 0xFF;

    if search_dir <= 0 {
        for i in 0..ss.count as usize {
            if c_index[ss.slice_type[i] as usize] == 0xFF {
                c_index[ss.slice_type[i] as usize] = i as u8;
                last_type = ss.slice_type[i];
            } else if ss.slice_type[i] != last_type {
                *direction = 1;
                return (i-1) as i32;
            }
        }
    } else {
        for i in (1..(ss.count-1) as usize).rev() {
            if c_index[ss.slice_type[i] as usize] == 0xFF {
                c_index[ss.slice_type[i] as usize] = i as u8;
                last_type = ss.slice_type[i];
            } else if ss.slice_type[i] != last_type {
                *direction = 0;
                return (i+1) as i32;
            }
        }
    }

    return -1;
}

#[cfg(test)]
mod test {
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
    fn ss_find_single_joining_move_test() {
        // TODO
    }

   #[test]
    fn ss_find_first_double_move_test() {
        // TODO
    }

    #[test]
    fn lots_of_solutions() {
        const SLICE_COUNT : i32 = 16;
        const COLOR_COUNT : i32 = 8;
        let mut search_dir : i32 = 0;
        let mut direction : i32 = 0;


        const SS_COUNT : usize = 1024;
        let mut ss : [SliceStack; SS_COUNT] = [SliceStack {
            count : 0,
            color_count : 0,
            slice_type : [0; MAX_SLICES]}; SS_COUNT];

        //for s in ss.iter_mut() {
        for s in &mut ss[..] {
            ss_init(s, SLICE_COUNT, COLOR_COUNT);
        }

        for s in &mut ss[..] {
            loop {
                if ss_iscomplete(s) { break; }
                let mut index = ss_find_single_joining_move(s, &mut direction);
                if index < 0 {
                    index = ss_find_first_double_move(s, search_dir, &mut direction);
                    search_dir = !search_dir;
                }

                assert!(index != -1);
                if direction == 0 { direction = -1; }
                ss_flip(s, (index + direction) as usize, direction);
            }
        }
    }

}

/* vim: set ts=4 sts=4 sw=4 et : */

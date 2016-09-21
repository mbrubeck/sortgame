
pub struct PrngCtxGaloisLsfw {
    pub value : u32
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


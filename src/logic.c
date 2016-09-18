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

#include "logic.h"

#include <stdint.h> // int types, limits, macros
#include <string.h> // memset
#include <stdio.h>

#define RUN_TEST 1


//#define ARRAY_LEN(a) (sizeof(a) / sizeof(a[0]))
#define ARRAY_LEN(a) ((unsigned char*)((&a)[1]) - (unsigned char*)a)

/* start PRNG helpers */
static unsigned int l_galois_lfsr = 34;
void
seed_galois_lfsr(unsigned int s) { l_galois_lfsr = s; }
unsigned int
galois_lfsr() {
    unsigned int *const r = &l_galois_lfsr;
    *r = (*r >> 1) ^ (-(*r & 1) & 0x80200003);
    return *r;
}

// splitmix64
static uint64_t l_sm64;
void
seed_splitmix64(uint64_t s) { l_sm64 = s; }
uint64_t
splitmix64() {
    uint64_t z = (l_sm64 += UINT64_C(0x9E3779B97F4A7C15));
    z = (z ^ (z >> 30)) * UINT64_C(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)) * UINT64_C(0x94D049BB133111EB);
    return z ^ (z >> 31);
}

static int
fakeRand(int rmin, int rmax) {
    static float const inv_max = 1.0f / (float)UINT32_MAX;
    float const f = galois_lfsr() * inv_max;
    float const frange = (float)(rmax-rmin);
    return (f * frange + 0.5f) + rmin;
}
/* end PRNG helpers */

static void
SliceStack_SwapSlices(struct SliceStack *ss, int i0, int i1) {
    void* obj;
    int type;
    type = ss->slice_type[i0];
    obj = ss->slice_object[i0];
    ss->slice_type[i0] = ss->slice_type[i1];
    ss->slice_object[i0] = ss->slice_object[i1];
    ss->slice_type[i1] = type;
    ss->slice_object[i1] = obj;
}

//
void
SliceStack_Create(struct SliceStack *ss, int count, int color_count) {
    int i;
    //memset(ss->slice_type, 0, sizeof(SliceStack::slice_type));
    //memset(ss->slice_object, 0, sizeof(SliceStack::slice_object));
    ss->count = count;
    ss->color_count = color_count;
    for (i = 0; i < count; ++i) {
        ss->slice_type[i]  = fakeRand(0, color_count-1);
        //ss->slice_object[i] = 0;
    }
}

void
SliceStack_Randomize(struct SliceStack *ss) {
    unsigned char indices[MAX_SLICES];
    int i;

    for (i = 0; i < ss->count; i++) {
        //indices[i] = i % ss->count
    }
}

//
void
SliceStack_Flip(struct SliceStack *ss, int index, int direction) {
    int end;
    int count;
    int i;

    if (direction > 0) {
        end = ss->count - 1;
        count = (ss->count - index) / 2;
        for (i = 0; i < count; ++i) {
            int const mirror = end - i;
            int const pos = index + i;
            SliceStack_SwapSlices(ss, pos, mirror);
        }
    } else {
        //end = 0;
        count = (index + 1) / 2;
        for (i = 0; i < count; ++i) {
            int const mirror = i;
            int const pos = index - i;
            SliceStack_SwapSlices(ss, pos, mirror);
        }
    }
}

//
int
SliceStack_IsComplete(struct SliceStack *ss) {
    unsigned int used_type_flags = 0;
    unsigned int last_type = 0xFFFFFFFF;
    int i;

    for (i = 0; i < ss->count; ++i) {
        unsigned int const type_flag = 1 << ss->slice_type[i];
        if ((used_type_flags & type_flag)
            && last_type != ss->slice_type[i]) {
            return 0;
        }
        used_type_flags |= type_flag;
        last_type = ss->slice_type[i];
    }

    return 1;
}

// Fragmentation value of stack; minimum is different slice types
// i.e. if fragmentation - slice_types == 0, it's completed
int
SliceStack_Fragmentation(struct SliceStack *ss)
{
    int last_type = -1;
    int frag = 0;
    int i;

    for (i = 0; i < ss->count; ++i) {
        if (last_type != ss->slice_type[i]) {
            ++frag;
        }
        last_type = ss->slice_type[i];
    }
    return frag;
}

int
SliceStack_Fragmentation2(struct SliceStack *ss)
{
    int frag = 1;
    int i_last = 0;
    int i;

    for (i = 1; i < ss->count; ++i) {
        if (ss->slice_type[i_last] != ss->slice_type[i]) {
            ++frag;
        }
        i_last = i;
    }
    return frag;
}

//
int
SliceStack_IsComplete2(struct SliceStack *ss) {
    // Fragmentation *may* be less than color count when color count doesn't
    // represent the number of colors present in the actual stack (as apposed
    // to what it was initialized with).
    return (SliceStack_Fragmentation(ss) - ss->color_count) <= 0;
}

// Returns valid index on success
// Returns -1 on failure to find edge move
int
SliceStack_FindSingleJoiningMove(struct SliceStack *ss, int *direction) {
    int outer_type = ss->slice_type[0];
    int found_diff_type = 0;
    int i;
    //*direction = 0;

    for (i = 0; i < ss->count; ++i) {
        if (ss->slice_type[i] == outer_type) {
            if (found_diff_type) {
                *direction = 0;
                return i;
            }
        } else {
            found_diff_type = 1;
        }
    }

    // if we haven't found a matching type, try the other end and direction
    //*direction = 1;
    outer_type = ss->slice_type[ss->count - 1];
    found_diff_type = 0;
    for (i = ss->count-1; i > 0; --i) {
        if (ss->slice_type[i] == outer_type) {
            if (found_diff_type) {
                *direction = 1;
                return i;
            }
        } else {
            found_diff_type = 1;
        }
    }

    return -1;
}

int
SliceStack_FindSingleJoiningMove2(struct SliceStack *ss, int *direction) {
    int i;
    int i_last = 0;

    for (i = 1; i < ss->count; ++i) {
        if (ss->slice_type[i] == ss->slice_type[0]
            && ss->slice_type[i] != ss->slice_type[i_last]) {
            *direction = 0;
            return i;
        }
        i_last = i;
    }

    // if we haven't found a matching type, try the other end and direction
    i_last = ss->count-1;
    for (i = ss->count-2; i > 0; --i) {
        if (ss->slice_type[i] == ss->slice_type[ss->count-1]
            && ss->slice_type[i] != ss->slice_type[i_last]) {
            *direction = 1;
            return i;
        }
        i_last = i;
    }

    return -1;
}

//
int
SliceStack_FindFirstDoubleMove(struct SliceStack *ss, int search_direction,
                               int *direction) {
    unsigned char c_index[MAX_SLICES];
    int i;
    int last_type;

    memset(c_index, 0xff, sizeof(c_index));
    last_type = 0xff;

    if (search_direction <= 0) {
        for (i = 0; i < ss->count; ++i) {
            // color hasn't been recorded yet
            if (c_index[ss->slice_type[i]] == 0xff) {
                c_index[ss->slice_type[i]] = i;
                last_type = ss->slice_type[i];
                // non-contiguous color found
                // last_type will have been initialized by first iteration; don't check
                // for 0xff. If last_type == current_type we don't need to update it
            } else if (ss->slice_type[i] != last_type) {
                *direction = 1;
                return i-1;
            }
        }
    } else {
        for (i = ss->count-1; i > 0; --i) {
            // color hasn't been recorded yet
            if (c_index[ss->slice_type[i]] == 0xff) {
                c_index[ss->slice_type[i]] = i;
                last_type = ss->slice_type[i];
                // non-contiguous color found
                // last_type will have been initialized by first iteration; don't check
                // for 0xff. If last_type == current_type we don't need to update it
            } else if (ss->slice_type[i] != last_type) {
                *direction = 0;
                return i+1;
            }
        }
    }
    // This should only happen when level is complete
    return -1;
}

#if (RUN_TEST)
#include <stdio.h>
#include <stdlib.h>
#include <inttypes.h>
static void
RunTests() {
    int const slice_count = 16;
    int const color_count = 8;
    static int search_direction = 1;
    int direction = 0;
    int index;
    int i;

    printf("starting big init\n");
    //int const ss_count = INT32_MAX / 1024;//1024*1024;
    int const ss_count = 1024*1024;
    size_t const ss_size = sizeof(struct SliceStack) * ss_count;
    struct SliceStack* ss = (struct SliceStack*)malloc(ss_size);
    int *initial_frag = (int*)malloc(ss_count*sizeof(int));
    int *moves_to_solve = (int*)malloc(ss_count*sizeof(int));
    memset(ss, 0, ss_size);
    memset(moves_to_solve, 0, ss_count*sizeof(int));

    for (i = 0; i < ss_count; ++i) {
        SliceStack_Create(ss+i, slice_count, color_count);
        initial_frag[i] = SliceStack_Fragmentation(ss+i);
        /*int const c1 = SliceStack_IsComplete(ss+i);
        int const c2 = SliceStack_IsComplete2(ss+i);
        if (c1 != c2) { printf("%d != %d:%d\n", c1, c2, SliceStack_Fragmentation(ss+i)); }*/
    }

    printf("starting big solve\n");

    unsigned long long iters = 0;
    for (i = 0; i < ss_count; ++i) {
        direction = 0;
        while (!SliceStack_IsComplete(ss+i)) {
            ++iters;
            index = SliceStack_FindSingleJoiningMove2(ss+i, &direction);
            if (index < 0) {
                index = SliceStack_FindFirstDoubleMove(ss+i,
                                                       search_direction, &direction);
                //search_direction = !search_direction;
            }
            if (index >= 0) {
                if (direction == 0)
                    direction = -1;
                SliceStack_Flip(ss+i, index+direction, direction);
                moves_to_solve[i]++;
            }
        }
    }

    printf("iters: %" PRIu64 "\n", iters);

    printf("finished big solve\n");

    for (i = 0; i < ss_count; ++i) {
        if (initial_frag[i] * 2 < moves_to_solve[i]) {
            printf("init frag : %d moves : %d\n", initial_frag[i], moves_to_solve[i]);
        }
    }

    free(ss);
    free(moves_to_solve);
    free(initial_frag);

    printf("big solve done\n");
}

int main(int argc, char** argv) {
    RunTests();
    return 0;
}
#endif // RUN_TESTS

/* vim: set ts=4 sts=4 sw=4 et : */

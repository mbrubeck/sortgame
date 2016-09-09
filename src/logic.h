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

#ifndef _SORTGAME_LOGIC_H_
#define _SORTGAME_LOGIC_H_

#define MAX_SLICES (64/4)

// Within 2 moves you should always be able to decrease fragmentation
// e.g. Move color to edge, move edge to other color
// Within those 2 moves, fragmentation should not (need to) increase

// Scoring could be tangent/angle of a line using move number on one
// axis and fragmentation on the other.

// Scoring could also be based on the total distance for pieces to have been
// moved next to like-color piece; smaller distance == larger score
// You (almost; edge pieces can't flip) always have two options of flipping
// the stack; Left or right.

// Bonus could be based on the number of moves that decreased frag-
// mentation

struct SliceStack {
    // number of valid slices in slices
    int count;
    // number of colors present in the stack
    int color_count;
    //
    unsigned char slice_type[MAX_SLICES];
    void* slice_object[MAX_SLICES];
};

void SliceStack_Create(struct SliceStack *ss, int count, int color_count);
void SliceStack_Flip(struct SliceStack *ss, int index, int direction);
int SliceStack_IsComplete(struct SliceStack *ss);
int SliceStack_IsComplete2(struct SliceStack *ss);
int SliceStack_Fragmentation(struct SliceStack *ss);
int SliceStack_Fragmentation2(struct SliceStack *ss);
// Checks edge slices against disconnected like-colored slices, returns flip
// index >= 0
int SliceStack_FindSingleJoiningMove(struct SliceStack *ss, int *direction);
int SliceStack_FindSingleJoiningMove2(struct SliceStack *ss, int *direction);
// Searches for disconnected like-colored slices, moves one (set) of them to
// edge.  FindSingleJoiningMove should then connect them after this move.
int SliceStack_FindFirstDoubleMove(struct SliceStack *ss, int search_direction,
                                   int *direction);

#endif // _SORTGAME_LOGIC_H_

/* vim: set ts=4 sts=4 sw=4 et : */

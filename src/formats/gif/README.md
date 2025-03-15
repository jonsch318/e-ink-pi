# How does GIF work?

A general description. The specifications are quite good for the file and byte structure but do not cleanly explain the image and animation structure

GIF works in image_data blocks. These can be some part of the virtual monitor (i.e. the whole image) or be of the same size.

The control how these individual blocks fit together into one or multiple images is controlled by the graphics control extension. The first important setting is the `delay-time` if it is set to 0 (or no graphics control extension is given) then all blocks are layered on top of another into one image.

Say there are 2 image blocks then a graphic control extension with delay-time 100 then image block and then another image block.

Then it would result in the 3 image blocks being layered upon one another then waiting 100 milliseconds and then layering the fourth image block. In reality this should not happen. The best approach is to define the behavior with a control extension for each image block.

## Internal Representation
We want an internal image and animation/video structure that can be adapted to multiple file types/formats.

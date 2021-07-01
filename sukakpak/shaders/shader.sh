#!/bin/zsh
rm -rf *.spv
glslc alt.frag -o alt.frag.spv
glslc alt.vert -o alt.vert.spv
glslc main.frag -o main.frag.spv
glslc main.vert -o main.vert.spv
glslc push.frag -o push.frag.spv
glslc uniform.frag -o uniform.frag.spv
glslc push.vert -o push.vert.spv

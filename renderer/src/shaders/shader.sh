#!/bin/zsh
rm -rf *.spv
glslc main.frag -o main.frag.spv
glslc main.vert -o main.vert.spv
glslc push.frag -o push.frag.spv
glslc push.vert -o push.vert.spv

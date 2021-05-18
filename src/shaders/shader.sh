#!/bin/zsh
rm -rf *.spv
glslc main.frag -o main.frag.spv
glslc main.vert -o main.vert.spv

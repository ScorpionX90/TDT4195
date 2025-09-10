---
# This is a YAML preamble, defining pandoc meta-variables.
# Reference: https://pandoc.org/MANUAL.html#variables
# Change them as you see fit.
title: TDT4195 Exercise 2
author:
- Øyvind Nestvold
- Fredrik Robertsen
date: \today # This is a latex command, ignored for HTML output
lang: en-US
papersize: a4
geometry: margin=4cm
toc: false
toc-title: "Table of Contents"
toc-depth: 2
numbersections: true
header-includes:
# The `atkinson` font, requires 'texlive-fontsextra' on arch or the 'atkinson' CTAN package
# Uncomment this line to enable:
#- '`\usepackage[sfdefault]{atkinson}`{=latex}'
colorlinks: true
links-as-notes: true
# The document is following this break is written using "Markdown" syntax
---

## Task 1)

### a-b) Color VBO and shading coloring

This task was already implemented as part of exercise 1. But for ease of access we include the same render here.

![](./images/gradient_triangles.png)

OpenGL does a simple interpolation between the vertex colors, and assigns a color to a pixel during shading based on a weighted blend of the vertex colors based on how close the location of a pixel is to the vertices of the fragment. All weights should add up to 1, resulting in a varying mix of all the vertex colors at any given point in the fragment. In other words it does a simple linear interpolation.


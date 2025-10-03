---
# This is a YAML preamble, defining pandoc meta-variables.
# Reference: https://pandoc.org/MANUAL.html#variables
# Change them as you see fit.
title: TDT4195 Exercise 3
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

## Task 1c)

For this task, we simply applied the normal vectors as components of the fragment shaders color vector. The resulting surface can be seen in the image below:

![](../report/images/crater.png)

## Task 1d)

Applying the simple formula for the color vector of the fragment shader yields a much more convincing lighting model as seen by the image below, though the resolution of the model is somewhat questionable in terms of realism.

![](../report/images/Lambertian.png)

## Task 2c)

Applying a scenegraph to the scene, ensured we had to revamp alot of or rendering logic. (All glory to the almighty ManualDrop<Pin<Box<SceneNode>>>>) But disregarding all of that, it works very quite well. Below is an image of the rendered helicopter.

![](../report/images/heli.png)

## Task 5a)

Below we highlight a lighting issue in our scene where transformed (moving) objects have static normals in relation to the light vector, thus producing unnatural lighting conditions where the "sun" follows the rotation of the rotating objects independently. For example a rotating helicopter may be brightly lit, but once rotated 180 degrees, is left in the dark.

![Brightly lit, non-rotated helicopter](../report/images/brightside-of-the-heli-moon.png)
![Dark, rotated helicopter](../report/images/dark-side-heli.png)

## Task 5c)

Fixing the lighting is a simple case of appyling the same transform as the object itself (excluding view projections) to each normal, and normalizing the normal to avoid maxing (or minmizing) the range of the color vector (which would result in a pure white or black image in most circumstances). Below are images of the new, corrected behaviour:

![Correctly lit, non-rotated helicopter](../report/images/bright_correct_heli.png)
![Correctly lit, rotated helicopter](../report/images/dark_correct_heli.png)


## Task 6a)

We solved this by producing and structuring 5 helicopter root `SceneNodes` in a loop during our setup, aswell as running a loop over these nodes during the draw step. Both loops are controlled by a single variable. We discussed reusing a single `SceneNode` and deemed it possible, but settled on the above model as keeping track of the transform for the individual models would be impractical in any other scenario than parameterized animation. And since we wanted to implement player controls for one of the helicopter in the optional tasks, we deemed the reuse somewhat redundant extra complexity. Below is an image of the coordinated choppers:

![5 synchronized (non-colliding) helicopters rendered in a loop](../report/images/5-heli.png)

## Optional Tasks (b), c), d) + f))

### Optional b)

### Optional c)

### Optional d)

The logic behind our approach is very quite simple, but at the same time a bit more complex than it ever had to be. Since we were given the liberty of opening the door, without ever haivng to close it, why not just get rid of it all together with a fair push?

The implementation started by making a simple approximation of the path the door might take. We opted for a simple version of the motion equations, and added some randomness to the amount of rotation and speed of the door in the x-z plane.

The release of the door is tracked by the event where the user presses the `E` key, in which case an animation context is recorded and passed to the world. The context keeps track of the animation function, origin of the animation, and the start time of the animation. All animation contexts are processed at drawtime.

Now, since the door is still parented under the helicopter at the time it is shot out of the helicopter, we must detach it to let it fall freely of the transform of the helicopter. This is done by recording the global transform of the door at the time of release (Note that all parents of the helicopter is (0,0,0), meaning the global transform is simply set the helicopter), and thereafter unchilding it from the helicopter.

Finally, the helicopter animationctx is stopped if it "falls out of the world", in other words, if its y-position is less than `-1000.0`. At this point the scenenode of the door is deleted.

### Optional f)

The cake is a lie

![](../report/images/easter_egg.png)
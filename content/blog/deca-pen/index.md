+++
title = "Deca-pen"
description = "Recreating the pen tracking method DodecaPen"
date = 2024-12-01
updated = 2023-12-18
draft = false

[taxonomies]
tags = ["Devlog","Research","Python"]
+++

As part of a system to help blind people fill out paper forms without human assistance, I'm recreating a pen tracking method outlined in [_DodecaPen_](http://media.ee.ntu.edu.tw/research/DodecaPen/) by Wu, Po-Chen et al. I'm using [this code](https://github.com/arkadeepnc/Visual-6-DoF-pose-tracker) as a base.

# Current progress

Here's a demo. The yellow dot is the estimated tip position in 3d, the blue dot is the estimated tip position projected onto the paper.

{{ video(sources=["demo.mp4", "demo.webm"]) }}

The position is quite noisy. I think this has two main factors: Low camera resolution, and uncalibrated relative marker positions.

To resolve the low resolution noise, I applied a Kalman filter to get this result:

{{ video(sources=["demo_kalman.mp4", "demo_kalman.webm"]) }}

You can see that the noise while the same set of markers is being detected is minimal. The noise mainly appears when the detected markers change. Over winter break I'll be callibrating the relative positions of the markers to each other, as currently we assume they're in an ideal configuration when they're not.

There's also a part where the position flies off. I think this is due to the markers being fit to positions where the depth changes drastically between frames. I'm guessing that this will disappear after calibration.

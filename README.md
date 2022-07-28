#  Vulkan for Rust Step by Step

##  What is this?

Vulkan is hard.
Legend says it that it takes 1k lines of code for just a triangle!
Tutorials are nice and all, but if only there was some repo to ~~copy~~ *reference* code.


This this **not** a tutorial!
Rather, it is just my own exploration of Vulkan saved as a collection of commits.


FYI, this specific repo uses [`winit`](https://github.com/rust-windowing/winit) (windowing) and [`ash`](https://github.com/ash-rs/ash) (sane Vulkan bindings).
Oh, and it only works on Linux (or other X11) :p.

##  Latest Commits

>  I might have forgotten to fill this in for every commit.
>  Just use `git log --all --oneline` or something lazy guy. :)

1. Initialization, Render Loop, and Color! ~ 566 SLOC

![](/images/0.png)

2. Hello Triangle w/ RGB! ~ 748 SLOC

![](/images/1.png)

3. Exact Same Triangle with Vertex Buffers! ~ 854 SLOC

![](/images/2.png)

4. Yet Another Triangle but Spinning! ~ 906 SLOC

![](/images/3.gif)

5. o/ Spinning Cube! ~ 1063 SLOC

![](/images/4.gif)

6. Double Buffering! ~ 1111 SLOC

![](/images/5.gif)

7. Spinning Cube with Uniforms! ~ 1262 SLOC

![](/images/6.gif)

##  Mistake Log

There will be a few commits with small afterthoughts or mistakes.
Here is a list of currently known afterthoughts / mistakes.
Be sure to check the latest commit's list.

- Cleanup crashes with multiple Vertex Buffers ~ affects #3-4
- Forgot to include color attachment subpass dependency ~ affects #2-4
- Improved garbage README video quality ~ affects #4-5
- Forgot to reset command buffers ~ affects #1-5
- Fixed broken dependency versions ~ affects #3-5
- Github doesn't support webm ~ affects #6


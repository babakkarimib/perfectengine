A non‑polygon‑based graphics engine written in Rust using SDL2 and WGPU.

After <a href="https://rust-lang.org/tools/install/">setting up your rust environment</a>
, just pull the code and run with: <br>
`cargo run --release`

Command line arguments to be used:
<ul>
    <li><b>gpu</b>: runs in GPU mode (enabled by default)</li>
    <li><b>cpu</b>: runs in CPU mode</li>
    <li><b>fullscreen</b>: runs in fullscreen mode</li>
    <li><b>framerate</b>: shows framerate in the command line</li>
    <li><b>w=[width]</b>: sets the width</li>
    <li><b>h=[height]</b>: sets the height</li>
</ul>

You can use any or a combination of these arguments with space between them like this:
`cargo run --release -- [argument1] [argument2]`

<br>
The main feature is that this engine does not render based on polygon calculations, but rather it uses a 2D pixel concept that has 3D properties. No rasterization is done, pixels just overlap. Also ray tracing is added as a layer on top. This project is built using the Rust language, WGPU library and SDL2. All GPU calculations are done in shaders.
<br>

<br>
The next step will be to detect the surface angel based on the 3D position of the pixels around a pixel and then use it to detect and then use the reflection factor based on the angel of the camera and the light source to the surface.
<br>
<br>

<img src="./frame-6.png" alt="Screenshot">

Note: On any platform if you just run the code you get the realtime demo. Here are the controls that are used in the realtime demo video:

<ul>
  <li>Mouse left drag: object rotation</li>
  <li>Mouse right drag: moves light</li>
  <li>Mouse wheel: light intensity</li>
  <li>Mouse middle + Left Ctrl drag: light rotation</li>
</ul>

<br>
Realtime Demo: https://drive.google.com/file/d/12gd-R1CQ-atdvcHmsXghGv22BQgWU_ba/view?usp=drivesdk

<br>
<br>
The documentations are not ready yet but the code base is pretty much short and self-documented. So I hope you enjoy the code.

<br>
<br>
<h3>Sponsorship</h3>

Ethereum: `0x53A6F9c6a717d5012629564c864f07330909bBA8`

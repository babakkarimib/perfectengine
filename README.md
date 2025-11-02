A non‑polygon‑based graphics engine written in Rust using SDL2 and WGPU.

Just pull the code and run with: <br>
`cargo run --release`

<br>
The main feature is that this engine does not render based on polygon calculations, but rather it uses a 2D pixel concept that has 3D properties. No rasterization is done, pixels just overlap. Also ray tracing is added as a layer on top. This project is built using the Rust language, WGPU library and SDL2. All GPU calculations are done in shaders.
<br>

<br>
The next step will be to detect the surface angel based on the 3D position of the pixels around a pixel and then use it to detect and then use the reflection factor based on the angel of the camera and the light source to the surface.
<br>
<br>

<img src="./frame-6.png" alt="Screenshot">

<br>
Demo: https://drive.google.com/file/d/12gd-R1CQ-atdvcHmsXghGv22BQgWU_ba/view?usp=drivesdk

<br>
<br>
The documentations are not ready yet but the code base is pretty much short and self-documented. So I hope you enjoy the code.

<br>
<br>
<h3>Sponsorship</h3>

Ethereum: `0x53A6F9c6a717d5012629564c864f07330909bBA8`

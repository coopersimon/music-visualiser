# Music Visualiser

A program that creates visuals based on audio. You can create custom mappings based on a script format.

## Running

It can be run as follows:

```sh
./visualiser --audio [AUDIO_FILE] --script [SCRIPT_FILE]
```

## Making a script

To create a scene, write a script file as follows:

```
object_1 {
    visual_parameter_1 = expression,
    visual_parameter_2 = expression,
    ...
}
object_2 {
    visual_parameter_1 = expression,
    visual_parameter_2 = expression,
    ...
}
```

There are a variety of objects to choose from. Each has its own list of visual parameters.

Each parameter is bound to an expression, which can either be constant or vary with time according to the audio input.

### Example

For a red circle placed at the centre of the screen, which varies in size based on the volume of the audio:

```
circle {
    x = 0.0,
    y = 0.0,
    radius = level * 1.5,
    line_width = 0.01,
    r = 1.0,
    g = 0.0,
    b = 0.0
}
```

TODO: image

For further examples, see the `examples` folder.

## Reference

Note that the screen space is defined as follows:

```
  ----- 1.0 -----
  |             |
-1.0     0     1.0
  |             |
  ---- -1.0 -----
```

With X varying from left-to-right as -1.0 to 1.0, and Y varying from bottom-to-top as -1.0 to 1.0.

### Objects

- `circle`
  - `x`, `y`: position of centre
  - `radius`: size of radius
  - `line_width`: width of line
  - `r`, `g`, `b`: colour
- `quad`
  - `x`, `y`: position of bottom-left
  - `width`, `height`: size of quad
  - `r`, `g`, `b`: colour

### Audio parameters

- `level`: The (absolute) amplitude of the audio track. 1.0 is max.
- `time`: The time into the song, in seconds.
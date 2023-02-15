# Kronos
Terminal Music Player Written In Rust. For offline listening and local files.

![Music](assets/music_tab.png?raw=true)
![Controls](assets/controls_tab.png?raw=true)


# Specs 

Supports the following formats

+ mp3
+ mp4
+ m4a 
+ wav
+ aac
+ flac

# Setup

1. Download Release 
``
    https://github.com/TrevorSatori/Kronos/releases/tag/v0.69
``
2. run program

# Customization

If the color scheme above isn't for you, change it. Kronos default config path is
``
    ~/.config/kronos/config.toml
``

Below are all the color options for each parameter in the toml file. 

black | blue | green | red |
yellow | magenta | cyan | gray |
dark gray | light red | light green | light yellow | light blue | 
light magenta | light cyan | white | 
rgb 0 - 255 |


![Customized](assets/customized.png?raw=true)

The above makes use of both premade color options as well as rgb values and can be recreated with the following inside a config.toml.

``
[theme]
foreground = "200, 100, 250"
background = "black"
highlight_foreground = "white"
highlight_background = "255, 165, 0" 
``
                        
# Using This Software?
Make sure to leave a star.





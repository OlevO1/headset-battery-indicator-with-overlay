# Headset Battery Indicator

Adds a small icon to the "system tray" area of the Windows task bar, which displays the battery level of most* connected wireless headsets.

![Screenshot of indicator on Windows task bar](docs/icon-screenshot.png)

## Features

* Works on Windows 10+
* Built using Rust, with very low resource usage (<1MB RAM)
* Supports most non-bluetooth headsets (SteelSeries, Logitech, Corsair, HyperX)
  * See all [supported headsets here](https://github.com/Sapd/HeadsetControl?tab=readme-ov-file#supported-headsets).
    * Some headsets (notably Arctis Wireless 1) are may not work even though they are listed as supported.
* Shows a little green dot to indicate charging

  ![Charging icon](docs/icon-charging.png)

Headset Battery Indicator depends on [Sapd/HeadsetControl](https://github.com/Sapd/HeadsetControl), which is licensed under GPL v3.

## Installation

* Download the [latest release](https://github.com/aarol/headset-battery-indicator/releases/latest) and run the installer

> Running the installer may result in a Windows defender SmartScreen warning. This happens to all executables that don't have a large enough install count. There's no way around it other than paying hundreds of dollars every year for a signed certificate from Microsoft :(

## Security

The code that is in this repository is the code that is in the executable. There is a [Github Action](https://github.com/aarol/headset-battery-indicator/actions) that builds the code from source and creates the release in the [releases page](https://github.com/aarol/headset-battery-indicator/releases).

The GitHub release is marked as immutable, so once the executable is built by the Actions workflow, it cannot be modified by me or anyone else. This ways, you can be sure that the code you're running is the same code that is in this repository.

## Troubleshooting

If you're experiencing crashes or other issues, you can try running the `headset-battery-indicator-debug.exe` located at `%localAppData%\Programs\HeadsetBatteryIndicator` or look at the log file located in the same folder.

### Why does it only show 100%, 75%, 50%, 25% or 0%?

This is limitation of the headsets themselves, as some devices only expose 5 possible battery states.

### My headset is connected, but it still shows "No headphone adapter found"

Your headset might be unsupported due to being a new model. See [Adding a new headset](#adding-a-new-headset)

## Development

Rust and Cargo need to be installed.

First, download or compile the HeadsetControl executable [from here](https://github.com/sapd/HeadsetControl/).

Then, clone this repository and copy the `headsetcontrol.exe` file into the project root folder (where `README.md` is).

Finally, from the `headset-battery-indicator` folder, you can:

* Run the application: `cargo run --release`

* Install the application locally: `cargo install`

* Debug the application by pressing `F5` in VS Code with the Rust Analyzer and CodeLLDB extensions installed.

### Translations

Translations can be added to the [lang.rs](./src/lang.rs) file.

## Adding a new headset

Since version 3.0.0, the program gets the battery status by using [Sapd/HeadsetControl](https://github.com/Sapd/HeadsetControl). If the headset you're using isn't currently supported, you can either wait if someone else adds support for it, or try adding it yourself.

I have a post on my website going a bit into reverse-engineering the headset APIs: <https://aarol.dev/posts/arctis-hid>

Reading the [HeadsetControl wiki](https://github.com/Sapd/HeadsetControl/wiki/Development#problems) might be helpful for troubleshooting.

### License

This project is licensed under GNU GPL v3.

You’re free to use, modify, and redistribute it, as long as your version is also licensed under GPL v3, and you include the source code and license when you share it.
See the [LICENSE](./LICENSE) file for full terms.

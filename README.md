# gdrive
<img src="https://user-images.githubusercontent.com/720405/210108089-32b7a259-b384-49c3-a2d3-fe07a42791e2.png" width="100">

## Overview
gdrive is a command line utility for interacting with Google Drive. This is the successor of [gdrive2](https://github.com/prasmussen/gdrive), though at the moment only the most basic functionality is implemented.

## Getting started

### Requirements
* Google OAuth Client credentials, see [docs](/docs/create_google_api_credentials.md)

### Install binary
* Download the latest binary from [the release section](https://github.com/glotlabs/gdrive/releases)
* Unpack and put the binary somewhere in your PATH (i.e. `/usr/local/bin` on linux and macos)
* Note that the binary is not code signed and will cause a warning on windows and macos when running. This will be fixed later, but for now you can find a workaround via you favorite search engine.

### Add google account to gdrive
* Run `gdrive account add`
* This will prompt you for your google Client ID and Client Secret (see [Requirements](#requirements))
* Next you will be presented with an url
* Follow the url and give approval for gdrive to access your Drive
* You will be redirected to `http://localhost:8085` (gdrive starts a temporary web server) which completes the setup
* Gdrive is now ready to use!


### Using gdrive on a remote server
Part of the flow for adding an account to gdrive requires your web browser to access `localhost:8085` on the machine that runs gdrive.
This makes it tricky to set up accounts on remote servers. The suggested workaround is to add the account to you local machine first
and then copy the configuration to the remote server. A `account export` and `account import` function will be added later
to simplify this process.

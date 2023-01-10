# gdrive

<img src="https://user-images.githubusercontent.com/720405/210108089-32b7a259-b384-49c3-a2d3-fe07a42791e2.png" width="100">

## Overview

gdrive is a command line application for interacting with Google Drive. This is the successor of [gdrive2](https://github.com/prasmussen/gdrive), though at the moment only the most basic functionality is implemented.

## Community

Join our [discord server](https://discord.gg/zPcdFUqKeQ) to discuss everything gdrive.

## Sponsor

Help keep this project alive. By sponsoring the [gdrive tier](https://github.com/sponsors/prasmussen)
you will help support:

- Keeping up with api changes
- Development of new features
- Fixing and answering of issues
- Writing of guides and docs

## Getting started

### Requirements

- Google OAuth Client credentials, see [docs](/docs/create_google_api_credentials.md)

### Install binary

- Download the latest binary from [the release section](https://github.com/glotlabs/gdrive/releases)
- Unpack and put the binary somewhere in your PATH (i.e. `/usr/local/bin` on linux and macos)
- Note that the binary is not code signed and will cause a warning on windows and macos when running. This will be fixed later, but for now you can find a workaround via you favorite search engine.

### Add google account to gdrive

- Run `gdrive account add`
- This will prompt you for your google Client ID and Client Secret (see [Requirements](#requirements))
- Next you will be presented with an url
- Follow the url and give approval for gdrive to access your Drive
- You will be redirected to `http://localhost:8085` (gdrive starts a temporary web server) which completes the setup
- Gdrive is now ready to use!

### Using gdrive on a remote server

Part of the flow for adding an account to gdrive requires your web browser to access `localhost:8085` on the machine that runs gdrive.
This makes it tricky to set up accounts on remote servers. The suggested workaround is to add the account on your local machine and import it on the remote server:
1. [local] Run `gdrive account add` 
2. [local] Run `gdrive account export <ACCOUNT_NAME>`
3. [local] Copy the exported archive to the remote server
4. [remote] Run `gdrive account import <ARCHIVE_PATH>`

### Credentials
Gdrive saves your account credentials and tokens under `$HOME/.config/gdrive3/`.
You don't usually need to use these files directly, but if someone gets access to them, they will also be able to access your Google Drive. Keep them safe.

### Gdrive on virtual machines in the cloud
There are some issues communicating with the Drive API from certain cloud providers.
For example on an AWS instance the api returns a lot of `429 Too Many Requests` / `503 Service Unavailable` / `502 Bad Gateway` errors while uploading.
While the same file uploads without any errors from a Linode instance.
Gdrive has retry logic built in for these errors, but it can slow down the upload significantly.
To check if you are affected by these errors you can run the `upload` command with these flags: `--print-chunk-errors` `--print-chunk-info`.
